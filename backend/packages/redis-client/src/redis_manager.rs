use std::{collections::HashMap, sync::Arc};

use fred::prelude::*;
use log::{error, info, warn};
use once_cell::sync::OnceCell;
use tokio::{sync::Mutex, task::JoinHandle};

type SubscriberMap = HashMap<String, JoinHandle<()>>;

#[derive(Clone)]
pub struct RedisManager {
    client: RedisClient,
    subscribers: Arc<Mutex<SubscriberMap>>,
}

static INSTANCE: OnceCell<RedisManager> = OnceCell::new();

impl RedisManager {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        let config = RedisConfig::from_url(redis_url)?;
        let client = RedisClient::new(config, None, None, None);

        Ok(Self {
            client,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn init_global(redis_url: &str) -> Result<&'static RedisManager, RedisError> {
        INSTANCE.get_or_try_init(|| Self::new(redis_url))
    }

    pub fn global() -> Option<&'static RedisManager> {
        INSTANCE.get()
    }

    pub fn client(&self) -> RedisClient {
        self.client.clone()
    }

    pub async fn connect(&self) -> Result<(), RedisError> {
        self.client.connect();
        self.client.wait_for_connect().await?;
        info!("Connected to Redis");
        Ok(())
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), RedisError> {
        self.client
            .set::<(), _, _>(key, value, None, None, false)
            .await
    }

    pub async fn set_with_ttl(
        &self,
        key: &str,
        value: &str,
        seconds: i64,
    ) -> Result<(), RedisError> {
        self.client
            .set::<(), _, _>(key, value, None, None, false)
            .await?;
        self.client.expire::<(), _>(key, seconds).await?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, RedisError> {
        let value: Option<String> = self.client.get(key).await?;
        Ok(value)
    }

    pub async fn delete(&self, key: &str) -> Result<(), RedisError> {
        self.client.del::<(), _>(key).await
    }

    pub async fn push_queue(&self, queue: &str, value: &str) -> Result<(), RedisError> {
        self.client.lpush::<(), _, _>(queue, value).await
    }

    pub async fn pop_queue(&self, queue: &str) -> Result<Option<String>, RedisError> {
        let value: Option<String> = self.client.rpop(queue, None).await?;
        Ok(value)
    }

    pub async fn publish(&self, channel: &str, payload: &str) -> Result<(), RedisError> {
        self.client.publish::<(), _, _>(channel, payload).await
    }

    pub async fn subscribe<F>(&self, channel: &str, handler: F) -> Result<(), RedisError>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let mut guard = self.subscribers.lock().await;
        if let Some(handle) = guard.remove(channel) {
            handle.abort();
            warn!("Replaced existing subscriber channel for {}", channel);
        }

        let client = self.client.clone();
        let channel_name = channel.to_string();
        let channel_key = channel_name.clone();
        let handler_arc = Arc::new(handler);

        let task = tokio::spawn({
            let handler_clone = handler_arc.clone();
            async move {
                if let Err(err) = client.subscribe(&channel_name).await {
                    error!("Subscribe failed for {}: {}", channel_name, err);
                    return;
                }

                let mut message_stream = client.message_rx();
                while let Ok(message) = message_stream.recv().await {
                    if let Some(text) = message.value.as_str() {
                        handler_clone(text.to_string());
                    }
                }
            }
        });

        (*guard).insert(channel_key, task);
        Ok(())
    }

    pub async fn unsubscribe(&self, channel: &str) {
        let mut guard = self.subscribers.lock().await;
        if let Some(handle) = guard.remove(channel) {
            handle.abort();
            info!("Unsubscribed from {}", channel);
        }
    }

    pub async fn stream_add(&self, stream: &str, pairs: &[(&str, &str)]) -> Result<(), RedisError> {
        let mut fields: Vec<(String, String)> = Vec::with_capacity(pairs.len());
        for (field, value) in pairs {
            fields.push(((*field).to_owned(), (*value).to_owned()));
        }

        self.client
            .xadd::<(), _, _, _, _>(stream, false, None, "*", fields)
            .await
    }
}
