use actix_web::{post, web, HttpResponse, Responder};
use crate::types::auth_types::{LoginUserInput};
use crate::models::admin_model::Admin;
use crate::utils::jwt::{create_jwt};
use bcrypt::{verify};
use serde_json::json;
use validator::Validate;
use sqlx::PgPool;
use std::env;

#[post ("admin/signin")]
pub async fn signin_admin(db_pool: web::Data<PgPool>, req: web::Json<LoginUserInput>) -> impl Responder {
    if let Err(e) = req.validate(){
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }))
    }

    let result = sqlx::query_as!(
        Admin,
        r#"
        SELECT id, name, email, password FROM admins WHERE email =$1
        "#,
        req.email,
    )
    .fetch_optional(db_pool.get_ref())
    .await;

    let existing_admin = match result {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(json!({
                "status":"error",
                "message": "Invalid email or user does not exist"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message":"Database query failed"
            }));
        }
    };

    let is_valid = verify(&req.password, &existing_admin.password).unwrap_or(false);
    if !is_valid {
        return HttpResponse::Unauthorized().json(json!({
            "status":"error",
            "message": "Incorrect password"
        }));
    }

    let jwt_token = env::var("JWT_SECRET").expect("JWT_SECRET must be present");
    let token = match create_jwt(&existing_admin.id, &jwt_token){
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