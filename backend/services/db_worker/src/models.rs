use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct EventTable {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub status: String,
    pub resolved_at: Option<String>,
    pub winning_outcome_id: Option<i64>,
    pub created_by: i64,
    pub img_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct OutcomeTable {
    pub id: i64,
    pub event_id: i64,
    pub name: String,
    pub status: String,
    pub img_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct MarketTable {
    pub id: i64,
    pub outcome_id: i64,
    pub side: String,
    pub last_price: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct UserTable {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub password: String,
    pub balance: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct AdminTable {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct UserMarketBookmark {
    pub id: i64,
    pub user_id: i64,
    pub market_id: i64,
    pub created_at: DateTime<Utc>,
}
