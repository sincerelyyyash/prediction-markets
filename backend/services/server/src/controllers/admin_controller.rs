use actix_web::{post, web, HttpResponse, Responder};
use crate::types::auth_types::{LoginUserInput};
use crate::types::admin_types::CreateEventAdminRequest;
use crate::models::admin_model::Admin;
use crate::models::market_model::{EventTable, OutcomeTable, MarketTable};
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


#[post ("admin/create-event")]
pub async fn create_event(db_pool: web::Data<PgPool>, req: web::Json<CreateEventAdminRequest>) -> impl Responder {
    if let Err(e) = req.0.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": e.to_string()
        }))
    }

    let mut tx  = match db_pool.begin().await {
        Ok(t) => t,
        Err(e)=> {
            return HttpResponse::InternalServerError().json(json!({
                "status":"error",
                "message":"Failed to start the database transaction"
            }));
        }
    };

    let event_res = sqlx::query_as!(
        EventTable,
        r#"
        INSERT INTO events (slug, title, description, category, status, resolved_at, created_by)
        VALUES($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, slug, title, description, category, status, resolved_at, winning_outcome_id, created_by
        "#,
        req.slug,
        req.title,
        req.description,
        req.category,
        req.status,
        req.resolved_at,
        req.created_by as i64
    )
    .fetch_one(&mut *tx)
    .await;

    let event = match event_res {
        Ok(e) => e,
        Err(e) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "status":"error",
                "message":"failed to create event"
            }));
        }
    };

    let mut created_outcomes = Vec::new();

    for outcome_input in &req.outcomes {
    
    let outcome_result = sqlx::query_as!(
        OutcomeTable,
        r#"
        INSERT INTO outcomes (event_id, name, status)
        VALUES($1, $2, $3)
        RETURNING id, event_id, name, status
        "#,
        event.id,
        outcome_input.name,
        outcome_input.status
    )
    .fetch_one(&mut *tx)
    .await;

    let outcome = match outcome_result {
        Ok(o) => o,
        Err(e) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("failed to create outcome: {}", outcome_input.name)
            }));
        }
    };

    let yes_market_result = sqlx::query_as!(
        MarketTable,
        r#"
        INSERT INTO markets (outcome_id, side)
        VALUES ($1, $2)
        RETURNING id, outcome_id, side
        "#,
        outcome.id,
        "YES"
    )
    .fetch_one(&mut *tx)
    .await;

    if let Err(e) = yes_market_result {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("failed to create YES market for outcome: {}", outcome.name)
        }));
    }

    let no_market_result = sqlx::query_as!(
        MarketTable,
        r#"
        INSERT INTO markets (outcome_id, side)
        VALUES ($1, $2)
        RETURNING id, outcome_id, side
        "#,
        outcome.id,
        "NO"
    )
    .fetch_one(&mut *tx)
    .await;

    if let Err(e) = no_market_result {
        let _ = tx.rollback().await;
        return HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("failed to create NO market for outcome: {}", outcome.name)
        }));
    }

        created_outcomes.push(outcome);
    }

    match tx.commit().await {
        Ok(_) => {
            HttpResponse::Created().json(json!({
                "status":"success",
                "message": "Event created successfully",
                "event": {
                    "id":event.id,
                    "slug": event.slug,
                    "title": event.title,
                    "description": event.description,
                    "category":event.category,
                "status": event.status,
                    "resolved_at": event.resolved_at,
                    "winning_outcome_id": event.winning_outcome_id,
                    "created_by": event.created_by
                },
                "outcomes": created_outcomes.iter().map(|o| json!({
                    "id": o.id,
                    "event_id": o.event_id,
                    "name": o.name,
                    "status": o.status
                })).collect::<Vec<_>>()
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status":"Error",
                "message": "Failed to create event"
            }))
        }
    }
}