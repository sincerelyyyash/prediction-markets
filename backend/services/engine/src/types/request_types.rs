use serde::Deserialize;
use crate::types::orderbook_types::{OrderSide, OrderType};

#[derive(Debug, Deserialize)]
pub struct PlaceOrderRequest {
    pub market_id: u64,
    pub user_id: u64,
    pub price: Option<u64>,
    #[serde(rename = "original_qty")]
    pub original_qty: u64,
    #[serde(rename = "remaining_qty")]
    pub remaining_qty: u64,
    #[serde(with = "order_side_string")]
    pub side: OrderSide,
    #[serde(with = "order_type_string", default = "default_order_type")]
    pub order_type: OrderType,
}

fn default_order_type() -> OrderType {
    OrderType::Limit
}

mod order_type_string {
    use serde::{Deserialize, Deserializer};
    use crate::types::orderbook_types::OrderType;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<OrderType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Market" => Ok(OrderType::Market),
            "Limit" => Ok(OrderType::Limit),
            _ => Err(serde::de::Error::custom("Invalid order type, must be Market or Limit")),
        }
    }
}

mod order_side_string {
    use serde::{Deserialize, Deserializer};
    use crate::types::orderbook_types::OrderSide;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<OrderSide, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Ask" => Ok(OrderSide::Ask),
            "Bid" => Ok(OrderSide::Bid),
            _ => Err(serde::de::Error::custom("Invalid side, must be Ask or Bid")),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: u64,
    #[serde(default)]
    pub market_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct ModifyOrderRequest {
    pub order_id: u64,
    pub price: Option<u64>,
    #[serde(rename = "original_qty")]
    pub original_qty: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GetOpenOrdersRequest {
    pub user_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct GetOrderStatusRequest {
    pub order_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct GetOrderHistoryRequest {
    #[allow(dead_code)]
    pub user_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub id: u64,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub balance: i64,
}

#[derive(Debug, Deserialize)]
pub struct GetBalanceRequest {
    pub user_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct OnrampRequest {
    pub user_id: u64,
    pub amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct SplitOrderRequest {
    pub user_id: u64,
    pub market1_id: u64,
    pub market2_id: u64,
    pub amount: u64,
}

#[derive(Debug, Deserialize)]
pub struct MergeOrderRequest {
    pub user_id: u64,
    pub market1_id: u64,
    pub market2_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct MarketOutcomeData {
    pub outcome_id: u64,
    pub yes_market_id: u64,
    pub no_market_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct InitEventMarketsRequest {
    pub event_id: u64,
    pub outcomes: Vec<MarketOutcomeData>,
}

#[derive(Debug, Deserialize)]
pub struct CloseEventMarketsRequest {
    pub event_id: u64,
    pub winning_outcome_id: u64,
}

