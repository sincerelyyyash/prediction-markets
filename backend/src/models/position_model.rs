use serde::{Deserialize, Serialize};
use sqlx::{FromRow};

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct Position {
    id: u32,
    user_id: u32,
    market_id: u32,
    yes_holding: u32,
    no_holding: u32,
}