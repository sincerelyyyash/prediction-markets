use crate::services::db_event_publisher::publish_db_event;
use crate::types::event_types::{
    CreateEventRequest, DeleteEventRequest, ResolveEventRequest, UpdateEventRequest,
};
use crate::utils::redis_stream::send_request_and_wait;
use actix_web::{delete, post, put, web, HttpResponse, Responder};
use chrono::Utc;
use engine::types::db_event_types::{
    DbEvent, EventCreatedEvent, EventDeletedEvent, EventResolvedEvent, EventUpdatedEvent,
    OutcomeData,
};
use rand::{thread_rng, Rng};
use redis_client::RedisRequest;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

#[post("/create-event")]
pub async fn create_event(req: web::Json<CreateEventRequest>) -> impl Responder {
    if let Err(e) = req.0.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let event_id = generate_safe_id();

    let mut outcomes_data = Vec::new();
    let mut created_outcomes = Vec::new();

    for outcome_input in &req.outcomes {
        let outcome_id = generate_safe_id();
        let yes_market_id = generate_safe_id();
        let no_market_id = generate_safe_id();

        outcomes_data.push(OutcomeData {
            outcome_id,
            name: outcome_input.name.clone(),
            status: outcome_input.status.clone(),
            yes_market_id,
            no_market_id,
        });

        created_outcomes.push(json!({
            "id": outcome_id,
            "event_id": event_id,
            "name": outcome_input.name,
            "status": outcome_input.status,
            "yes_market_id": yes_market_id,
            "no_market_id": no_market_id
        }));
    }

    let event = DbEvent::EventCreated(EventCreatedEvent {
        event_id,
        slug: req.slug.clone(),
        title: req.title.clone(),
        description: req.description.clone(),
        category: req.category.clone(),
        status: req.status.clone(),
        resolved_at: req.resolved_at.clone(),
        created_by: req.created_by,
        outcomes: outcomes_data.clone(),
        timestamp: Utc::now(),
    });

    if let Err(e) = publish_db_event(event).await {
        eprintln!("Failed to publish event creation: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to create event"
        }));
    }

    let init_markets_data = json!({
        "event_id": event_id,
        "outcomes": outcomes_data.iter().map(|outcome| {
            json!({
                "outcome_id": outcome.outcome_id,
                "yes_market_id": outcome.yes_market_id,
                "no_market_id": outcome.no_market_id,
            })
        }).collect::<Vec<_>>()
    });

    let request_id = Uuid::new_v4().to_string();
    let init_request = RedisRequest::new(
        "engine",
        "init-event-markets",
        "Initialize event markets",
        init_markets_data,
    );

    if let Err(e) = send_request_and_wait(request_id, init_request, 10).await {
        eprintln!("Failed to initialize markets: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Event created but failed to initialize markets"
        }));
    }

    HttpResponse::Created().json(json!({
        "status":"success",
        "message": "Event created successfully",
        "event": {
            "id": event_id,
            "slug": req.slug,
            "title": req.title,
            "description": req.description,
            "category": req.category,
            "status": req.status,
            "resolved_at": req.resolved_at,
            "winning_outcome_id": null,
            "created_by": req.created_by
        },
        "outcomes": created_outcomes
    }))
}

#[post("/resolve-event")]
pub async fn resolve_event(req: web::Json<ResolveEventRequest>) -> impl Responder {
    if let Err(e) = req.0.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let event_request_id = Uuid::new_v4().to_string();
    let event_request = RedisRequest::new(
        "db_worker",
        "get_event_by_id",
        "Get event by ID",
        json!({
            "event_id": req.event_id,
        }),
    );

    let event_response = match send_request_and_wait(event_request_id, event_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                if response.status_code == 404 {
                    return HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "Event not found"
                    }));
                }
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to load event"
                }));
            }
            response
        }
        Err(e) => {
            eprintln!("Failed to fetch event: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to load event"
            }));
        }
    };

    let event_data = &event_response.data["event"];
    let event_status = event_data["status"].as_str().unwrap_or("");
    let event_winning_outcome_id = event_data["winning_outcome_id"].as_i64();
    let event_id = event_data["id"].as_i64().unwrap_or(0);

    if event_status == "RESOLVED" || event_winning_outcome_id.is_some() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Event already resolved"
        }));
    }

    let outcome_request_id = Uuid::new_v4().to_string();
    let outcome_request = RedisRequest::new(
        "db_worker",
        "get_outcome_by_id",
        "Get outcome by ID",
        json!({
            "outcome_id": req.winning_outcome_id,
        }),
    );

    let outcome_response =
        match send_request_and_wait(outcome_request_id, outcome_request, 10).await {
            Ok(response) => {
                if response.status_code >= 400 {
                    if response.status_code == 404 {
                        return HttpResponse::BadRequest().json(json!({
                            "status": "error",
                            "message": "Winning outcome not found"
                        }));
                    }
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "failed to load winning outcome"
                    }));
                }
                response
            }
            Err(e) => {
                eprintln!("Failed to fetch outcome: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "failed to load winning outcome"
                }));
            }
        };

    let outcome_data = outcome_response
        .data
        .get("outcome")
        .cloned()
        .unwrap_or_else(|| serde_json::json!(null));
    let outcome_event_id = outcome_data["event_id"].as_i64().unwrap_or(0);
    if outcome_event_id != event_id {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Outcome does not belong to this event"
        }));
    }

    let resolved_at_value = req
        .resolved_at
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());

    let event_resolved = DbEvent::EventResolved(EventResolvedEvent {
        event_id: req.event_id,
        status: req.status.clone(),
        resolved_at: resolved_at_value.clone(),
        winning_outcome_id: req.winning_outcome_id,
        timestamp: Utc::now(),
    });

    if let Err(e) = publish_db_event(event_resolved).await {
        eprintln!("Failed to publish event resolution: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to resolve event"
        }));
    }

    let close_markets_data = json!({
        "event_id": req.event_id,
        "winning_outcome_id": req.winning_outcome_id,
    });

    let request_id = Uuid::new_v4().to_string();
    let close_request = RedisRequest::new(
        "engine",
        "close-event-markets",
        "Close event markets",
        close_markets_data,
    );

    if let Err(e) = send_request_and_wait(request_id, close_request, 10).await {
        eprintln!("Failed to close markets: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Event resolved but failed to close markets"
        }));
    }

    HttpResponse::Ok().json(json!({
        "status":"success",
        "message": "Event resolved successfully",
        "event_id": req.event_id,
        "winning_outcome_id": req.winning_outcome_id,
        "resolved_at": resolved_at_value
    }))
}

#[put("/update-event")]
pub async fn update_event(req: web::Json<UpdateEventRequest>) -> impl Responder {
    if let Err(e) = req.0.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let event_request_id = Uuid::new_v4().to_string();
    let event_request = RedisRequest::new(
        "db_worker",
        "get_event_by_id",
        "Get event by ID",
        json!({
            "event_id": req.event_id,
        }),
    );

    let event_response = match send_request_and_wait(event_request_id, event_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                if response.status_code == 404 {
                    return HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "Event not found"
                    }));
                }
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to load event"
                }));
            }
            response
        }
        Err(e) => {
            eprintln!("Failed to fetch event: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to load event"
            }));
        }
    };

    let event_data = &event_response.data["event"];
    let event_status = event_data["status"].as_str().unwrap_or("");
    let event_winning_outcome_id = event_data["winning_outcome_id"].as_i64();

    if event_status == "RESOLVED" || event_winning_outcome_id.is_some() {
        return HttpResponse::BadRequest().json(json!({
            "status" : "error",
            "message" : "Cannot update a resolved event"
        }));
    }

    let slug = req
        .slug
        .as_ref()
        .unwrap_or(&event_data["slug"].as_str().unwrap_or("").to_string())
        .clone();
    let title = req
        .title
        .as_ref()
        .unwrap_or(&event_data["title"].as_str().unwrap_or("").to_string())
        .clone();
    let description = req
        .description
        .as_ref()
        .unwrap_or(&event_data["description"].as_str().unwrap_or("").to_string())
        .clone();
    let status = req
        .status
        .as_ref()
        .unwrap_or(&event_status.to_string())
        .clone();
    let category = req
        .category
        .as_ref()
        .unwrap_or(&event_data["category"].as_str().unwrap_or("").to_string())
        .clone();

    let event_updated = DbEvent::EventUpdated(EventUpdatedEvent {
        event_id: req.event_id,
        slug: slug.clone(),
        title: title.clone(),
        description: description.clone(),
        category: category.clone(),
        status: status.clone(),
        timestamp: Utc::now(),
    });

    if let Err(e) = publish_db_event(event_updated).await {
        eprintln!("Failed to publish event update: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to update event"
        }));
    }

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Event updated successfully",
        "event": {
           "id": req.event_id,
            "slug": slug,
            "title": title,
            "description": description,
            "category": category,
            "status": status,
            "resolved_at": event_data["resolved_at"],
            "winning_outcome_id": event_data["winning_outcome_id"],
            "created_by": event_data["created_by"]
        }
    }))
}

#[delete("/delete-event")]
pub async fn delete_event(req: web::Json<DeleteEventRequest>) -> impl Responder {
    if let Err(e) = req.0.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let event_request_id = Uuid::new_v4().to_string();
    let event_request = RedisRequest::new(
        "db_worker",
        "get_event_by_id",
        "Get event by ID",
        json!({
            "event_id": req.event_id,
        }),
    );

    let event_response = match send_request_and_wait(event_request_id, event_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                if response.status_code == 404 {
                    return HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "Event not found"
                    }));
                }
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to load event"
                }));
            }
            response
        }
        Err(e) => {
            eprintln!("Failed to fetch event: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to load event"
            }));
        }
    };

    let event_data = &event_response.data["event"];
    let event_status = event_data["status"].as_str().unwrap_or("");
    let event_winning_outcome_id = event_data["winning_outcome_id"].as_i64();

    if event_status == "RESOLVED" || event_winning_outcome_id.is_some() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "cannot delete a resolved event"
        }));
    }

    let event_deleted = DbEvent::EventDeleted(EventDeletedEvent {
        event_id: req.event_id,
        timestamp: Utc::now(),
    });

    if let Err(e) = publish_db_event(event_deleted).await {
        eprintln!("Failed to publish event deletion: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to delete event"
        }));
    }

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Event deleted succesfully",
        "event_id": req.event_id
    }))
}

fn generate_safe_id() -> u64 {
    let mut rng = thread_rng();
    rng.gen_range(1..=9_000_000_000_000_000u64)
}
