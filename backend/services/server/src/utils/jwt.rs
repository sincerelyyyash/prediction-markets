use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i64,
    exp: usize,
}

pub fn create_jwt(id: &i64, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: id.to_owned(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<i64, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims.sub)
}

pub fn extract_user_id(req: &HttpRequest) -> Result<i64, HttpResponse> {
    let auth_header = req.headers().get("Authorization").ok_or_else(|| {
        HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Missing Authorization header"
        }))
    })?;

    let token_str = auth_header.to_str().map_err(|_| {
        HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid Authorization header"
        }))
    })?;

    let token = token_str.strip_prefix("Bearer ").ok_or_else(|| {
        HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid token format"
        }))
    })?;

    let jwt_secret = env::var("JWT_SECRET").map_err(|_| {
        HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "JWT secret not configured"
        }))
    })?;

    verify_jwt(token, &jwt_secret).map_err(|_| {
        HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid or expired token"
        }))
    })
}

pub fn get_user_id_from_extensions(req: &HttpRequest) -> Result<i64, HttpResponse> {
    req.extensions().get::<i64>().copied().ok_or_else(|| {
        HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "User ID not found in request. Ensure auth middleware is applied."
        }))
    })
}
