use redis_client::RedisManager;
use redis_client::RedisResponse;
use log::{error, info};
use crate::utils::redis_stream::resolve_pending_request;
use fred::prelude::*;
use fred::types::XReadResponse;

pub async fn start_db_read_response_consumer() {
    info!("start_db_read_response_consumer() called");
    let redis_manager = match RedisManager::global() {
        Some(rm) => rm,
        None => {
            error!("Redis manager not initialized, cannot start DB read response consumer");
            return;
        }
    };

    let client = redis_manager.client();
    let stream_name = "db_read_responses";
    info!("About to spawn DB read response consumer task");

    tokio::spawn(async move {
        info!("Starting DB read response consumer for stream: {}", stream_name);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        info!("DB read response consumer ready to start reading");
        let mut last_id = "0".to_string();

        let mut iteration = 0;
        loop {
            iteration += 1;
            match read_stream_messages(&client, stream_name, &mut last_id).await {
                Ok(messages) => {
                    if !messages.is_empty() {
                        info!("Read {} messages from {} (iteration {})", messages.len(), stream_name, iteration);
                        if let Err(e) = process_message(messages).await {
                            error!("Error processing DB read response message: {}", e);
                        }
                    } else if iteration % 100 == 0 {
                        info!("No messages in {} (iteration {}, last_id: {})", stream_name, iteration, last_id);
                    }
                }
                Err(e) => {
                    error!("Error reading from stream {}: {}", stream_name, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
}

async fn read_stream_messages(
    client: &RedisClient,
    stream: &str,
    last_id: &mut String,
) -> Result<Vec<(String, std::collections::HashMap<String, String>)>, RedisError> {
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
    
    let mut result: Vec<(String, std::collections::HashMap<String, String>)> = Vec::new();
    
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
                                        let mut fields_map = std::collections::HashMap::new();
                                        
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

async fn process_message(messages: Vec<(String, std::collections::HashMap<String, String>)>) -> Result<(), String> {
    for (msg_id, fields) in messages {
        let request_id = fields.get("request_id")
            .ok_or_else(|| "Missing request_id field in message".to_string())?;
        let data_str = fields.get("data")
            .ok_or_else(|| "Missing data field in message".to_string())?;

        let response: RedisResponse<serde_json::Value> = serde_json::from_str(data_str)
            .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

        info!("Processing DB read response: request_id={}, status={}, msg_id={}", request_id, response.status_code, msg_id);
        resolve_pending_request(request_id.clone(), response);
    }
    Ok(())
}

