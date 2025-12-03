use crate::utils::jwt::extract_user_id;
use crate::utils::redis_stream::send_request_and_wait;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use redis_client::RedisRequest;
use serde_json::json;
use uuid::Uuid;

#[get("/users/{user_id}")]
pub async fn get_user_by_id(req: HttpRequest, path: web::Path<u64>) -> impl Responder {
    let user_id = path.into_inner();

    let authenticated_user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    if user_id > i64::MAX as u64 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid user ID: exceeds maximum value"
        }));
    }

    if authenticated_user_id != user_id as i64 {
        return HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Access denied"
        }));
    }

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new(
        "db_worker",
        "get_user_by_id",
        "Get user by ID",
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

            let user_id_value = response
                .data
                .get("user")
                .and_then(|u| u.get("id"))
                .cloned()
                .unwrap_or_else(|| json!(null));

            let user_id_u64 = if let Some(id) = user_id_value.as_u64() {
                id
            } else if let Some(id) = user_id_value.as_i64() {
                if id < 0 {
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Invalid user data: negative id"
                    }));
                }
                id as u64
            } else {
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Invalid user data: missing id"
                }));
            };

            let email = response
                .data
                .get("user")
                .and_then(|u| u.get("email"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = response
                .data
                .get("user")
                .and_then(|u| u.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let balance = response
                .data
                .get("user")
                .and_then(|u| u.get("balance"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            HttpResponse::Ok().json(json!({
                "id": user_id_u64,
                "email": email,
                "name": name,
                "balance": balance
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch user",
            "error": e
        })),
    }
}

#[get("/users")]
pub async fn get_all_users(req: HttpRequest) -> impl Responder {
    let _admin_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = Uuid::new_v4().to_string();
    let read_request = RedisRequest::new("db_worker", "get_all_users", "Get all users", json!({}));

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
            "message": "Failed to fetch users",
            "error": e
        })),
    }
}
