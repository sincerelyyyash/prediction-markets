use actix_web::{get, web, HttpResponse, Responder};
use crate::models::market_model::{EventTable, OutcomeTable, MarketTable};
use serde_json::json;
use sqlx::PgPool;
use redis_client::RedisManager;
use log::warn;

#[get("/events")]
pub async fn get_all_events(db_pool: web::Data<PgPool>) -> impl Responder {
    const CACHE_KEY: &str = "events:all";
    const CACHE_TTL: i64 = 86400; // 24 hours

    if let Some(redis_manager) = RedisManager::global() {
        match redis_manager.get(CACHE_KEY).await {
            Ok(Some(cached_data)) => {
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&cached_data) {
                    return HttpResponse::Ok().json(response);
                }
            }
            Ok(None) => {
                warn!("Redis cache miss for {}", CACHE_KEY);
            }
            Err(e) => {
                warn!("Redis cache read error: {:?}", e);
            }
        }
    }
    let events = match sqlx::query_as!(
        EventTable,
        r#"
        SELECT id, slug, title, description, category, status, resolved_at,
            winning_outcome_id, created_by
        FROM events
        ORDER BY id DESC
        "#
    )
    .fetch_all(db_pool.get_ref())
    .await {
        Ok(events) => events,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to fetch events"
            }));
        }
    };

    let mut events_with_outcomes = Vec::new();

    for event in &events {
        let outcomes = match sqlx::query_as!(
            OutcomeTable,
            r#"
            SELECT id, event_id, name, status
            FROM outcomes
            WHERE event_id = $1
            ORDER BY id ASC
            "#,
            event.id
        )
        .fetch_all(db_pool.get_ref())
        .await {
            Ok(outcomes) => outcomes,
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("Failed to fetch outcomes for event {}", event.id)
                }));
            }
        };

        events_with_outcomes.push(json!({
            "id": event.id,
            "slug": event.slug,
            "title": event.title,
            "description": event.description,
            "category": event.category,
            "status": event.status,
            "resolved_at": event.resolved_at,
            "winning_outcome_id": event.winning_outcome_id,
            "created_by": event.created_by,
            "outcomes": outcomes.iter().map(|o| json!({
                "id": o.id,
                "event_id": o.event_id,
                "name": o.name,
                "status": o.status
            })).collect::<Vec<_>>()
        }));
    }

    let response_data = json!({
        "status": "success",
        "message": "Events fetched successfully",
        "events": events_with_outcomes,
        "count": events.len()
    });

    if let Some(redis_manager) = RedisManager::global() {
        if let Ok(response_json) = serde_json::to_string(&response_data) {
            if let Err(e) = redis_manager.set_with_ttl(CACHE_KEY, &response_json, CACHE_TTL).await {
                warn!("Failed to cache events data: {:?}", e);
            }
        }
    }

    HttpResponse::Ok().json(response_data)
}

#[get("/events/{event_id}")]
pub async fn get_event_by_id(db_pool: web::Data<PgPool>, path: web::Path<u64>) -> impl Responder {
    let event_id = path.into_inner() as i64;
    const CACHE_TTL: i64 = 86400; // 24 hours
    let cache_key = format!("event:{}", event_id);

    if let Some(redis_manager) = RedisManager::global() {
        match redis_manager.get(&cache_key).await {
            Ok(Some(cached_data)) => {
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&cached_data) {
                    return HttpResponse::Ok().json(response);
                }
            }
            Ok(None) => {
                warn!("Redis cache miss for {}", cache_key);
            }
            Err(e) => {
                warn!("Redis cache read error: {:?}", e);
            }
        }
    }
    let event = match sqlx::query_as!(
        EventTable,
        r#"
        SELECT id, slug, title, description, category, status,
                resolved_at, winning_outcome_id, created_by
        FROM events
        WHERE id = $1
        "#,
        event_id 
    )
    .fetch_optional(db_pool.get_ref())
    .await {
        Ok(Some(event)) => event,
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "event not found"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to load event"
            }));
        }
    };

    let outcomes = match sqlx::query_as!(
        OutcomeTable,
        r#"
        SELECT id, event_id, name, status
        FROM outcomes
        WHERE event_id = $1
        ORDER BY id ASC
        "#,
        event_id
    )
    .fetch_all(db_pool.get_ref())
    .await {
        Ok(outcomes) => outcomes,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to load outcomes"
            }))
        }
    };

    let outcome_ids: Vec<i64> = outcomes.iter().map(|o| o.id).collect();

    let markets = if outcome_ids.is_empty() {
        Vec::new()
    } else {
        match sqlx::query_as!(
            MarketTable,
            r#"
            SELECT id, outcome_id, side
            FROM markets
            WHERE outcome_id = ANY($1)
            ORDER BY outcome_id ASC, side ASC
            "#,
            &outcome_ids[..]
        )
        .fetch_all(db_pool.get_ref())
        .await {
            Ok(markets) => markets,
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "failed to load markets"
                }));
            }
        }
    };

    let response_data = json!({
        "status": "success",
        "message": "Event fetched successfully",
        "event": {
            "id": event.id,
            "slug": event.slug,
            "title": event.title,
            "description": event.description,
            "category": event.category,
            "status": event.status,
            "resolved_at": event.resolved_at,
            "winning_outcome_id": event.winning_outcome_id,
            "created_by": event.created_by
        },
        "outcomes": outcomes.iter().map(|o| {
            let outcome_markets: Vec<_> = markets
                .iter()
                .filter(|m| m.outcome_id == o.id)
                .map(|m| json!({
                    "id": m.id,
                    "outcome_id": m.outcome_id,
                    "side": m.side
                }))
                .collect();
            
            json!({
                "id": o.id,
                "event_id": o.event_id,
                "name": o.name,
                "status": o.status,
                "markets": outcome_markets
            })
        }).collect::<Vec<_>>()
    });

    if let Some(redis_manager) = RedisManager::global() {
        if let Ok(response_json) = serde_json::to_string(&response_data) {
            if let Err(e) = redis_manager.set_with_ttl(&cache_key, &response_json, CACHE_TTL).await {
                warn!("Failed to cache event data: {:?}", e);
            }
        }
    }

    HttpResponse::Ok().json(response_data)
}
