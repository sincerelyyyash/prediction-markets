use sqlx::PgPool;
use serde_json::Value;
use log::info;
use redis_client::RedisResponse;
use crate::models::AdminTable;
use super::common::send_read_response;

pub async fn handle_get_admin_by_email(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let email = data["email"].as_str()
        .ok_or_else(|| "Invalid email".to_string())?;

    let admin = match sqlx::query_as!(
        AdminTable,
        r#"
        SELECT id, email, name, password
        FROM admins
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await {
        Ok(Some(admin)) => admin,
        Ok(None) => {
            let response = RedisResponse::new(
                404,
                false,
                "Admin not found",
                serde_json::json!(null),
            );
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch admin: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch admin: {}", e));
        }
    };

    let response_data = serde_json::json!({
        "id": admin.id,
        "email": admin.email,
        "name": admin.name,
        "password": admin.password,
    });

    let response = RedisResponse::new(
        200,
        true,
        "Admin fetched successfully",
        response_data,
    );

    send_read_response(&request_id, response).await?;
    info!("Processed get_admin_by_email request: request_id={}", request_id);
    Ok(())
}

pub async fn handle_get_admin_by_id(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let id = data["id"].as_i64()
        .ok_or_else(|| "Invalid id".to_string())?;

    let admin = match sqlx::query_as!(
        AdminTable,
        r#"
        SELECT id, email, name, password
        FROM admins
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await {
        Ok(Some(admin)) => admin,
        Ok(None) => {
            let response = RedisResponse::new(
                404,
                false,
                "Admin not found",
                serde_json::json!(null),
            );
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch admin: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch admin: {}", e));
        }
    };

    let response_data = serde_json::json!({
        "id": admin.id,
        "email": admin.email,
        "name": admin.name,
    });

    let response = RedisResponse::new(
        200,
        true,
        "Admin fetched successfully",
        response_data,
    );

    send_read_response(&request_id, response).await?;
    info!("Processed get_admin_by_id request: request_id={}", request_id);
    Ok(())
}

