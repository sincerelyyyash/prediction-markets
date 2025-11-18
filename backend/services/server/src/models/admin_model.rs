use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Admin {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub password: String,
}