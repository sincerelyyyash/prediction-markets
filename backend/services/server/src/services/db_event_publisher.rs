use redis_client::RedisManager;
use engine::types::db_event_types::DbEvent;
use log::{error, warn};
use serde_json;

const DB_EVENTS_STREAM: &str = "db_events";

pub async fn publish_db_event(event: DbEvent) -> Result<(), String> {
    let redis_manager = match RedisManager::global() {
        Some(rm) => rm,
        None => {
            warn!("Redis manager not initialized, cannot publish DB event");
            return Err("Redis manager not initialized".into());
        }
    };

    let event_json = match serde_json::to_string(&event) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize DB event: {}", e);
            return Err(format!("Failed to serialize event: {}", e));
        }
    };

    match redis_manager
        .stream_add(DB_EVENTS_STREAM, &[("data", &event_json)])
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to publish DB event to stream: {}", e);
            Err(format!("Failed to publish event: {}", e))
        }
    }
}

