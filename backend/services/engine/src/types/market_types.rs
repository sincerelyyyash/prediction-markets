use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum MarketSide {
    Yes,
    No,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Market {
    pub market_id: u64,
    pub status: MarketStatus,
    pub side: Option<MarketSide>,
    pub paired_market_id: Option<u64>,
    pub event_id: Option<u64>,
    pub outcome_id: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum MarketStatus {
    Active,
    Paused,
    Resolved,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct MarketMeta {
    pub event_id: u64,
    pub outcome_id: u64,
    pub yes_market_id: u64,
    pub no_market_id: u64,
}
