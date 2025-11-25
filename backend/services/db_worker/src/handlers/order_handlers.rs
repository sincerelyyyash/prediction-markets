use super::common::send_read_response;
use log::info;
use redis_client::RedisResponse;
use serde_json::Value;
use sqlx::PgPool;

pub async fn handle_get_order_by_id(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let order_id = data["order_id"]
        .as_u64()
        .ok_or_else(|| "Invalid order_id".to_string())? as i64;
    let user_id = data
        .get("user_id")
        .and_then(|v| v.as_u64())
        .map(|v| v as i64);

    let order = match sqlx::query!(
        r#"
        SELECT order_id, user_id, market_id, side, price, original_qty, remaining_qty, 
               filled_qty, status, created_at, updated_at, cancelled_at, filled_at
        FROM orders
        WHERE order_id = $1
        "#,
        order_id
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(order)) => order,
        Ok(None) => {
            let response =
                RedisResponse::new(404, false, "Order not found", serde_json::json!(null));
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch order: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch order: {}", e));
        }
    };

    if let Some(uid) = user_id {
        if order.user_id != uid {
            let response = RedisResponse::new(403, false, "Access denied", serde_json::json!(null));
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
    }

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Order fetched successfully",
        "order": {
            "order_id": order.order_id,
            "user_id": order.user_id,
            "market_id": order.market_id,
            "side": order.side,
            "price": order.price,
            "original_qty": order.original_qty,
            "remaining_qty": order.remaining_qty,
            "filled_qty": order.filled_qty,
            "status": order.status,
            "created_at": order.created_at,
            "updated_at": order.updated_at,
            "cancelled_at": order.cancelled_at,
            "filled_at": order.filled_at
        }
    });

    let response = RedisResponse::new(200, true, "Order fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_order_by_id request: request_id={}, order_id={}",
        request_id, order_id
    );
    Ok(())
}

pub async fn handle_get_orders_by_user(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let user_id = data["user_id"]
        .as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())? as i64;

    let orders = match sqlx::query!(
        r#"
        SELECT order_id, user_id, market_id, side, price, original_qty, remaining_qty, 
               filled_qty, status, created_at, updated_at, cancelled_at, filled_at
        FROM orders
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(orders) => orders,
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch orders: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch orders: {}", e));
        }
    };

    let orders_json: Vec<serde_json::Value> = orders
        .iter()
        .map(|o| {
            serde_json::json!({
                "order_id": o.order_id,
                "user_id": o.user_id,
                "market_id": o.market_id,
                "side": o.side,
                "price": o.price,
                "original_qty": o.original_qty,
                "remaining_qty": o.remaining_qty,
                "filled_qty": o.filled_qty,
                "status": o.status,
                "created_at": o.created_at,
                "updated_at": o.updated_at,
                "cancelled_at": o.cancelled_at,
                "filled_at": o.filled_at
            })
        })
        .collect();

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Orders fetched successfully",
        "orders": orders_json,
        "count": orders.len()
    });

    let response = RedisResponse::new(200, true, "Orders fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_orders_by_user request: request_id={}, user_id={}",
        request_id, user_id
    );
    Ok(())
}

pub async fn handle_get_orders_by_market(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let market_id = data["market_id"]
        .as_u64()
        .ok_or_else(|| "Invalid market_id".to_string())? as i64;

    let orders = match sqlx::query!(
        r#"
        SELECT order_id, user_id, market_id, side, price, original_qty, remaining_qty, 
               filled_qty, status, created_at, updated_at, cancelled_at, filled_at
        FROM orders
        WHERE market_id = $1
        ORDER BY created_at DESC
        "#,
        market_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(orders) => orders,
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch orders: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch orders: {}", e));
        }
    };

    let orders_json: Vec<serde_json::Value> = orders
        .iter()
        .map(|o| {
            serde_json::json!({
                "order_id": o.order_id,
                "user_id": o.user_id,
                "market_id": o.market_id,
                "side": o.side,
                "price": o.price,
                "original_qty": o.original_qty,
                "remaining_qty": o.remaining_qty,
                "filled_qty": o.filled_qty,
                "status": o.status,
                "created_at": o.created_at,
                "updated_at": o.updated_at,
                "cancelled_at": o.cancelled_at,
                "filled_at": o.filled_at
            })
        })
        .collect();

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Orders fetched successfully",
        "orders": orders_json,
        "count": orders.len()
    });

    let response = RedisResponse::new(200, true, "Orders fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_orders_by_market request: request_id={}, market_id={}",
        request_id, market_id
    );
    Ok(())
}
