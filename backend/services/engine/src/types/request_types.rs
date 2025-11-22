use serde::Deserialize;
use crate::types::orderbook_types::OrderSide;

#[derive(Debug, Deserialize)]
pub struct PlaceOrderRequest {
    pub market_id: u64,
    pub user_id: u64,
    pub price: u64,
    #[serde(rename = "original_qty")]
    pub original_qty: u64,
    #[serde(rename = "remaining_qty")]
    pub remaining_qty: u64,
    #[serde(with = "order_side_string")]
    pub side: OrderSide,
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

