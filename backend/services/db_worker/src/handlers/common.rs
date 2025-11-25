use redis_client::{RedisManager, RedisResponse};

pub const CACHE_TTL_EVENTS: i64 = 86400;
pub const CACHE_TTL_SEARCH: i64 = 3600;

pub async fn send_read_response(
    request_id: &str,
    response: RedisResponse<serde_json::Value>,
) -> Result<(), String> {
    let redis_manager = match RedisManager::global() {
        Some(rm) => rm,
        None => {
            return Err("Redis manager not initialized".into());
        }
    };

    let response_json = serde_json::to_string(&response)
        .map_err(|e| format!("Failed to serialize response: {}", e))?;

    redis_manager
        .stream_add(
            "db_read_responses",
            &[("request_id", &request_id), ("data", &response_json)],
        )
        .await
        .map_err(|e| format!("Failed to send response: {}", e))?;

    Ok(())
}
