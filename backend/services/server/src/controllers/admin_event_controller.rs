use actix_web::{post, put, delete, web, HttpResponse, Responder};
use crate::types::event_types::{CreateEventRequest, DeleteEventRequest, ResolveEventRequest, UpdateEventRequest};
use crate::models::market_model::{EventTable, OutcomeTable, MarketTable};
use serde_json::json;
use validator::Validate;
use sqlx::PgPool;
use chrono::Utc;
use redis_client::RedisManager;
use log::warn;


#[post ("/create-event")]
pub async fn create_event(db_pool: web::Data<PgPool>, req: web::Json<CreateEventRequest>) -> impl Responder {
    if let Err(e) = req.0.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }))
    }

    let mut tx  = match db_pool.begin().await {
        Ok(t) => t,
        Err(e)=> {
            return HttpResponse::InternalServerError().json(json!({
                "status":"error",
                "message":"Failed to start the database transaction"
            }));
        }
    };

    let event_res = sqlx::query_as!(
        EventTable,
        r#"
        INSERT INTO events (slug, title, description, category, status, resolved_at, created_by)
        VALUES($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, slug, title, description, category, status, resolved_at, winning_outcome_id, created_by
        "#,
        req.slug,
        req.title,
        req.description,
        req.category,
        req.status,
        req.resolved_at,
        req.created_by as i64
    )
    .fetch_one(&mut *tx)
    .await;

    let event = match event_res {
        Ok(e) => e,
        Err(e) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "status":"error",
                "message":"failed to create event"
            }));
        }
    };

    let mut created_outcomes = Vec::new();

    for outcome_input in &req.outcomes {
    
    let outcome_result = sqlx::query_as!(
        OutcomeTable,
        r#"
        INSERT INTO outcomes (event_id, name, status)
        VALUES($1, $2, $3)
        RETURNING id, event_id, name, status
        "#,
        event.id,
        outcome_input.name,
        outcome_input.status
    )
    .fetch_one(&mut *tx)
    .await;

    let outcome = match outcome_result {
        Ok(o) => o,
        Err(e) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("failed to create outcome: {}", outcome_input.name)
            }));
        }
    };

    let yes_market_result = sqlx::query_as!(
        MarketTable,
        r#"
        INSERT INTO markets (outcome_id, side)
        VALUES ($1, $2)
        RETURNING id, outcome_id, side
        "#,
        outcome.id,
        "YES"
    )
    .fetch_one(&mut *tx)
    .await;

    if let Err(e) = yes_market_result {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("failed to create YES market for outcome: {}", outcome.name)
        }));
    }

    let no_market_result = sqlx::query_as!(
        MarketTable,
        r#"
        INSERT INTO markets (outcome_id, side)
        VALUES ($1, $2)
        RETURNING id, outcome_id, side
        "#,
        outcome.id,
        "NO"
    )
    .fetch_one(&mut *tx)
    .await;

    if let Err(e) = no_market_result {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("failed to create NO market for outcome: {}", outcome.name)
        }));
    }

        created_outcomes.push(outcome);
    }

    match tx.commit().await {
        Ok(_) => {
            
            if let Some(redis_manager) = RedisManager::global() {
                if let Err(e) = redis_manager.delete("events:all").await {
                    warn!("Failed to invalidate cache after event creation: {:?}", e);
                }
            }

            HttpResponse::Created().json(json!({
                "status":"success",
                "message": "Event created successfully",
                "event": {
                    "id":event.id,
                    "slug": event.slug,
                    "title": event.title,
                    "description": event.description,
                    "category":event.category,
                "status": event.status,
                    "resolved_at": event.resolved_at,
                    "winning_outcome_id": event.winning_outcome_id,
                    "created_by": event.created_by
                },
                "outcomes": created_outcomes.iter().map(|o| json!({
                    "id": o.id,
                    "event_id": o.event_id,
                    "name": o.name,
                    "status": o.status
                })).collect::<Vec<_>>()
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status":"Error",
                "message": "Failed to create event"
            }))
        }
    }
}

#[post ("/resolve-event")]
pub async fn resolve_event(db_pool: web::Data<PgPool>, req: web::Json<ResolveEventRequest>) -> impl Responder {
    if let Err(e) = req.0.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let mut tx = match  db_pool.begin().await {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "Error",
                "message": "Internal server error"
            }));
        }
    };

    let event = match sqlx::query_as!(
        EventTable,
        r#"
        SELECT id, slug, title, description, category, status,
        resolved_at, winning_outcome_id, created_by
        FROM events
        WHERE id = $1
        FOR UPDATE
        "#,
        req.event_id as i64
    )
    .fetch_optional(&mut *tx)
    .await {
        Ok(Some(event)) => event,
        Ok(None) => {
            let _ = tx.rollback().await;
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "Event not found"
            }));
        }
        Err(_) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to load event"
            }));
        }
    };

    if event.status == "RESOLVED" || event.winning_outcome_id.is_some() {
        let _ = tx.rollback().await;
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Event already resolved"
        }));
    }

    let winning_outcome = match sqlx::query_as!(
        OutcomeTable,
        r#"
        SELECT id, event_id, name, status
        FROM outcomes
        WHERE id = $1
        "#,
        req.winning_outcome_id as i64
    )
    .fetch_optional(&mut *tx)
    .await {
        Ok(Some(outcome)) if outcome.event_id == event.id => outcome,
        Ok(Some(_)) => {
            let _ = tx.rollback().await;
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Outcome does not belong to this event"
            }));
        }
        Ok(None) => {
            let _ = tx.rollback().await;
            return HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Winning outcome not found"
            }));
        }
        Err(_) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "failed to load winning outcome"
            }));
        }
    };

    let resolved_at_value = req.resolved_at.clone().unwrap_or_else(|| Utc::now().to_rfc3339());

    if let Err(_) = sqlx::query!(
        r#"
        UPDATE events
        SET status = $1,
        resolved_at = $2,
        winning_outcome_id = $3
        WHERE id = $4
        "#,
        req.status,
        resolved_at_value,
        winning_outcome.id,
        event.id
    )
    .execute(&mut *tx)
    .await {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to update event"
        }));
    }

    if let Err(_) = sqlx::query!(
        r#"
        UPDATE outcomes
        SET status = 'RESOLVED'
        WHERE id = $1
        "#,
        winning_outcome.id
    )
    .execute(&mut *tx)
    .await {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to mark winning outcome"
        }));
    }

    if let Err(_) = sqlx::query!(
        r#"
        UPDATE outcomes 
        SET status = 'REJECTED'
        WHERE event_id = $1 AND id <> $2
        "#,
        event.id,
        winning_outcome.id 
    )
    .execute(&mut *tx)
    .await {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to mark losing outcomes"
        }));
    }

    match tx.commit().await {
        Ok(_) => {
            if let Some(redis_manager) = RedisManager::global() {
                let cache_key = format!("event:{}", event.id);
                if let Err(e) = redis_manager.delete("events:all").await {
                    warn!("Failed to invalidate events:all cache after resolution: {:?}", e);
                }
                if let Err(e) = redis_manager.delete(&cache_key).await {
                    warn!("Failed to invalidate event cache after resolution: {:?}", e);
                }
            }

            HttpResponse::Ok().json(json!({
                "status":"success",
                "message": "Event resolved successfully",
                "event_id": event.id,
                "winning_outcome_id": winning_outcome.id,
                "resolved_at": resolved_at_value
            }))
        }
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to finalise resolution"
        }))
    }
}

#[put ("/update-event")]
pub async fn update_event(db_pool: web::Data<PgPool>, req: web::Json<UpdateEventRequest>) -> impl Responder {
    if let Err(e) = req.0.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let event = match sqlx::query_as!(
        EventTable,
        r#"
        SELECT id, slug, title, description, category, status, resolved_at,
            winning_outcome_id, created_by
        FROM events
        WHERE id = $1
        "#,
        req.event_id as i64
    )
    .fetch_optional(db_pool.get_ref())
    .await {
        Ok(Some(event)) => event,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "Event not found"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to load event"
            }));
        }
    };

    if event.status == "RESOLVED" || event.winning_outcome_id.is_some() {
        return HttpResponse::BadRequest().json(json!({
            "status" : "error",
            "message" : "Cannot update a resolved event"
        }));
    }

    let slug = req.slug.as_ref().unwrap_or(&event.slug);
    let title = req.title.as_ref().unwrap_or(&event.title);
    let description = req.description.as_ref().unwrap_or(&event.description);
    let status = req.status.as_ref().unwrap_or(&event.status);
    let category = req.category.as_ref().unwrap_or(&event.category);

    let updated_event = match sqlx::query_as!(
        EventTable,
        r#"
        UPDATE events
        SET slug = $1,
            title = $2,
            description = $3,
            status = $4,
            category = $5
        WHERE id = $6
        RETURNING id, slug, title, description, category, status, resolved_at, winning_outcome_id, created_by
        "#,
        slug,
        title,
        description,
        status,
        category,
        req.event_id as i64
    )
    .fetch_optional(db_pool.get_ref())
    .await { 
        Ok(Some(event)) => event,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "Event not found after update"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to update event"
            }));
        }
    };

    if let Some(redis_manager) = RedisManager::global() {
        let cache_key = format!("event:{}", updated_event.id);
        if let Err(e) = redis_manager.delete("events:all").await {
            warn!("Failed to invalidate events:all cache after update: {:?}", e);
        }
        if let Err(e) = redis_manager.delete(&cache_key).await {
            warn!("Failed to invalidate event cache after update: {:?}", e);
        }
    }

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Event updated successfully",
        "event": {
           "id": updated_event.id,
            "slug": updated_event.slug,
            "title": updated_event.title,
            "description": updated_event.description,
            "category": updated_event.category,
            "status": updated_event.status,
            "resolved_at": updated_event.resolved_at,
            "winning_outcome_id": updated_event.winning_outcome_id,
            "created_by": updated_event.created_by
        }
    }))
}

#[delete("/delete-event")]
pub async fn delete_event(db_pool: web::Data<PgPool>, req: web::Json<DeleteEventRequest>) -> impl Responder {
    if let Err(e) = req.0.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let mut tx = match db_pool.begin().await {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to start the database transaction"
            }));
        }
    };

    let event = match sqlx::query_as!(
        EventTable,
        r#"
        SELECT id, slug, title, description, category, status,
        resolved_at, winning_outcome_id, created_by
        FROM events
        WHERE id = $1
        FOR UPDATE
        "#,
        req.event_id as i64 
    )
    .fetch_optional(&mut *tx)
    .await {
        Ok(Some(event)) => event,
        Ok(None) => {
            let _ = tx.rollback().await;
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "Event not found"
            }));
        }
        Err(_) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to load event"
            }));
        }
    };

    if event.status == "RESOLVED" || event.winning_outcome_id.is_some() {
        let _ =  tx.rollback().await;
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "cannot delete a resolved event"
        }));
    }

    if let Err(_) = sqlx::query!(
        r#"
        DELETE FROM markets
        WHERE outcome_id IN (
            SELECT id FROM outcomes WHERE event_id = $1
        )
        "#,
        event.id 
    )
    .execute(&mut *tx)
    .await {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to delete markets"
        }));
    }

    if let Err(_) = sqlx::query!(
        r#"
        DELETE FROM outcomes
        WHERE event_id = $1
        "#,
        event.id 
    )
    .execute(&mut *tx)
    .await {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to delete outcomes"
        }));
    }

    if let Err(_) = sqlx::query!(
        r#"
        DELETE FROM events
        WHERE id = $1
        "#,
        event.id 
    )
    .execute(&mut *tx)
    .await {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to delete event"
        }));
    }

    match tx.commit().await {
        Ok(_) => {
            if let Some(redis_manager) = RedisManager::global() {
                let cache_key = format!("event:{}", event.id);
                if let Err(e) = redis_manager.delete("events:all").await {
                    warn!("Failed to invalidate events:all cache after deletion: {:?}", e);
                }
                if let Err(e) = redis_manager.delete(&cache_key).await {
                    warn!("Failed to invalidate event cache after deletion: {:?}", e);
                }
            }

            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Event deleted succesfully",
                "event_id": event.id 
            }))
        }
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to finalize deletion"
        }))
    }
}