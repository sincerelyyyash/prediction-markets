use super::common::send_read_response;
use crate::models::OutcomeTable;
use log::info;
use redis_client::RedisResponse;
use serde_json::Value;
use sqlx::PgPool;

pub async fn handle_get_outcome_by_id(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let outcome_id = data["outcome_id"]
        .as_u64()
        .ok_or_else(|| "Invalid outcome_id".to_string())? as i64;

    let outcome = match sqlx::query_as!(
        OutcomeTable,
        r#"
        SELECT id, event_id, name, status, img_url
        FROM outcomes
        WHERE id = $1
        "#,
        outcome_id
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(outcome)) => outcome,
        Ok(None) => {
            let response =
                RedisResponse::new(404, false, "Outcome not found", serde_json::json!(null));
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch outcome: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch outcome: {}", e));
        }
    };

    let markets = match sqlx::query!(
        r#"
        SELECT id, outcome_id, side, last_price
        FROM markets
        WHERE outcome_id = $1
        ORDER BY side ASC
        "#,
        outcome_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(markets) => markets,
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch markets: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch markets: {}", e));
        }
    };

    let markets_json: Vec<serde_json::Value> = markets
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "outcome_id": m.outcome_id,
                "side": m.side.as_str(),
                "last_price": m.last_price
            })
        })
        .collect();

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Outcome fetched successfully",
        "outcome": {
            "id": outcome.id,
            "event_id": outcome.event_id,
            "name": outcome.name,
            "status": outcome.status,
            "img_url": outcome.img_url,
            "markets": markets_json
        }
    });

    let response = RedisResponse::new(200, true, "Outcome fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_outcome_by_id request: request_id={}",
        request_id
    );
    Ok(())
}
