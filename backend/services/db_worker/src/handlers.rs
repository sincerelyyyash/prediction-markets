use sqlx::PgPool;
use serde_json::Value;
use log::{info, warn};
use chrono::Utc;
use redis_client::RedisManager;

pub async fn handle_db_event(event: Value, pool: &PgPool) -> Result<(), String> {
    let event_type = event.get("event_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing event_type field".to_string())?;

    match event_type {
        "order_placed" => {
            handle_order_placed(event, pool).await
        }
        "order_cancelled" => {
            handle_order_cancelled(event, pool).await
        }
        "order_modified" => {
            handle_order_modified(event, pool).await
        }
        "order_filled" => {
            handle_order_filled(event, pool).await
        }
        "trade_executed" => {
            handle_trade_executed(event, pool).await
        }
        "position_updated" => {
            handle_position_updated(event, pool).await
        }
        "balance_updated" => {
            handle_balance_updated(event, pool).await
        }
        "user_created" => {
            handle_user_created(event, pool).await
        }
        "event_created" => {
            handle_event_created(event, pool).await
        }
        "event_resolved" => {
            handle_event_resolved(event, pool).await
        }
        "event_updated" => {
            handle_event_updated(event, pool).await
        }
        "event_deleted" => {
            handle_event_deleted(event, pool).await
        }
        _ => {
            Err(format!("Unknown event type: {}", event_type))
        }
    }
}

async fn handle_order_placed(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let order_id = data["order_id"].as_u64()
        .ok_or_else(|| "Invalid order_id".to_string())?;
    let user_id = data["user_id"].as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())?;
    let market_id = data["market_id"].as_u64()
        .ok_or_else(|| "Invalid market_id".to_string())?;
    let side = data["side"].as_str()
        .ok_or_else(|| "Invalid side".to_string())?;
    let price = data["price"].as_u64()
        .ok_or_else(|| "Invalid price".to_string())?;
    let original_qty = data["original_qty"].as_u64()
        .ok_or_else(|| "Invalid original_qty".to_string())?;
    let remaining_qty = data["remaining_qty"].as_u64()
        .ok_or_else(|| "Invalid remaining_qty".to_string())?;

    sqlx::query!(
        r#"
        INSERT INTO orders (order_id, user_id, market_id, side, price, original_qty, remaining_qty, filled_qty, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 'open')
        ON CONFLICT (order_id) DO NOTHING
        "#,
        order_id as i64,
        user_id as i64,
        market_id as i64,
        side,
        price as i64,
        original_qty as i64,
        remaining_qty as i64,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert order: {}", e))?;

    info!("Order placed: order_id={}, user_id={}", order_id, user_id);
    Ok(())
}

async fn handle_order_cancelled(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let order_id = data["order_id"].as_u64()
        .ok_or_else(|| "Invalid order_id".to_string())?;

    sqlx::query!(
        r#"
        UPDATE orders
        SET status = 'cancelled', cancelled_at = NOW(), updated_at = NOW()
        WHERE order_id = $1
        "#,
        order_id as i64,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to cancel order: {}", e))?;

    info!("Order cancelled: order_id={}", order_id);
    Ok(())
}

async fn handle_order_modified(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let order_id = data["order_id"].as_u64()
        .ok_or_else(|| "Invalid order_id".to_string())?;
    let price = data["price"].as_u64()
        .ok_or_else(|| "Invalid price".to_string())?;
    let original_qty = data["original_qty"].as_u64()
        .ok_or_else(|| "Invalid original_qty".to_string())?;
    let remaining_qty = data["remaining_qty"].as_u64()
        .ok_or_else(|| "Invalid remaining_qty".to_string())?;

    sqlx::query!(
        r#"
        UPDATE orders
        SET price = $1, original_qty = $2, remaining_qty = $3, updated_at = NOW()
        WHERE order_id = $4
        "#,
        price as i64,
        original_qty as i64,
        remaining_qty as i64,
        order_id as i64,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to modify order: {}", e))?;

    info!("Order modified: order_id={}", order_id);
    Ok(())
}

async fn handle_order_filled(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let order_id = data["order_id"].as_u64()
        .ok_or_else(|| "Invalid order_id".to_string())?;
    let filled_qty = data["filled_qty"].as_u64()
        .ok_or_else(|| "Invalid filled_qty".to_string())?;
    let remaining_qty = data["remaining_qty"].as_u64()
        .ok_or_else(|| "Invalid remaining_qty".to_string())?;
    let status = data["status"].as_str()
        .ok_or_else(|| "Invalid status".to_string())?;

    let filled_at = if status == "filled" {
        Some(Utc::now().naive_utc())
    } else {
        None
    };

    sqlx::query!(
        r#"
        UPDATE orders
        SET filled_qty = $1, remaining_qty = $2, status = $3, updated_at = NOW(), filled_at = $4
        WHERE order_id = $5
        "#,
        filled_qty as i64,
        remaining_qty as i64,
        status,
        filled_at,
        order_id as i64,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update order fill: {}", e))?;

    info!("Order filled: order_id={}, filled_qty={}, status={}", order_id, filled_qty, status);
    Ok(())
}

async fn handle_trade_executed(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let trade_id = data["trade_id"].as_str()
        .ok_or_else(|| "Invalid trade_id".to_string())?;
    let market_id = data["market_id"].as_u64()
        .ok_or_else(|| "Invalid market_id".to_string())?;
    let taker_order_id = data["taker_order_id"].as_u64()
        .ok_or_else(|| "Invalid taker_order_id".to_string())?;
    let maker_order_id = data["maker_order_id"].as_u64()
        .ok_or_else(|| "Invalid maker_order_id".to_string())?;
    let taker_user_id = data["taker_user_id"].as_u64()
        .ok_or_else(|| "Invalid taker_user_id".to_string())?;
    let maker_user_id = data["maker_user_id"].as_u64()
        .ok_or_else(|| "Invalid maker_user_id".to_string())?;
    let price = data["price"].as_u64()
        .ok_or_else(|| "Invalid price".to_string())?;
    let quantity = data["quantity"].as_u64()
        .ok_or_else(|| "Invalid quantity".to_string())?;
    let taker_side = data["taker_side"].as_str()
        .ok_or_else(|| "Invalid taker_side".to_string())?;

    sqlx::query!(
        r#"
        INSERT INTO trades (trade_id, market_id, taker_order_id, maker_order_id, taker_user_id, maker_user_id, price, quantity, taker_side)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (trade_id) DO NOTHING
        "#,
        trade_id,
        market_id as i64,
        taker_order_id as i64,
        maker_order_id as i64,
        taker_user_id as i64,
        maker_user_id as i64,
        price as i64,
        quantity as i64,
        taker_side,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert trade: {}", e))?;

    info!("Trade executed: trade_id={}, market_id={}", trade_id, market_id);
    Ok(())
}

async fn handle_position_updated(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let user_id = data["user_id"].as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())?;
    let market_id = data["market_id"].as_u64()
        .ok_or_else(|| "Invalid market_id".to_string())?;
    let quantity = data["quantity"].as_u64()
        .ok_or_else(|| "Invalid quantity".to_string())?;

    if quantity == 0 {
        sqlx::query!(
            r#"
            DELETE FROM positions
            WHERE user_id = $1 AND market_id = $2
            "#,
            user_id as i64,
            market_id as i64,
        )
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete position: {}", e))?;
    } else {
        let exists: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(SELECT 1 FROM positions WHERE user_id = $1 AND market_id = $2)
            "#,
            user_id as i64,
            market_id as i64,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to check position existence: {}", e))?;

        if exists {
            sqlx::query!(
                r#"
                UPDATE positions
                SET quantity = $1, updated_at = NOW()
                WHERE user_id = $2 AND market_id = $3
                "#,
                quantity as i64,
                user_id as i64,
                market_id as i64,
            )
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update position: {}", e))?;
        } else {
            sqlx::query!(
                r#"
                INSERT INTO positions (user_id, market_id, quantity, updated_at)
                VALUES ($1, $2, $3, NOW())
                "#,
                user_id as i64,
                market_id as i64,
                quantity as i64,
            )
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to insert position: {}", e))?;
        }
    }

    info!("Position updated: user_id={}, market_id={}, quantity={}", user_id, market_id, quantity);
    Ok(())
}

async fn handle_balance_updated(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let user_id = data["user_id"].as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())?;
    let balance = data["balance"].as_i64()
        .ok_or_else(|| "Invalid balance".to_string())?;

    sqlx::query!(
        r#"
        UPDATE users
        SET balance = $1
        WHERE id = $2
        "#,
        balance,
        user_id as i64,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update balance: {}", e))?;

    info!("Balance updated: user_id={}, balance={}", user_id, balance);
    Ok(())
}

async fn handle_user_created(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let user_id = data["user_id"].as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())?;
    let email = data["email"].as_str()
        .ok_or_else(|| "Invalid email".to_string())?;
    let name = data["name"].as_str()
        .ok_or_else(|| "Invalid name".to_string())?;
    let password = data["password"].as_str()
        .ok_or_else(|| "Invalid password".to_string())?;
    let balance = data["balance"].as_i64()
        .ok_or_else(|| "Invalid balance".to_string())?;

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, name, password, balance)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO NOTHING
        "#,
        user_id as i64,
        email,
        name,
        password,
        balance,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert user: {}", e))?;

    info!("User created: user_id={}, email={}", user_id, email);
    Ok(())
}

async fn handle_event_created(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let event_id = data["event_id"].as_u64()
        .ok_or_else(|| "Invalid event_id".to_string())?;
    let slug = data["slug"].as_str()
        .ok_or_else(|| "Invalid slug".to_string())?;
    let title = data["title"].as_str()
        .ok_or_else(|| "Invalid title".to_string())?;
    let description = data["description"].as_str()
        .ok_or_else(|| "Invalid description".to_string())?;
    let category = data["category"].as_str()
        .ok_or_else(|| "Invalid category".to_string())?;
    let status = data["status"].as_str()
        .ok_or_else(|| "Invalid status".to_string())?;
    let resolved_at = data["resolved_at"].as_str();
    let created_by = data["created_by"].as_u64()
        .ok_or_else(|| "Invalid created_by".to_string())?;
    let outcomes = data["outcomes"].as_array()
        .ok_or_else(|| "Invalid outcomes".to_string())?;

    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    sqlx::query!(
        r#"
        INSERT INTO events (id, slug, title, description, category, status, resolved_at, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (id) DO NOTHING
        "#,
        event_id as i64,
        slug,
        title,
        description,
        category,
        status,
        resolved_at,
        created_by as i64,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to insert event: {}", e))?;

    for outcome_data in outcomes {
        let outcome_id = outcome_data["outcome_id"].as_u64()
            .ok_or_else(|| "Invalid outcome_id".to_string())?;
        let name = outcome_data["name"].as_str()
            .ok_or_else(|| "Invalid outcome name".to_string())?;
        let outcome_status = outcome_data["status"].as_str()
            .ok_or_else(|| "Invalid outcome status".to_string())?;
        let yes_market_id = outcome_data["yes_market_id"].as_u64()
            .ok_or_else(|| "Invalid yes_market_id".to_string())?;
        let no_market_id = outcome_data["no_market_id"].as_u64()
            .ok_or_else(|| "Invalid no_market_id".to_string())?;

        sqlx::query!(
            r#"
            INSERT INTO outcomes (id, event_id, name, status)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO NOTHING
            "#,
            outcome_id as i64,
            event_id as i64,
            name,
            outcome_status,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert outcome: {}", e))?;

        sqlx::query!(
            r#"
            INSERT INTO markets (id, outcome_id, side)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO NOTHING
            "#,
            yes_market_id as i64,
            outcome_id as i64,
            "YES",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert YES market: {}", e))?;

        sqlx::query!(
            r#"
            INSERT INTO markets (id, outcome_id, side)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO NOTHING
            "#,
            no_market_id as i64,
            outcome_id as i64,
            "NO",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert NO market: {}", e))?;
    }

    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    if let Some(redis_manager) = RedisManager::global() {
        if let Err(e) = redis_manager.delete("events:all").await {
            warn!("Failed to invalidate events:all cache after event creation: {:?}", e);
        }
    }

    info!("Event created: event_id={}, title={}", event_id, title);
    Ok(())
}

async fn handle_event_resolved(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let event_id = data["event_id"].as_u64()
        .ok_or_else(|| "Invalid event_id".to_string())?;
    let status = data["status"].as_str()
        .ok_or_else(|| "Invalid status".to_string())?;
    let resolved_at = data["resolved_at"].as_str()
        .ok_or_else(|| "Invalid resolved_at".to_string())?;
    let winning_outcome_id = data["winning_outcome_id"].as_u64()
        .ok_or_else(|| "Invalid winning_outcome_id".to_string())?;

    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    sqlx::query!(
        r#"
        UPDATE events
        SET status = $1, resolved_at = $2, winning_outcome_id = $3
        WHERE id = $4
        "#,
        status,
        resolved_at,
        winning_outcome_id as i64,
        event_id as i64,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to update event: {}", e))?;

    sqlx::query!(
        r#"
        UPDATE outcomes
        SET status = 'RESOLVED'
        WHERE id = $1
        "#,
        winning_outcome_id as i64,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to mark winning outcome: {}", e))?;

    sqlx::query!(
        r#"
        UPDATE outcomes
        SET status = 'REJECTED'
        WHERE event_id = $1 AND id <> $2
        "#,
        event_id as i64,
        winning_outcome_id as i64,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to mark losing outcomes: {}", e))?;

    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    if let Some(redis_manager) = RedisManager::global() {
        let cache_key = format!("event:{}", event_id);
        if let Err(e) = redis_manager.delete("events:all").await {
            warn!("Failed to invalidate events:all cache after resolution: {:?}", e);
        }
        if let Err(e) = redis_manager.delete(&cache_key).await {
            warn!("Failed to invalidate event cache after resolution: {:?}", e);
        }
    }

    info!("Event resolved: event_id={}, winning_outcome_id={}", event_id, winning_outcome_id);
    Ok(())
}

async fn handle_event_updated(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let event_id = data["event_id"].as_u64()
        .ok_or_else(|| "Invalid event_id".to_string())?;
    let slug = data["slug"].as_str()
        .ok_or_else(|| "Invalid slug".to_string())?;
    let title = data["title"].as_str()
        .ok_or_else(|| "Invalid title".to_string())?;
    let description = data["description"].as_str()
        .ok_or_else(|| "Invalid description".to_string())?;
    let category = data["category"].as_str()
        .ok_or_else(|| "Invalid category".to_string())?;
    let status = data["status"].as_str()
        .ok_or_else(|| "Invalid status".to_string())?;

    sqlx::query!(
        r#"
        UPDATE events
        SET slug = $1, title = $2, description = $3, status = $4, category = $5
        WHERE id = $6
        "#,
        slug,
        title,
        description,
        status,
        category,
        event_id as i64,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update event: {}", e))?;

    if let Some(redis_manager) = RedisManager::global() {
        let cache_key = format!("event:{}", event_id);
        if let Err(e) = redis_manager.delete("events:all").await {
            warn!("Failed to invalidate events:all cache after update: {:?}", e);
        }
        if let Err(e) = redis_manager.delete(&cache_key).await {
            warn!("Failed to invalidate event cache after update: {:?}", e);
        }
    }

    info!("Event updated: event_id={}, title={}", event_id, title);
    Ok(())
}

async fn handle_event_deleted(event: Value, pool: &PgPool) -> Result<(), String> {
    let data = &event;

    let event_id = data["event_id"].as_u64()
        .ok_or_else(|| "Invalid event_id".to_string())?;

    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    sqlx::query!(
        r#"
        DELETE FROM markets
        WHERE outcome_id IN (
            SELECT id FROM outcomes WHERE event_id = $1
        )
        "#,
        event_id as i64,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to delete markets: {}", e))?;

    sqlx::query!(
        r#"
        DELETE FROM outcomes
        WHERE event_id = $1
        "#,
        event_id as i64,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to delete outcomes: {}", e))?;

    sqlx::query!(
        r#"
        DELETE FROM events
        WHERE id = $1
        "#,
        event_id as i64,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to delete event: {}", e))?;

    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    if let Some(redis_manager) = RedisManager::global() {
        let cache_key = format!("event:{}", event_id);
        if let Err(e) = redis_manager.delete("events:all").await {
            warn!("Failed to invalidate events:all cache after deletion: {:?}", e);
        }
        if let Err(e) = redis_manager.delete(&cache_key).await {
            warn!("Failed to invalidate event cache after deletion: {:?}", e);
        }
    }

    info!("Event deleted: event_id={}", event_id);
    Ok(())
}

