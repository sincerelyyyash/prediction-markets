use crate::utils::jwt::extract_user_id;
use crate::utils::redis_stream::send_request_and_wait;
use crate::utils::responses::build_json_from_redis_response;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use redis_client::RedisRequest;
use serde_json::json;
use uuid::Uuid;

#[get("/positions")]
pub async fn get_positions(req: HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = Uuid::new_v4().to_string();
    let positions_request = RedisRequest::new(
        "db_worker",
        "get_positions_by_user",
        "Get user positions",
        json!({
            "user_id": user_id as u64,
        }),
    );

    match send_request_and_wait(request_id, positions_request, 10).await {
        Ok(response) => build_json_from_redis_response(&response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch positions",
            "error": e
        })),
    }
}

#[get("/positions/history")]
pub async fn get_positions_history(req: HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = Uuid::new_v4().to_string();
    let positions_request = RedisRequest::new(
        "db_worker",
        "get_positions_by_user",
        "Get user positions history",
        json!({
            "user_id": user_id as u64,
        }),
    );

    match send_request_and_wait(request_id, positions_request, 10).await {
        Ok(response) => build_json_from_redis_response(&response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch positions history",
            "error": e
        })),
    }
}

#[get("/positions/{market_id}")]
pub async fn get_position_by_market(req: HttpRequest, path: web::Path<u64>) -> impl Responder {
    let market_id = path.into_inner();

    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = Uuid::new_v4().to_string();
    let position_request = RedisRequest::new(
        "db_worker",
        "get_position_by_user_and_market",
        "Get user position for market",
        json!({
            "user_id": user_id as u64,
            "market_id": market_id,
        }),
    );

    match send_request_and_wait(request_id, position_request, 10).await {
        Ok(response) => build_json_from_redis_response(&response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch position",
            "error": e
        })),
    }
}

#[get("/positions/portfolio")]
pub async fn get_portfolio(req: HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = Uuid::new_v4().to_string();
    let portfolio_request = RedisRequest::new(
        "engine",
        "get-portfolio",
        "Get user portfolio",
        json!({
            "user_id": user_id as u64,
        }),
    );

    match send_request_and_wait(request_id, portfolio_request, 10).await {
        Ok(response) => build_json_from_redis_response(&response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch portfolio",
            "error": e
        })),
    }
}
