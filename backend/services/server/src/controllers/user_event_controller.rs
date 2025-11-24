use crate::types::event_types::EventSearchQueryRequest;
use crate::utils::redis_stream::send_request_and_wait;
use actix_web::{get, web, HttpResponse, Responder};
use log::warn;
use redis_client::{RedisManager, RedisRequest};
use serde_json::json;
use uuid::Uuid;

#[get("/events")]
pub async fn get_all_events() -> impl Responder {
    const CACHE_KEY: &str = "events:all";

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

    let request_id = Uuid::new_v4().to_string();
    let read_request =
        RedisRequest::new("db_worker", "get_all_events", "Get all events", json!({}));

    match send_request_and_wait(request_id, read_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(response.data)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch events",
            "error": e
        })),
    }
}

#[get("/events/{event_id}")]
pub async fn get_event_by_id(path: web::Path<u64>) -> impl Responder {
    let event_id = path.into_inner();
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

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new(
        "db_worker",
        "get_event_by_id",
        "Get event by ID",
        json!({
            "event_id": event_id
        }),
    );

    match send_request_and_wait(request_id, read_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                if response.status_code == 404 {
                    return HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": response.message,
                        "data": response.data
                    }));
                }
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(response.data)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch event",
            "error": e
        })),
    }
}

#[get("/events/search")]
pub async fn search_events(query: web::Query<EventSearchQueryRequest>) -> impl Responder {
    let cache_key = format!(
        "events:search:q:{}:cat:{}:status:{}",
        query.q.as_deref().unwrap_or(""),
        query.category.as_deref().unwrap_or(""),
        query.status.as_deref().unwrap_or("")
    );

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

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new(
        "db_worker",
        "search_events",
        "Search events",
        json!({
            "q": query.q,
            "category": query.category,
            "status": query.status
        }),
    );

    match send_request_and_wait(request_id, read_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                let status = actix_web::http::StatusCode::from_u16(response.status_code as u16)
                    .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                return HttpResponse::build(status).json(json!({
                    "status": if response.success { "success" } else { "error" },
                    "message": response.message,
                    "data": response.data
                }));
            }
            HttpResponse::Ok().json(response.data)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to search events",
            "error": e
        })),
    }
}
