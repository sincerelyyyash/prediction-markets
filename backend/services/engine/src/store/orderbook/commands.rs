use tokio::sync::oneshot;

use crate::types::market_types::MarketMeta;
use crate::types::orderbook_types::{
    EventOrderbookSnapshot, Order, OrderbookSnapshot, OutcomeOrderbookSnapshot,
};
use crate::types::user_types::User;

#[derive(Debug)]
pub enum Command {
    PlaceOrder(Order, oneshot::Sender<Result<Order, String>>),
    CancelOrder(u64, u64, oneshot::Sender<Result<Order, String>>),
    ModifyOrder(Order, oneshot::Sender<Result<Order, String>>),

    GetBestBid(u64, oneshot::Sender<Result<u64, String>>),
    GetBestAsk(u64, oneshot::Sender<Result<u64, String>>),
    GetOrderBook(u64, oneshot::Sender<Result<OrderbookSnapshot, String>>),
    GetOrderbooksByEvent(u64, oneshot::Sender<Result<EventOrderbookSnapshot, String>>),
    GetOrderbooksByOutcome(
        u64,
        oneshot::Sender<Result<OutcomeOrderbookSnapshot, String>>,
    ),
    GetUserOpenOrders(u64, oneshot::Sender<Result<Vec<Order>, String>>),
    GetOrderStatus(u64, oneshot::Sender<Result<Order, String>>),
    AddUser(User, oneshot::Sender<Option<User>>),
    GetUserByEmail(String, oneshot::Sender<Option<User>>),
    GetUserById(u64, oneshot::Sender<Option<User>>),
    GetBalance(u64, oneshot::Sender<Result<i64, String>>),
    UpdateBalance(u64, i64, oneshot::Sender<Result<(), String>>),
    GetPosition(u64, u64, oneshot::Sender<Result<u64, String>>),
    GetUserPositions(
        u64,
        oneshot::Sender<Result<::std::collections::HashMap<u64, u64>, String>>,
    ),
    UpdatePosition(u64, u64, i64, oneshot::Sender<Result<(), String>>),
    CheckPositionSufficient(u64, u64, u64, oneshot::Sender<Result<bool, String>>),
    CreateSplitPosition(u64, u64, u64, u64, oneshot::Sender<Result<(), String>>),
    MergePosition(u64, u64, u64, oneshot::Sender<Result<(), String>>),
    InitMarkets(Vec<MarketMeta>, oneshot::Sender<Result<(), String>>),
    CloseEventMarkets(u64, u64, oneshot::Sender<Result<(), String>>),
}
