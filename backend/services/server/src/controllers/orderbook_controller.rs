use crate::utils::redis_stream::send_request_and_wait;
use actix_web::{get, web, HttpResponse, Responder};
use redis_client::RedisRequest;
use serde_json::json;
use uuid::Uuid;

#[get("/orderbooks/market/{market_id}")]
pub async fn get_orderbook_by_market(path: web::Path<u64>) -> impl Responder {
    let market_id = path.into_inner();
    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "engine",
        "get-orderbook",
        "Get orderbook snapshot",
        json!({ "market_id": market_id }),
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
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
            "message": "Failed to fetch orderbook",
            "error": e
        })),
    }
}

#[get("/orderbooks/event/{event_id}")]
pub async fn get_orderbooks_by_event(path: web::Path<u64>) -> impl Responder {
    let event_id = path.into_inner();
    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "engine",
        "get-orderbook-by-event",
        "Get event orderbooks",
        json!({ "event_id": event_id }),
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
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
            "message": "Failed to fetch event orderbooks",
            "error": e
        })),
    }
}

#[get("/orderbooks/outcome/{outcome_id}")]
pub async fn get_orderbooks_by_outcome(path: web::Path<u64>) -> impl Responder {
    let outcome_id = path.into_inner();
    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "engine",
        "get-orderbook-by-outcome",
        "Get outcome orderbooks",
        json!({ "outcome_id": outcome_id }),
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
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
            "message": "Failed to fetch outcome orderbooks",
            "error": e
        })),
    }
}
