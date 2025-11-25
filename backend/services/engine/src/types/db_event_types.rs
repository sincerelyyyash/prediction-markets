use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum DbEvent {
    #[serde(rename = "order_placed")]
    OrderPlaced(OrderPlacedEvent),
    #[serde(rename = "order_cancelled")]
    OrderCancelled(OrderCancelledEvent),
    #[serde(rename = "order_modified")]
    OrderModified(OrderModifiedEvent),
    #[serde(rename = "order_filled")]
    OrderFilled(OrderFilledEvent),
    #[serde(rename = "trade_executed")]
    TradeExecuted(TradeExecutedEvent),
    #[serde(rename = "position_updated")]
    PositionUpdated(PositionUpdatedEvent),
    #[serde(rename = "balance_updated")]
    BalanceUpdated(BalanceUpdatedEvent),
    #[serde(rename = "user_created")]
    UserCreated(UserCreatedEvent),
    #[serde(rename = "event_created")]
    EventCreated(EventCreatedEvent),
    #[serde(rename = "event_resolved")]
    EventResolved(EventResolvedEvent),
    #[serde(rename = "event_updated")]
    EventUpdated(EventUpdatedEvent),
    #[serde(rename = "event_deleted")]
    EventDeleted(EventDeletedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPlacedEvent {
    pub order_id: u64,
    pub user_id: u64,
    pub market_id: u64,
    pub side: String,
    pub price: u64,
    pub original_qty: u64,
    pub remaining_qty: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancelledEvent {
    pub order_id: u64,
    pub user_id: u64,
    pub market_id: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderModifiedEvent {
    pub order_id: u64,
    pub user_id: u64,
    pub market_id: u64,
    pub price: u64,
    pub original_qty: u64,
    pub remaining_qty: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFilledEvent {
    pub order_id: u64,
    pub user_id: u64,
    pub market_id: u64,
    pub filled_qty: u64,
    pub remaining_qty: u64,
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecutedEvent {
    pub trade_id: String,
    pub market_id: u64,
    pub taker_order_id: u64,
    pub maker_order_id: u64,
    pub taker_user_id: u64,
    pub maker_user_id: u64,
    pub price: u64,
    pub quantity: u64,
    pub taker_side: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdatedEvent {
    pub user_id: u64,
    pub market_id: u64,
    pub quantity: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceUpdatedEvent {
    pub user_id: u64,
    pub balance: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub user_id: u64,
    pub email: String,
    pub name: String,
    pub password: String,
    pub balance: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeData {
    pub outcome_id: u64,
    pub name: String,
    pub status: String,
    pub yes_market_id: u64,
    pub no_market_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCreatedEvent {
    pub event_id: u64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub status: String,
    pub resolved_at: Option<String>,
    pub created_by: u64,
    pub outcomes: Vec<OutcomeData>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResolvedEvent {
    pub event_id: u64,
    pub status: String,
    pub resolved_at: String,
    pub winning_outcome_id: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventUpdatedEvent {
    pub event_id: u64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDeletedEvent {
    pub event_id: u64,
    pub timestamp: DateTime<Utc>,
}
