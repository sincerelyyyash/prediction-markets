use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderTypeInput {
    Market,
    Limit,
}

#[derive(Deserialize, Validate, Debug)]
pub struct PlaceOrderInput {
    #[validate(range(min = 1, message = "Market ID must be greater than 0"))]
    pub market_id: u64,
    pub side: OrderSideInput,
    #[serde(default = "default_order_type")]
    pub order_type: OrderTypeInput,
    pub price: Option<u64>,
    #[validate(range(min = 1, message = "Quantity must be greater than 0"))]
    pub quantity: u64,
}

fn default_order_type() -> OrderTypeInput {
    OrderTypeInput::Limit
}

#[derive(Deserialize, Validate, Debug)]
pub struct SplitOrderInput {
    #[validate(range(min = 1, message = "Market 1 ID must be greater than 0"))]
    pub market1_id: u64,
    #[validate(range(min = 1, message = "Market 2 ID must be greater than 0"))]
    pub market2_id: u64,
    #[validate(range(min = 1, message = "Amount must be greater than 0"))]
    pub amount: u64,
}

#[derive(Deserialize, Validate, Debug)]
pub struct MergeOrderInput {
    #[validate(range(min = 1, message = "Market 1 ID must be greater than 0"))]
    pub market1_id: u64,
    #[validate(range(min = 1, message = "Market 2 ID must be greater than 0"))]
    pub market2_id: u64,
}

#[derive(Deserialize, Validate, Debug)]
pub struct ModifyOrderInput {
    #[validate(range(min = 1, message = "Price must be greater than 0"))]
    pub price: Option<u64>,
    #[validate(range(min = 1, message = "Quantity must be greater than 0"))]
    pub quantity: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum OrderSideInput {
    Ask,
    Bid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderResponse {
    pub order_id: Option<u64>,
    pub market_id: u64,
    pub user_id: u64,
    pub price: u64,
    pub original_qty: u64,
    pub remaining_qty: u64,
    pub side: OrderSideResponse,
    pub order_type: OrderTypeResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrderTypeResponse {
    Market,
    Limit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrderSideResponse {
    Ask,
    Bid,
}

#[derive(Serialize, Debug)]
pub struct OrderHistoryResponse {
    pub orders: Vec<OrderResponse>,
}
