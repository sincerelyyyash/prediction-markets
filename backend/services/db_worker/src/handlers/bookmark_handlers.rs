use super::common::send_read_response;
use log::info;
use redis_client::RedisResponse;
use serde_json::{Value, json};
use sqlx::PgPool;

pub async fn handle_add_market_bookmark(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let user_id = data["user_id"]
        .as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())? as i64;
    let market_id = data["market_id"]
        .as_u64()
        .ok_or_else(|| "Invalid market_id".to_string())? as i64;

    let query_result = sqlx::query!(
        r#"
        INSERT INTO user_market_bookmarks (user_id, market_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, market_id) DO NOTHING
        "#,
        user_id,
        market_id
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to add bookmark: {}", e))?;

    let created = query_result.rows_affected() > 0;
    let message = if created {
        "Bookmark added successfully"
    } else {
        "Bookmark already exists"
    };

    let response = RedisResponse::new(
        200,
        true,
        message,
        json!({
            "created": created,
            "bookmark": {
                "user_id": user_id,
                "market_id": market_id
            }
        }),
    );

    send_read_response(&request_id, response).await?;
    info!(
        "Processed add bookmark request: request_id={}, user_id={}, market_id={}, created={}",
        request_id, user_id, market_id, created
    );
    Ok(())
}

pub async fn handle_remove_market_bookmark(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let user_id = data["user_id"]
        .as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())? as i64;
    let market_id = data["market_id"]
        .as_u64()
        .ok_or_else(|| "Invalid market_id".to_string())? as i64;

    let delete_result = sqlx::query!(
        r#"
        DELETE FROM user_market_bookmarks
        WHERE user_id = $1 AND market_id = $2
        "#,
        user_id,
        market_id
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to remove bookmark: {}", e))?;

    if delete_result.rows_affected() == 0 {
        let response = RedisResponse::new(
            404,
            false,
            "Bookmark not found",
            json!({ "market_id": market_id }),
        );
        send_read_response(&request_id, response).await?;
        return Ok(());
    }

    let response = RedisResponse::new(
        200,
        true,
        "Bookmark removed successfully",
        json!({
            "bookmark": {
                "user_id": user_id,
                "market_id": market_id
            }
        }),
    );

    send_read_response(&request_id, response).await?;
    info!(
        "Processed remove bookmark request: request_id={}, user_id={}, market_id={}",
        request_id, user_id, market_id
    );
    Ok(())
}

pub async fn handle_get_market_bookmarks(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let user_id = data["user_id"]
        .as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())? as i64;

    let bookmarks = sqlx::query!(
        r#"
        SELECT
            umb.market_id,
            umb.created_at,
            m.side,
            m.last_price,
            m.img_url,
            o.id AS outcome_id,
            o.name AS outcome_name,
            e.id AS event_id,
            e.slug AS event_slug,
            e.title AS event_title,
            e.category,
            e.status,
            e.description
        FROM user_market_bookmarks umb
        INNER JOIN markets m ON m.id = umb.market_id
        INNER JOIN outcomes o ON o.id = m.outcome_id
        INNER JOIN events e ON e.id = o.event_id
        WHERE umb.user_id = $1
        ORDER BY umb.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch bookmarks: {}", e))?;

    let payload: Vec<Value> = bookmarks
        .iter()
        .map(|row| {
            json!({
                "market_id": row.market_id,
                "created_at": row.created_at.to_rfc3339(),
                "market": {
                    "side": row.side,
                    "last_price": row.last_price,
                    "img_url": row.img_url,
                    "outcome": {
                        "id": row.outcome_id,
                        "name": &row.outcome_name
                    },
                    "event": {
                        "id": row.event_id,
                        "slug": &row.event_slug,
                        "title": &row.event_title,
                        "description": &row.description,
                        "category": &row.category,
                        "status": &row.status
                    }
                }
            })
        })
        .collect();

    let response = RedisResponse::new(
        200,
        true,
        "Bookmarks fetched successfully",
        json!({
            "user_id": user_id,
            "bookmarks": payload,
            "count": payload.len()
        }),
    );

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get bookmarks request: request_id={}, user_id={}, count={}",
        request_id,
        user_id,
        payload.len()
    );
    Ok(())
}

pub async fn handle_get_for_you_markets(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let user_id = data["user_id"]
        .as_u64()
        .ok_or_else(|| "Invalid user_id".to_string())? as i64;

    let limit = data["limit"].as_i64().unwrap_or(20).clamp(1, 100);
    let offset = data["offset"].as_i64().unwrap_or(0).max(0);

    let markets = sqlx::query!(
        r#"
        WITH user_bookmarks AS (
            SELECT market_id
            FROM user_market_bookmarks
            WHERE user_id = $1
        ),
        user_positions AS (
            SELECT DISTINCT market_id
            FROM positions
            WHERE user_id = $1
        ),
        user_categories AS (
            SELECT DISTINCT e.category
            FROM markets m
            INNER JOIN outcomes o ON o.id = m.outcome_id
            INNER JOIN events e ON e.id = o.event_id
            WHERE m.id IN (
                SELECT market_id FROM user_positions
                UNION
                SELECT market_id FROM user_bookmarks
            )
        ),
        for_you AS (
            SELECT DISTINCT
                m.id AS market_id,
                m.side,
                m.last_price,
                m.img_url,
                o.id AS outcome_id,
                o.name AS outcome_name,
                e.id AS event_id,
                e.slug AS event_slug,
                e.title AS event_title,
                e.description,
                e.category,
                e.status,
                COALESCE(p.quantity, 0) AS position_qty,
                CASE
                    WHEN p.market_id IS NOT NULL THEN 'portfolio'
                    ELSE 'category'
                END AS recommendation_reason
            FROM markets m
            INNER JOIN outcomes o ON o.id = m.outcome_id
            INNER JOIN events e ON e.id = o.event_id
            LEFT JOIN positions p
                ON p.market_id = m.id
                AND p.user_id = $1
            WHERE (
                m.id IN (SELECT market_id FROM user_positions)
                OR e.category IN (SELECT category FROM user_categories)
            )
            AND m.id NOT IN (SELECT market_id FROM user_bookmarks)
        )
        SELECT *
        FROM for_you
        ORDER BY recommendation_reason = 'portfolio' DESC, event_id DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch For You markets: {}", e))?;

    let payload: Vec<Value> = markets
        .iter()
        .map(|row| {
            json!({
                "market_id": row.market_id,
                "market": {
                    "side": &row.side,
                    "last_price": row.last_price,
                    "img_url": &row.img_url,
                    "outcome": {
                        "id": row.outcome_id,
                        "name": &row.outcome_name
                    },
                    "event": {
                        "id": row.event_id,
                        "slug": &row.event_slug,
                        "title": &row.event_title,
                        "description": &row.description,
                        "category": &row.category,
                        "status": &row.status
                    }
                },
                "position_qty": row.position_qty,
                "recommendation_reason": &row.recommendation_reason
            })
        })
        .collect();

    let response = RedisResponse::new(
        200,
        true,
        "For You markets fetched successfully",
        json!({
            "user_id": user_id,
            "markets": payload,
            "count": payload.len(),
            "limit": limit,
            "offset": offset
        }),
    );

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get For You request: request_id={}, user_id={}, count={}",
        request_id,
        user_id,
        payload.len()
    );
    Ok(())
}
