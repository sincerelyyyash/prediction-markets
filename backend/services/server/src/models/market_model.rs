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
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct OutcomeTable {
    pub id: i64,
    pub event_id: i64,
    pub name: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct MarketTable {
    pub id: i64,
    pub outcome_id: i64,
    pub side: String,
}