use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct RedisRequest<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub service: String,
    pub action: String,
    pub message: String,
    pub data: T,
}

impl<T> RedisRequest<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new(service: impl Into<String>, action: impl Into<String>, message: impl Into<String>, data: T) -> Self {
        Self {
            service: service.into(),
            action: action.into(),
            message: message.into(),
            data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct RedisResponse<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub status_code: i32,
    pub success: bool,
    pub message: String,
    pub data: T,
}

impl<T> RedisResponse<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new(status_code: i32, success: bool, message: impl Into<String>, data: T) -> Self {
        Self {
            status_code,
            success,
            message: message.into(),
            data,
        }
    }
}

