use redis_client::RedisManager;
use serde_json::Value;
use log::{error, info};

const DLQ_STREAM: &str = "db_events:dlq";

pub async fn send_to_dlq(message_id: &str, event: &Value, error: &str) -> Result<(), String> {
    let redis_manager = match RedisManager::global() {
        Some(rm) => rm,
        None => {
            error!("Redis manager not initialized, cannot send to DLQ");
            return Err("Redis manager not initialized".into());
        }
    };

    let event_json = serde_json::to_string(event)
        .map_err(|e| format!("Failed to serialize event for DLQ: {}", e))?;

    let dlq_message = serde_json::json!({
        "message_id": message_id,
        "event": event_json,
        "error": error,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let dlq_json = serde_json::to_string(&dlq_message)
        .map_err(|e| format!("Failed to serialize DLQ message: {}", e))?;

    match redis_manager
        .stream_add(DLQ_STREAM, &[("data", &dlq_json)])
        .await
    {
        Ok(_) => {
            info!("Sent failed message to DLQ: message_id={}, error={}", message_id, error);
            Ok(())
        }
        Err(e) => {
            error!("Failed to send message to DLQ: {}", e);
            Err(format!("Failed to send to DLQ: {}", e))
        }
    }
}

