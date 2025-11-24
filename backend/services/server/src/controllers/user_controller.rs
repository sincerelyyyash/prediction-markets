use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use crate::types::auth_types::{SignUpUserInput, LoginUserInput};
use crate::utils::jwt::{create_jwt, extract_user_id};
use crate::utils::redis_stream::send_request_and_wait;
use crate::services::db_event_publisher::publish_db_event;
use redis_client::RedisRequest;
use engine::types::db_event_types::{DbEvent, UserCreatedEvent};
use bcrypt::{hash, DEFAULT_COST, verify};
use serde_json::json;
use validator::Validate;
use uuid::Uuid;
use chrono::Utc;
use std::env;

#[post("/user/signup")]
pub async fn signup_user(req: web::Json<SignUpUserInput>) -> impl Responder {
    if let Err(e) = req.validate(){
        return HttpResponse::BadRequest().json(json!({
            "status":"error",
            "message": e.to_string()
        }));
    }

    let hashed_password = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status":"error",
                "message":"Failed to sign up user"
            }))
        }
    };

    let user_id = Uuid::new_v4().as_u128() as u64;

    let event = DbEvent::UserCreated(UserCreatedEvent {
        user_id,
        email: req.email.clone(),
        name: req.name.clone(),
        password: hashed_password,
        balance: 0,
        timestamp: Utc::now(),
    });

    if let Err(e) = publish_db_event(event).await {
        eprintln!("Failed to publish user creation event: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to create user"
        }));
    }

    let user_data = json!({
        "id": user_id,
        "name": req.name,
        "email": req.email,
        "balance": 0,
    });

    let request_id = Uuid::new_v4().to_string();
    let redis_request = RedisRequest::new(
        "engine",
        "create-user",
        "Create user in engine",
        user_data,
    );

    match send_request_and_wait(request_id, redis_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "User creation queued but failed to sync with engine",
                    "user_id": user_id
                }));
            }
        }
        Err(e) => {
            eprintln!("Failed to sync user with engine: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "User creation queued but failed to sync with engine",
                "user_id": user_id
            }));
        }
    }

    HttpResponse::Ok().json(json!({
        "status":"success",
        "message":"User registered Succesfully",
        "user_id": user_id 
    }))
}

#[post("/user/signin")]
pub async fn signin_user(req: web::Json<LoginUserInput>) -> impl Responder {
    if let Err(e) = req.validate(){
        return HttpResponse::BadRequest().json(json!({
            "status":"error",
            "message": e.to_string()
        }))
    }

    let request_id = Uuid::new_v4().to_string();
    let user_request = RedisRequest::new(
        "db_worker",
        "get_user_by_email",
        "Get user by email",
        json!({
            "email": req.email,
        }),
    );

    let user_response = match send_request_and_wait(request_id, user_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                if response.status_code == 404 {
                    return HttpResponse::Unauthorized().json(json!({
                        "status":"error",
                        "message": "Invalid email or User does not exists"
                    }));
                }
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to fetch user"
                }));
            }
            response
        }
        Err(e) => {
            eprintln!("Failed to fetch user: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database query failed"
            }));
        }
    };

    let user_id_i64 = match user_response.data["id"].as_i64() {
        Some(id) => id,
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Invalid user data"
            }));
        }
    };
    
    let user_id_u64 = if user_id_i64 < 0 {
        user_id_i64 as u64
    } else {
        user_id_i64 as u64
    };
    
    let user_id = user_id_u64 as i64;

    let user_password = match user_response.data["password"].as_str() {
        Some(pwd) => pwd,
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Invalid user data"
            }));
        }
    };

    let user_balance = user_response.data["balance"].as_i64().unwrap_or(0);
    let user_email = user_response.data["email"].as_str().unwrap_or("").to_string();
    let user_name = user_response.data["name"].as_str().unwrap_or("").to_string();

    let is_valid = verify(&req.password, user_password).unwrap_or(false);
    if !is_valid {
        return HttpResponse::Unauthorized().json(json!({
            "status":"error",
            "message": "Incorrect password"
        }));
    }

    let jwt_token = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = match create_jwt(&user_id, &jwt_token){
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status":"error",
                "message":"Failed to sign in user"
            }))
        }
    };

    let balance_request_id = Uuid::new_v4().to_string();
    let balance_request = RedisRequest::new(
        "engine",
        "get-balance",
        "Get user balance from engine",
        json!({
            "user_id": user_id_u64,
        }),
    );

    let balance = match send_request_and_wait(balance_request_id, balance_request, 10).await {
        Ok(response) => {
            if response.status_code >= 400 {
                user_balance
            } else {
                response.data.get("balance").and_then(|v| v.as_i64()).unwrap_or(user_balance)
            }
        }
        Err(_) => user_balance,
    };

    HttpResponse::Ok().json(json!({
        "status":"success",
        "message":"Sign in successfully",
        "token": token,
        "user":{
            "id": user_id_u64,
            "email": user_email,
            "name": user_name,
            "balance": balance
        }
    }))
}

#[get("/user/balance")]
pub async fn get_balance(req: HttpRequest) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let request_id = Uuid::new_v4().to_string();
    let balance_request = RedisRequest::new(
        "engine",
        "get-balance",
        "Get user balance",
        json!({
            "user_id": user_id as u64,
        }),
    );

    match send_request_and_wait(request_id, balance_request, 10).await {
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
                "balance": response.data.get("balance").and_then(|v| v.as_i64()).unwrap_or(0)
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to get balance",
                "error": e
            }))
        }
    }
}

#[post("/user/onramp")]
pub async fn onramp(req: HttpRequest, body: web::Json<serde_json::Value>) -> impl Responder {
    let user_id = match extract_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let amount = body["amount"].as_i64()
        .ok_or_else(|| HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Missing or invalid amount"
        })));

    let amount = match amount {
        Ok(a) => a,
        Err(resp) => return resp,
    };

    if amount <= 0 {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Amount must be greater than 0"
        }));
    }

    let request_id = Uuid::new_v4().to_string();
    let onramp_request = RedisRequest::new(
        "engine",
        "onramp",
        "Add balance to user",
        json!({
            "user_id": user_id as u64,
            "amount": amount,
        }),
    );

    match send_request_and_wait(request_id, onramp_request, 10).await {
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
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to add balance",
                "error": e
            }))
        }
    }
}
