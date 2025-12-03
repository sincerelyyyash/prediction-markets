use super::common::send_read_response;
use crate::models::UserTable;
use log::info;
use redis_client::RedisResponse;
use serde_json::Value;
use sqlx::PgPool;

pub async fn handle_get_user_by_email(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let email = data["email"]
        .as_str()
        .ok_or_else(|| "Invalid email".to_string())?;

    let user = match sqlx::query_as!(
        UserTable,
        r#"
        SELECT id, email, name, password, balance
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            let response =
                RedisResponse::new(404, false, "User not found", serde_json::json!(null));
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch user: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch user: {}", e));
        }
    };

    if user.id <= 0 {
        let error_response = RedisResponse::new(
            500,
            false,
            format!("Invalid user data: non-positive user ID {}", user.id),
            serde_json::json!(null),
        );
        send_read_response(&request_id, error_response).await?;
        return Err(format!("Invalid user ID {} from database", user.id));
    }

    let response_data = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "password": user.password,
        "balance": user.balance,
    });

    let response = RedisResponse::new(200, true, "User fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_user_by_email request: request_id={}",
        request_id
    );
    Ok(())
}

pub async fn handle_get_user_by_id(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let user_id_u64 = data["user_id"]
        .as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())?;
    
    if user_id_u64 > i64::MAX as u64 {
        return Err(format!("User ID {} exceeds i64::MAX", user_id_u64));
    }
    if user_id_u64 == 0 {
        return Err("User ID cannot be zero".to_string());
    }
    let user_id = user_id_u64 as i64;

    let user = match sqlx::query_as!(
        UserTable,
        r#"
        SELECT id, email, name, password, balance
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            let response =
                RedisResponse::new(404, false, "User not found", serde_json::json!(null));
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch user: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch user: {}", e));
        }
    };

    if user.id <= 0 {
        let error_response = RedisResponse::new(
            500,
            false,
            format!("Invalid user data: non-positive user ID {}", user.id),
            serde_json::json!(null),
        );
        send_read_response(&request_id, error_response).await?;
        return Err(format!("Invalid user ID {} from database", user.id));
    }

    let response_data = serde_json::json!({
        "status": "success",
        "message": "User fetched successfully",
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "balance": user.balance
        }
    });

    let response = RedisResponse::new(200, true, "User fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_user_by_id request: request_id={}, user_id={}",
        request_id, user_id
    );
    Ok(())
}

pub async fn handle_get_all_users(
    _data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let users = match sqlx::query_as!(
        UserTable,
        r#"
        SELECT id, email, name, password, balance
        FROM users
        ORDER BY id DESC
        "#
    )
    .fetch_all(pool)
    .await
    {
        Ok(users) => users,
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch users: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch users: {}", e));
        }
    };

    let users_json: Vec<serde_json::Value> = users
        .iter()
        .filter(|u| u.id > 0)
        .map(|u| {
            serde_json::json!({
                "id": u.id,
                "email": u.email,
                "name": u.name,
                "balance": u.balance
            })
        })
        .collect();

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Users fetched successfully",
        "users": users_json,
        "count": users.len()
    });

    let response = RedisResponse::new(200, true, "Users fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!("Processed get_all_users request: request_id={}", request_id);
    Ok(())
}
