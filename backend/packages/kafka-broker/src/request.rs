use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct kafkaRequest<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub service: String,
    pub action: String,
    pub message: String,
    pub data: T,
}

impl<T> kafkaRequest<T>
where
T: Serialize + for<'de> Deserialize<'de> + Clone,
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