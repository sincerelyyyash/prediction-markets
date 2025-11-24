use redis_client::RedisManager;
use sqlx::PgPool;
use fred::prelude::*;
use fred::types::XReadResponse;
use log::{error, info};
use serde_json::Value;
use std::collections::HashMap;
use crate::handlers;
use crate::dead_letter;

const DB_EVENTS_STREAM: &str = "db_events";

pub async fn start_db_event_consumer(pool: PgPool) {
    let redis_manager = match RedisManager::global() {
        Some(rm) => rm,
        None => {
            error!("Redis manager not initialized, cannot start DB event consumer");
            return;
        }
    };

    let client = redis_manager.client();
    info!("Starting DB event consumer for stream: {}", DB_EVENTS_STREAM);
    let mut last_id = "0".to_string();

    loop {
        match read_stream_messages(&client, DB_EVENTS_STREAM, &mut last_id).await {
            Ok(messages) => {
                if !messages.is_empty() {
                    if let Err(e) = process_messages(messages, &pool).await {
                        error!("Error processing messages: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stream {}: {}", DB_EVENTS_STREAM, e);
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
    
    let mut result: Vec<(String, HashMap<String, String>)> = Vec::new();
    
    if let RedisValue::Array(streams_array) = raw_result {
        for stream_entry in streams_array {
            if let RedisValue::Array(stream_data) = stream_entry {
                if stream_data.len() >= 2 {
                    if let RedisValue::Array(messages) = &stream_data[1] {
                        for message in messages {
                            if let RedisValue::Array(msg_data) = message {
                                if msg_data.len() >= 2 {
                                    let msg_id = msg_data[0].as_str().map(|s| s.to_string()).unwrap_or_else(|| String::new());
                                    
                                    if let RedisValue::Array(fields_array) = &msg_data[1] {
                                        let mut fields_map = HashMap::new();
                                        
                                        for i in (0..fields_array.len()).step_by(2) {
                                            if i + 1 < fields_array.len() {
                                                let key = fields_array[i].as_str().map(|s| s.to_string()).unwrap_or_else(|| String::new());
                                                let value = fields_array[i + 1].as_str().map(|s| s.to_string()).unwrap_or_else(|| String::new());
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
        let data_str = fields.get("data")
            .ok_or_else(|| "Missing data field in message".to_string())?;

        let event: Value = serde_json::from_str(data_str)
            .map_err(|e| format!("Failed to parse event JSON: {}", e))?;

        let event_clone = event.clone();
        match handlers::handle_db_event(event, pool).await {
            Ok(_) => {
                info!("Successfully processed event: {}", msg_id);
            }
            Err(e) => {
                error!("Failed to process event {}: {}", msg_id, e);
                if let Err(dlq_err) = dead_letter::send_to_dlq(&msg_id, &event_clone, &e).await {
                    error!("Failed to send to DLQ: {}", dlq_err);
                }
            }
        }
    }

    Ok(())
}

