use sqlx::PgPool;
use serde_json::Value;
use log::info;
use redis_client::RedisResponse;
use super::common::send_read_response;

pub async fn handle_get_position_by_user_and_market(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let user_id = data["user_id"].as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())? as i64;
    let market_id = data["market_id"].as_u64()
        .ok_or_else(|| "Invalid market_id".to_string())? as i64;

    let position = match sqlx::query!(
        r#"
        SELECT user_id, market_id, quantity, created_at, updated_at
        FROM positions
        WHERE user_id = $1 AND market_id = $2
        "#,
        user_id,
        market_id
    )
    .fetch_optional(pool)
    .await {
        Ok(Some(pos)) => pos,
        Ok(None) => {
            let response = RedisResponse::new(
                404,
                false,
                "Position not found",
                serde_json::json!(null),
            );
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch position: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch position: {}", e));
        }
    };

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Position fetched successfully",
        "position": {
            "user_id": position.user_id,
            "market_id": position.market_id,
            "quantity": position.quantity,
            "created_at": position.created_at,
            "updated_at": position.updated_at
        }
    });

    let response = RedisResponse::new(
        200,
        true,
        "Position fetched successfully",
        response_data,
    );

    send_read_response(&request_id, response).await?;
    info!("Processed get_position_by_user_and_market request: request_id={}, user_id={}, market_id={}", request_id, user_id, market_id);
    Ok(())
}

pub async fn handle_get_positions_by_user(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let user_id = data["user_id"].as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())? as i64;

    let positions = match sqlx::query!(
        r#"
        SELECT user_id, market_id, quantity, created_at, updated_at
        FROM positions
        WHERE user_id = $1
        ORDER BY updated_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await {
        Ok(positions) => positions,
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch positions: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch positions: {}", e));
        }
    };

    let positions_json: Vec<serde_json::Value> = positions.iter().map(|p| {
        serde_json::json!({
            "user_id": p.user_id,
            "market_id": p.market_id,
            "quantity": p.quantity,
            "created_at": p.created_at,
            "updated_at": p.updated_at
        })
    }).collect();

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Positions fetched successfully",
        "positions": positions_json,
        "count": positions.len()
    });

    let response = RedisResponse::new(
        200,
        true,
        "Positions fetched successfully",
        response_data,
    );

    send_read_response(&request_id, response).await?;
    info!("Processed get_positions_by_user request: request_id={}, user_id={}", request_id, user_id);
    Ok(())
}

