use crate::handlers;
use fred::prelude::*;
use log::{error, info};
use redis_client::RedisManager;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;

const DB_READ_REQUESTS_STREAM: &str = "db_read_requests";

pub async fn start_read_request_consumer(pool: PgPool) {
    let redis_manager = match RedisManager::global() {
        Some(rm) => rm,
        None => {
            error!("Redis manager not initialized, cannot start read request consumer");
            return;
        }
    };

    let client = redis_manager.client();
    info!(
        "Starting read request consumer for stream: {}",
        DB_READ_REQUESTS_STREAM
    );
    let mut last_id = "0".to_string();

    let mut iteration = 0;
    loop {
        iteration += 1;
        match read_stream_messages(&client, DB_READ_REQUESTS_STREAM, &mut last_id).await {
            Ok(messages) => {
                if !messages.is_empty() {
                    info!(
                        "Read {} messages from stream (iteration {})",
                        messages.len(),
                        iteration
                    );
                    if let Err(e) = process_messages(messages, &pool).await {
                        error!("Error processing read request messages: {}", e);
                    }
                } else if iteration % 100 == 0 {
                    info!(
                        "No messages in stream (iteration {}, last_id: {})",
                        iteration, last_id
                    );
                }
            }
            Err(e) => {
                error!(
                    "Error reading from stream {}: {}",
                    DB_READ_REQUESTS_STREAM, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

async fn read_stream_messages(
    client: &RedisClient,
    stream: &str,
    last_id: &mut String,
) -> Result<Vec<(String, HashMap<String, String>)>, RedisError> {
    use fred::types::RedisValue;

    let streams = vec![stream];
    let ids = vec![last_id.as_str()];

    let raw_result: RedisValue = match client
        .xread::<RedisValue, _, _>(Some(10), None, streams, ids)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            let error_msg = e.to_string();
            error!("XREAD error: {}", error_msg);
            return Err(e);
        }
    };

    info!("XREAD raw result: {:?}", raw_result);

    let mut result: Vec<(String, HashMap<String, String>)> = Vec::new();

    if let RedisValue::Array(streams_array) = raw_result {
        for stream_entry in streams_array {
            if let RedisValue::Array(stream_data) = stream_entry {
                if stream_data.len() >= 2 {
                    if let RedisValue::Array(messages) = &stream_data[1] {
                        for message in messages {
                            if let RedisValue::Array(msg_data) = message {
                                if msg_data.len() >= 2 {
                                    let msg_id = msg_data[0]
                                        .as_str()
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| String::new());

                                    if let RedisValue::Array(fields_array) = &msg_data[1] {
                                        let mut fields_map = HashMap::new();

                                        for i in (0..fields_array.len()).step_by(2) {
                                            if i + 1 < fields_array.len() {
                                                let key = fields_array[i]
                                                    .as_str()
                                                    .map(|s| s.to_string())
                                                    .unwrap_or_else(|| String::new());
                                                let value = fields_array[i + 1]
                                                    .as_str()
                                                    .map(|s| s.to_string())
                                                    .unwrap_or_else(|| String::new());
                                                fields_map.insert(key, value);
                                            }
                                        }

                                        if !msg_id.is_empty() && msg_id > *last_id {
                                            *last_id = msg_id.clone();
                                        }

                                        result.push((msg_id, fields_map));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

async fn process_messages(
    messages: Vec<(String, HashMap<String, String>)>,
    pool: &PgPool,
) -> Result<(), String> {
    for (msg_id, fields) in messages {
        let request_id = fields
            .get("request_id")
            .ok_or_else(|| "Missing request_id field in message".to_string())?;
        let data_str = fields
            .get("data")
            .ok_or_else(|| "Missing data field in message".to_string())?;

        let request: Value = serde_json::from_str(data_str)
            .map_err(|e| format!("Failed to parse request JSON: {}", e))?;

        let action = request
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing action field".to_string())?;

        let data = request
            .get("data")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        info!(
            "Processing read request: action={}, request_id={}, msg_id={}",
            action, request_id, msg_id
        );

        let result = match action {
            "get_all_events" => {
                handlers::handle_get_all_events(data, pool, request_id.clone()).await
            }
            "get_event_by_id" => {
                handlers::handle_get_event_by_id(data, pool, request_id.clone()).await
            }
            "search_events" => handlers::handle_search_events(data, pool, request_id.clone()).await,
            "get_user_by_email" => {
                handlers::handle_get_user_by_email(data, pool, request_id.clone()).await
            }
            "get_admin_by_email" => {
                handlers::handle_get_admin_by_email(data, pool, request_id.clone()).await
            }
            "get_admin_by_id" => {
                handlers::handle_get_admin_by_id(data, pool, request_id.clone()).await
            }
            "get_outcome_by_id" => {
                handlers::handle_get_outcome_by_id(data, pool, request_id.clone()).await
            }
            "get_trades_by_user" => {
                handlers::handle_get_trades_by_user(data, pool, request_id.clone()).await
            }
            "get_trade_by_id" => {
                handlers::handle_get_trade_by_id(data, pool, request_id.clone()).await
            }
            "get_trades_by_market" => {
                handlers::handle_get_trades_by_market(data, pool, request_id.clone()).await
            }
            "get_order_by_id" => {
                handlers::handle_get_order_by_id(data, pool, request_id.clone()).await
            }
            "get_orders_by_user" => {
                handlers::handle_get_orders_by_user(data, pool, request_id.clone()).await
            }
            "get_orders_by_market" => {
                handlers::handle_get_orders_by_market(data, pool, request_id.clone()).await
            }
            "get_user_by_id" => {
                handlers::handle_get_user_by_id(data, pool, request_id.clone()).await
            }
            "get_all_users" => handlers::handle_get_all_users(data, pool, request_id.clone()).await,
            "get_position_by_user_and_market" => {
                handlers::handle_get_position_by_user_and_market(data, pool, request_id.clone())
                    .await
            }
            "get_positions_by_user" => {
                handlers::handle_get_positions_by_user(data, pool, request_id.clone()).await
            }
            "add_market_bookmark" => {
                handlers::handle_add_market_bookmark(data, pool, request_id.clone()).await
            }
            "remove_market_bookmark" => {
                handlers::handle_remove_market_bookmark(data, pool, request_id.clone()).await
            }
            "get_user_bookmarks" => {
                handlers::handle_get_market_bookmarks(data, pool, request_id.clone()).await
            }
            "get_for_you_markets" => {
                handlers::handle_get_for_you_markets(data, pool, request_id.clone()).await
            }
            _ => {
                error!("Unknown read action: {}", action);
                let error_response = redis_client::RedisResponse::new(
                    400,
                    false,
                    format!("Unknown action: {}", action),
                    serde_json::json!(null),
                );
                handlers::send_read_response(&request_id, error_response).await
            }
        };

        if let Err(e) = result {
            error!("Failed to process read request {}: {}", action, e);
        }
    }

    Ok(())
}
