use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use redis_client::{RedisManager, RedisRequest, RedisResponse};
use serde::Serialize;

type PendingRequests = Arc<Mutex<HashMap<String, oneshot::Sender<RedisResponse<serde_json::Value>>>>>;

static PENDING_REQUESTS: once_cell::sync::Lazy<PendingRequests> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub async fn send_request_and_wait<T>(
    request_id: String,
    request: RedisRequest<T>,
    timeout_secs: u64,
) -> Result<RedisResponse<serde_json::Value>, String>
where
    T: Serialize + serde::de::DeserializeOwned + Send + Sync + Clone,
{
    let (tx, rx) = oneshot::channel();
    
    {
        let mut pending = PENDING_REQUESTS.lock().await;
        pending.insert(request_id.clone(), tx);
    }

    let redis_manager = RedisManager::global()
        .ok_or_else(|| "Redis manager not initialized".to_string())?;

    let request_json = serde_json::to_string(&request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;

    redis_manager
        .stream_add(
            "server_requests",
            &[
                ("request_id", &request_id),
                ("data", &request_json),
            ],
        )
        .await
        .map_err(|e| format!("Failed to send request to stream: {}", e))?;

    tokio::select! {
        result = rx => {
            let mut pending = PENDING_REQUESTS.lock().await;
            pending.remove(&request_id);
            result.map_err(|_| "Response channel closed".to_string())
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(timeout_secs)) => {
            let mut pending = PENDING_REQUESTS.lock().await;
            pending.remove(&request_id);
            Err(format!("Timeout waiting for response for request_id: {}", request_id))
        }
    }
}

pub fn resolve_pending_request(request_id: String, response: RedisResponse<serde_json::Value>) {
    let pending = PENDING_REQUESTS.clone();
    tokio::spawn(async move {
        let mut pending_guard = pending.lock().await;
        if let Some(sender) = pending_guard.remove(&request_id) {
            let _ = sender.send(response);
        }
    });
}

