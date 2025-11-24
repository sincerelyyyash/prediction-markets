use crate::types::auth_types::LoginUserInput;
use crate::utils::jwt::create_jwt;
use crate::utils::redis_stream::send_request_and_wait;
use actix_web::{post, web, HttpResponse, Responder};
use bcrypt::verify;
use redis_client::RedisRequest;
use serde_json::json;
use std::env;
use uuid::Uuid;
use validator::Validate;

#[post("admin/signin")]
pub async fn signin_admin(req: web::Json<LoginUserInput>) -> impl Responder {
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }));
    }

    let request_id = Uuid::new_v4().to_string();
    let admin_request = RedisRequest::new(
        "db_worker",
        "get_admin_by_email",
        "Get admin by email",
        json!({
            "email": req.email,
        }),
    );

    let admin_response = match send_request_and_wait(request_id, admin_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                if response.status_code == 404 {
                    return HttpResponse::Unauthorized().json(json!({
                        "status":"error",
                        "message": "Invalid email or user does not exist"
                    }));
                }
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message":"Database query failed"
                }));
            }
            response
        }
        Err(e) => {
            eprintln!("Failed to fetch admin: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message":"Database query failed"
            }));
        }
    };

    let admin_id = match admin_response.data["id"].as_i64() {
        Some(id) => id,
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Invalid admin data"
            }));
        }
    };

    let admin_password = match admin_response.data["password"].as_str() {
        Some(pwd) => pwd,
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Invalid admin data"
            }));
        }
    };

    let is_valid = verify(&req.password, admin_password).unwrap_or(false);
    if !is_valid {
        return HttpResponse::Unauthorized().json(json!({
            "status":"error",
            "message": "Incorrect password"
        }));
    }

    let jwt_token = env::var("JWT_SECRET").expect("JWT_SECRET must be present");
    let token = match create_jwt(&admin_id, &jwt_token) {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to sign in user"
            }))
        }
    };

    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Sign in successfully",
        "token": token,
    }))
}
