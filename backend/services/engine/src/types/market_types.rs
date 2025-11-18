use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone )]
pub struct Market {
    pub market_id: u64,
    pub status: MarketStatus,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum MarketStatus {
    Active,
    Paused,
    Resolved,
    Cancelled,
}