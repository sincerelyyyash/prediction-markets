use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{Utc, Duration};


#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub name: String,
    pub password: String,
    pub balance: u64,
}


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,      
    exp: usize,       
}
