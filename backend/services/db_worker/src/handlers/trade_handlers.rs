use super::common::send_read_response;
use log::info;
use redis_client::RedisResponse;
use serde_json::Value;
use sqlx::PgPool;

pub async fn handle_get_trades_by_user(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let user_id = data["user_id"]
        .as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())? as i64;

    let trades = match sqlx::query!(
        r#"
        SELECT trade_id, market_id, taker_order_id, maker_order_id, taker_user_id, 
               maker_user_id, price, quantity, taker_side, executed_at
        FROM trades
        WHERE taker_user_id = $1 OR maker_user_id = $1
        ORDER BY executed_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(trades) => trades,
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch trades: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch trades: {}", e));
        }
    };

    let trades_json: Vec<serde_json::Value> = trades
        .iter()
        .map(|t| {
            serde_json::json!({
                "trade_id": t.trade_id,
                "market_id": t.market_id,
                "taker_order_id": t.taker_order_id,
                "maker_order_id": t.maker_order_id,
                "taker_user_id": t.taker_user_id,
                "maker_user_id": t.maker_user_id,
                "price": t.price,
                "quantity": t.quantity,
                "taker_side": t.taker_side,
                "executed_at": t.executed_at
            })
        })
        .collect();

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Trades fetched successfully",
        "trades": trades_json,
        "count": trades.len()
    });

    let response = RedisResponse::new(200, true, "Trades fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_trades_by_user request: request_id={}, user_id={}",
        request_id, user_id
    );
    Ok(())
}

pub async fn handle_get_trade_by_id(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let trade_id = data["trade_id"]
        .as_str()
        .ok_or_else(|| "Invalid trade_id".to_string())?;

    let trade = match sqlx::query!(
        r#"
        SELECT trade_id, market_id, taker_order_id, maker_order_id, taker_user_id, 
               maker_user_id, price, quantity, taker_side, executed_at
        FROM trades
        WHERE trade_id = $1
        "#,
        trade_id
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(trade)) => trade,
        Ok(None) => {
            let response =
                RedisResponse::new(404, false, "Trade not found", serde_json::json!(null));
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch trade: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch trade: {}", e));
        }
    };

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Trade fetched successfully",
        "trade": {
            "trade_id": trade.trade_id,
            "market_id": trade.market_id,
            "taker_order_id": trade.taker_order_id,
            "maker_order_id": trade.maker_order_id,
            "taker_user_id": trade.taker_user_id,
            "maker_user_id": trade.maker_user_id,
            "price": trade.price,
            "quantity": trade.quantity,
            "taker_side": trade.taker_side,
            "executed_at": trade.executed_at
        }
    });

    let response = RedisResponse::new(200, true, "Trade fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_trade_by_id request: request_id={}, trade_id={}",
        request_id, trade_id
    );
    Ok(())
}

pub async fn handle_get_trades_by_market(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let market_id = data["market_id"]
        .as_u64()
        .ok_or_else(|| "Invalid market_id".to_string())? as i64;

    let trades = match sqlx::query!(
        r#"
        SELECT trade_id, market_id, taker_order_id, maker_order_id, taker_user_id, 
               maker_user_id, price, quantity, taker_side, executed_at
        FROM trades
        WHERE market_id = $1
        ORDER BY executed_at DESC
        "#,
        market_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(trades) => trades,
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch trades: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch trades: {}", e));
        }
    };

    let trades_json: Vec<serde_json::Value> = trades
        .iter()
        .map(|t| {
            serde_json::json!({
                "trade_id": t.trade_id,
                "market_id": t.market_id,
                "taker_order_id": t.taker_order_id,
                "maker_order_id": t.maker_order_id,
                "taker_user_id": t.taker_user_id,
                "maker_user_id": t.maker_user_id,
                "price": t.price,
                "quantity": t.quantity,
                "taker_side": t.taker_side,
                "executed_at": t.executed_at
            })
        })
        .collect();

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Trades fetched successfully",
        "trades": trades_json,
        "count": trades.len()
    });

    let response = RedisResponse::new(200, true, "Trades fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_trades_by_market request: request_id={}, market_id={}",
        request_id, market_id
    );
    Ok(())
}
