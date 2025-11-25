use crate::types::market_types::MarketSide;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookData {
    pub market_id: u64,
    pub asks: BTreeMap<u64, u64>,
    pub bids: BTreeMap<u64, u64>,
    pub ask_queue: HashMap<u64, Vec<u64>>,
    pub bid_queue: HashMap<u64, Vec<u64>>,
    pub orders: HashMap<u64, Order>,
    pub last_price: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: Option<u64>,
    pub market_id: u64,
    pub user_id: u64,
    pub price: u64,
    pub original_qty: u64,
    pub remaining_qty: u64,
    pub side: OrderSide,
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Ask,
    Bid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSnapshot {
    pub market_id: u64,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub last_price: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub price: u64,
    pub quantity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrderbookSnapshot {
    pub market_id: u64,
    pub side: Option<MarketSide>,
    pub snapshot: OrderbookSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeOrderbookSnapshot {
    pub outcome_id: u64,
    pub event_id: Option<u64>,
    pub markets: Vec<MarketOrderbookSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOrderbookSnapshot {
    pub event_id: u64,
    pub outcomes: Vec<OutcomeOrderbookSnapshot>,
}
