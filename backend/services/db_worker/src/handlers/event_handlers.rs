use super::common::{CACHE_TTL_EVENTS, CACHE_TTL_SEARCH, send_read_response};
use crate::models::OutcomeTable;
use log::{info, warn};
use redis_client::{RedisManager, RedisResponse};
use serde_json::Value;
use sqlx::{PgPool, Row};

pub async fn handle_get_all_events(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let events = match sqlx::query_as!(
        crate::models::EventTable,
        r#"
        SELECT id, slug, title, description, category, status, resolved_at,
            winning_outcome_id, created_by, img_url
        FROM events
        ORDER BY id DESC
        "#
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to fetch events: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to fetch events: {}", e));
        }
    };

    let mut events_with_outcomes = Vec::new();

    for event in &events {
        let outcomes = match sqlx::query_as!(
            OutcomeTable,
            r#"
            SELECT id, event_id, name, status, img_url
            FROM outcomes
            WHERE event_id = $1
            ORDER BY id ASC
            "#,
            event.id
        )
        .fetch_all(pool)
        .await
        {
            Ok(outcomes) => outcomes,
            Err(e) => {
                let error_response = RedisResponse::new(
                    500,
                    false,
                    format!("Failed to fetch outcomes for event {}: {}", event.id, e),
                    serde_json::json!(null),
                );
                send_read_response(&request_id, error_response).await?;
                return Err(format!("Failed to fetch outcomes: {}", e));
            }
        };

        events_with_outcomes.push(serde_json::json!({
            "id": event.id,
            "slug": event.slug,
            "title": event.title,
            "description": event.description,
            "category": event.category,
            "status": event.status,
            "resolved_at": event.resolved_at,
            "winning_outcome_id": event.winning_outcome_id,
            "created_by": event.created_by,
            "img_url": event.img_url,
            "outcomes": outcomes.iter().map(|o| {
                serde_json::json!({
                    "id": o.id,
                    "event_id": o.event_id,
                    "name": &o.name,
                    "status": &o.status,
                    "img_url": o.img_url
                })
            }).collect::<Vec<_>>()
        }));
    }

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Events fetched successfully",
        "events": events_with_outcomes,
        "count": events.len()
    });

    if let Some(redis_manager) = RedisManager::global() {
        if let Ok(response_json) = serde_json::to_string(&response_data) {
            if let Err(e) = redis_manager
                .set_with_ttl("events:all", &response_json, CACHE_TTL_EVENTS)
                .await
            {
                warn!("Failed to cache events data: {:?}", e);
            }
        }
    }

    let response = RedisResponse::new(200, true, "Events fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_all_events request: request_id={}",
        request_id
    );
    Ok(())
}

pub async fn handle_get_event_by_id(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let event_id = data["event_id"]
        .as_u64()
        .ok_or_else(|| "Invalid event_id".to_string())? as i64;

    let event = match sqlx::query_as!(
        crate::models::EventTable,
        r#"
        SELECT id, slug, title, description, category, status,
                resolved_at, winning_outcome_id, created_by, img_url
        FROM events
        WHERE id = $1
        "#,
        event_id
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(event)) => event,
        Ok(None) => {
            let response =
                RedisResponse::new(404, false, "Event not found", serde_json::json!(null));
            send_read_response(&request_id, response).await?;
            return Ok(());
        }
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to load event: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to load event: {}", e));
        }
    };

    let outcomes = match sqlx::query!(
        r#"
        SELECT id, event_id, name, status, img_url
        FROM outcomes
        WHERE event_id = $1
        ORDER BY id ASC
        "#,
        event_id
    )
    .fetch_all(pool)
    .await
    {
        Ok(outcomes) => outcomes,
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to load outcomes: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to load outcomes: {}", e));
        }
    };

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Event fetched successfully",
        "event": {
            "id": event.id,
            "slug": event.slug,
            "title": event.title,
            "description": event.description,
            "category": event.category,
            "status": event.status,
            "resolved_at": event.resolved_at,
            "winning_outcome_id": event.winning_outcome_id,
            "created_by": event.created_by,
            "img_url": event.img_url
        },
        "outcomes": outcomes.iter().map(|o| {
            serde_json::json!({
                "id": o.id,
                "event_id": o.event_id,
                "name": o.name.as_str(),
                "status": o.status.as_str(),
                "img_url": o.img_url
            })
        }).collect::<Vec<_>>()
    });

    if let Some(redis_manager) = RedisManager::global() {
        let cache_key = format!("event:{}", event_id);
        if let Ok(response_json) = serde_json::to_string(&response_data) {
            if let Err(e) = redis_manager
                .set_with_ttl(&cache_key, &response_json, CACHE_TTL_EVENTS)
                .await
            {
                warn!("Failed to cache event data: {:?}", e);
            }
        }
    }

    let response = RedisResponse::new(200, true, "Event fetched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!(
        "Processed get_event_by_id request: request_id={}, event_id={}",
        request_id, event_id
    );
    Ok(())
}

pub async fn handle_search_events(
    data: Value,
    pool: &PgPool,
    request_id: String,
) -> Result<(), String> {
    let q = data["q"].as_str();
    let category = data["category"].as_str();
    let status = data["status"].as_str();

    let cache_key = format!(
        "events:search:q:{}:cat:{}:status:{}",
        q.unwrap_or(""),
        category.unwrap_or(""),
        status.unwrap_or("")
    );

    let mut query_builder = sqlx::QueryBuilder::new(
        r#"
        SELECT id, slug, title, description, category, status, resolved_at,
            winning_outcome_id, created_by
        FROM events
        WHERE 1=1
        "#,
    );

    if let Some(search_term) = q {
        if !search_term.is_empty() {
            let search_pattern = format!("%{}%", search_term);
            query_builder.push(" AND (title ILIKE ");
            query_builder.push_bind(search_pattern.clone());
            query_builder.push(" OR description ILIKE ");
            query_builder.push_bind(search_pattern.clone());
            query_builder.push(" OR slug ILIKE ");
            query_builder.push_bind(search_pattern);
            query_builder.push(")");
        }
    }

    if let Some(cat) = category {
        if !cat.is_empty() {
            query_builder.push(" AND category = ");
            query_builder.push_bind(cat);
        }
    }

    if let Some(stat) = status {
        if !stat.is_empty() {
            query_builder.push(" AND status = ");
            query_builder.push_bind(stat);
        }
    }

    query_builder.push(" ORDER BY id DESC");

    let query = query_builder.build();
    let events_result: Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> =
        query.fetch_all(pool).await;

    let events = match events_result {
        Ok(rows) => rows
            .iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<i64, _>("id"),
                    "slug": row.get::<String, _>("slug"),
                    "title": row.get::<String, _>("title"),
                    "description": row.get::<String, _>("description"),
                    "category": row.get::<String, _>("category"),
                    "status": row.get::<String, _>("status"),
                    "resolved_at": row.get::<Option<String>, _>("resolved_at"),
                    "winning_outcome_id": row.get::<Option<i64>, _>("winning_outcome_id"),
                    "created_by": row.get::<i64, _>("created_by"),
                    "img_url": row.get::<Option<String>, _>("img_url"),
                })
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            let error_response = RedisResponse::new(
                500,
                false,
                format!("Failed to search events: {}", e),
                serde_json::json!(null),
            );
            send_read_response(&request_id, error_response).await?;
            return Err(format!("Failed to search events: {}", e));
        }
    };

    let mut events_with_outcomes = Vec::new();

    for event_json in &events {
        let event_id = event_json["id"].as_i64().unwrap();
        let outcomes = match sqlx::query!(
            r#"
            SELECT id, event_id, name, status, img_url
            FROM outcomes
            WHERE event_id = $1
            ORDER BY id ASC
            "#,
            event_id
        )
        .fetch_all(pool)
        .await
        {
            Ok(outcomes) => outcomes,
            Err(e) => {
                let error_response = RedisResponse::new(
                    500,
                    false,
                    format!("Failed to fetch outcomes for event {}: {}", event_id, e),
                    serde_json::json!(null),
                );
                send_read_response(&request_id, error_response).await?;
                return Err(format!("Failed to fetch outcomes: {}", e));
            }
        };

        let mut event_with_outcomes = event_json.clone();
        event_with_outcomes["outcomes"] = serde_json::json!(
            outcomes
                .iter()
                .map(|o| {
                    serde_json::json!({
                        "id": o.id,
                        "event_id": o.event_id,
                        "name": o.name.as_str(),
                        "status": o.status.as_str(),
                        "img_url": o.img_url
                    })
                })
                .collect::<Vec<_>>()
        );
        events_with_outcomes.push(event_with_outcomes);
    }

    let response_data = serde_json::json!({
        "status": "success",
        "message": "Events searched successfully",
        "events": events_with_outcomes,
        "count": events.len(),
        "query": {
            "q": q,
            "category": category,
            "status": status
        }
    });

    if let Some(redis_manager) = RedisManager::global() {
        if let Ok(response_json) = serde_json::to_string(&response_data) {
            if let Err(e) = redis_manager
                .set_with_ttl(&cache_key, &response_json, CACHE_TTL_SEARCH)
                .await
            {
                warn!("Failed to cache search results: {:?}", e);
            }
        }
    }

    let response = RedisResponse::new(200, true, "Events searched successfully", response_data);

    send_read_response(&request_id, response).await?;
    info!("Processed search_events request: request_id={}", request_id);
    Ok(())
}
