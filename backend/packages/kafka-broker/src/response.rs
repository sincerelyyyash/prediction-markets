use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KafkaResponse<T>
where 
    T: Serialize + for<'de>Deserialize<'de> + Clone,
{
    pub status_code: i32,
    pub success: bool,
    pub message: String,
    pub data: T,
}

impl<T>KafkaResponse<T>
where
    T: Serialize + for<'de>Deserialize<'de> + Clone,
{
    pub fn new(status_code: impl Into<i32>, success: impl Into<bool>, message: impl Into<String>, data: T) -> Self {
        Self{
            status_code: status_code.into(),
            success: success.into(),
            message: message.into(),
            data,
        }
    }
}