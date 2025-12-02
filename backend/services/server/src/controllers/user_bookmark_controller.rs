use crate::utils::jwt::extract_user_id;
use crate::utils::redis_stream::send_request_and_wait;
use crate::utils::responses::build_json_from_redis_response;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use redis_client::RedisRequest;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct BookmarkPayload {
    pub market_id: u64,
}

#[derive(Deserialize, Default)]
pub struct ForYouQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[post("/user/bookmarks")]
pub async fn add_market_bookmark(
    req: HttpRequest,
    payload: web::Json<BookmarkPayload>,
) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    if payload.market_id == 0 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "market_id must be greater than 0"
        }));
    }

    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "db_worker",
        "add_market_bookmark",
        "Add market bookmark",
        json!({
            "user_id": user_id as u64,
            "market_id": payload.market_id,
        }),
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => build_json_from_redis_response(&response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to add bookmark",
            "error": e
        })),
    }
}

#[delete("/user/bookmarks/{market_id}")]
pub async fn remove_market_bookmark(req: HttpRequest, path: web::Path<u64>) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let market_id = path.into_inner();

    if market_id == 0 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "market_id must be greater than 0"
        }));
    }

    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "db_worker",
        "remove_market_bookmark",
        "Remove market bookmark",
        json!({
            "user_id": user_id as u64,
            "market_id": market_id,
        }),
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => build_json_from_redis_response(&response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to remove bookmark",
            "error": e
        })),
    }
}

#[get("/user/bookmarks")]
pub async fn get_user_bookmarks(req: HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "db_worker",
        "get_user_bookmarks",
        "Fetch user bookmarks",
        json!({
            "user_id": user_id as u64,
        }),
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => build_json_from_redis_response(&response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch bookmarks",
            "error": e
        })),
    }
}

#[get("/user/markets/for-you")]
pub async fn get_for_you_markets(
    req: HttpRequest,
    query: web::Query<ForYouQuery>,
) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "db_worker",
        "get_for_you_markets",
        "Fetch For You markets",
        json!({
            "user_id": user_id as u64,
            "limit": limit,
            "offset": offset,
        }),
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => build_json_from_redis_response(&response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch For You markets",
            "error": e
        })),
    }
}
