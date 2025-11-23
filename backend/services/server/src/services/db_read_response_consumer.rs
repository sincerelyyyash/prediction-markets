use redis_client::RedisManager;
use redis_client::RedisResponse;
use log::{error, info};
use crate::utils::redis_stream::resolve_pending_request;
use fred::prelude::*;
use fred::types::XReadResponse;

pub async fn start_db_read_response_consumer() {
    let redis_manager = match RedisManager::global() {
        Some(rm) => rm,
        None => {
            error!("Redis manager not initialized, cannot start DB read response consumer");
            return;
        }
    };

    let client = redis_manager.client();
    let stream_name = "db_read_responses";

    tokio::spawn(async move {
        info!("Starting DB read response consumer for stream: {}", stream_name);
        let mut last_id = "0".to_string();

        loop {
            match read_stream_messages(&client, stream_name, &mut last_id).await {
                Ok(messages) => {
                    if !messages.is_empty() {
                        if let Err(e) = process_message(messages).await {
                            error!("Error processing DB read response message: {}", e);
                        }
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
) -> Result<Vec<(String, Vec<(String, Vec<(String, String)>)>)>, RedisError> {
    use std::collections::HashMap;
    
    let streams = vec![stream];
    let ids = vec![last_id.as_str()];
    
    let xread_result: XReadResponse<String, String, String, RedisValue> = match client
        .xread(Some(10), None, streams, ids)
        .await
    {
        Ok(result) => result,
        Err(e) => {

            let error_msg = e.to_string();
            if error_msg.contains("Cannot convert to map") || error_msg.contains("Parse Error") {

                return Ok(Vec::new());
            }

            return Err(e);
        }
    };
    
    let mut result: Vec<(String, Vec<(String, HashMap<String, String>)>)> = Vec::new();
    for (stream_name, stream_entries) in xread_result.into_iter() {
        let mut messages: Vec<(String, HashMap<String, String>)> = Vec::new();
        
        for (msg_id, fields) in stream_entries {
            let mut fields_map: HashMap<String, String> = HashMap::new();
            
            for (key, value) in fields {
                let key_str = key.to_string();
                let value_str = value.as_str().map(|s| s.to_string()).unwrap_or_else(|| "".to_string());
                fields_map.insert(key_str, value_str);
            }
            
            messages.push((msg_id, fields_map));
        }
        
        if !messages.is_empty() {
            result.push((stream_name, messages));
        }
    }
    
    let mut formatted_result = Vec::new();
    for (stream_name, messages_vec) in result {
        let mut messages = Vec::new();
        for (msg_id, fields_map) in messages_vec {
            let fields: Vec<(String, String)> = fields_map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            messages.push((msg_id.clone(), fields));
            if msg_id > *last_id {
                *last_id = msg_id;
            }
        }
        if !messages.is_empty() {
            formatted_result.push((stream_name, messages));
        }
    }

    Ok(formatted_result)
}

async fn process_message(stream_data: Vec<(String, Vec<(String, Vec<(String, String)>)>)>) -> Result<(), String> {
    for (_stream_name, messages) in stream_data {
        for (_msg_id, fields) in messages {
            let mut request_id: Option<String> = None;
            let mut data_str: Option<String> = None;
            
            for (key, value) in &fields {
                if key == "request_id" {
                    request_id = Some(value.clone());
                } else if key == "data" {
                    data_str = Some(value.clone());
                }
            }
            
            let request_id = request_id.ok_or_else(|| "Missing request_id in message".to_string())?;
            let data_str = data_str.ok_or_else(|| "Missing data in message".to_string())?;

            let response: RedisResponse<serde_json::Value> = serde_json::from_str(&data_str)
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            resolve_pending_request(request_id, response);
        }
    }
    Ok(())
}

