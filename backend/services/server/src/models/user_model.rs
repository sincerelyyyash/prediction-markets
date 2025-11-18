use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub password: String,
    pub balance: i64,
}

