use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct EventTable {
    pub id: u64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub status: String,
    pub resolved_at: Option<String>,
    pub winning_outcome_id: Option<u64>,
    pub created_by: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct OutcomeTable {
    pub id: u64,
    pub event_id: u64,
    pub name: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct MarketTable {
    pub id: u64,
    pub outcome_id: u64,
    pub side: String,
}