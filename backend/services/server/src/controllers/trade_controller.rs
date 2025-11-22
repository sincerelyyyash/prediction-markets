use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use crate::utils::jwt::extract_user_id;
use crate::utils::redis_stream::send_request_and_wait;
use redis_client::RedisRequest;
use serde_json::json;
use uuid::Uuid;

#[get("/trades")]
pub async fn get_trades(req: HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new(
        "db_worker",
        "get_trades_by_user",
        "Get trades for user",
        json!({
            "user_id": user_id as u64,
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
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to fetch trades",
                "error": e
            }))
        }
    }
}

#[get("/trades/{trade_id}")]
pub async fn get_trade_by_id(req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let trade_id = path.into_inner();

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new(
        "db_worker",
        "get_trade_by_id",
        "Get trade by ID",
        json!({
            "trade_id": trade_id,
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
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to fetch trade",
                "error": e
            }))
        }
    }
}

#[get("/trades/market/{market_id}")]
pub async fn get_trades_by_market(req: HttpRequest, path: web::Path<u64>) -> impl Responder {
    let market_id = path.into_inner();

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new(
        "db_worker",
        "get_trades_by_market",
        "Get trades for market",
        json!({
            "market_id": market_id,
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
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to fetch trades",
                "error": e
            }))
        }
    }
}

#[get("/trades/user/{user_id}")]
pub async fn get_trades_by_user_id(req: HttpRequest, path: web::Path<u64>) -> impl Responder {
    let user_id = path.into_inner();

    let _admin_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new(
        "db_worker",
        "get_trades_by_user",
        "Get trades for user",
        json!({
            "user_id": user_id,
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
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to fetch trades",
                "error": e
            }))
        }
    }
}

