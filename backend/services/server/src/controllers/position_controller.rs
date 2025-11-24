use crate::utils::jwt::extract_user_id;
use crate::utils::redis_stream::send_request_and_wait;
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
        Ok(response) => {
            if response.status_code >= 400 {
                if response.status_code == 404 {
                    return HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": response.message,
                        "data": response.data
                    }));
                } else if response.status_code == 400 {
                    return HttpResponse::BadRequest().json(json!({
                        "status": "error",
                        "message": response.message,
                        "data": response.data
                    }));
                } else {
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": response.message,
                        "data": response.data
                    }));
                }
            }
            HttpResponse::Ok().json(json!({
                "status": "success",
                "message": response.message,
                "data": response.data
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch portfolio",
            "error": e
        })),
    }
}
