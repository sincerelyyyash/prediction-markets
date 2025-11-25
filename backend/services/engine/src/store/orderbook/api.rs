use std::collections::HashMap;

use tokio::sync::{mpsc, oneshot};

use crate::store::orderbook::commands::Command;
use crate::types::market_types::MarketMeta;
use crate::types::orderbook_types::{
    EventOrderbookSnapshot, Order, OrderbookSnapshot, OutcomeOrderbookSnapshot,
};
use crate::types::user_types::User;

#[derive(Clone)]
pub struct Orderbook {
    pub(crate) tx: mpsc::Sender<Command>,
}

impl Orderbook {
    pub(crate) fn new(tx: mpsc::Sender<Command>) -> Self {
        Self { tx }
    }

    pub async fn place_order(&self, order: Order) -> Result<Order, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::PlaceOrder(order, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to place order".into()))
    }

    pub async fn cancel_order(&self, market_id: u64, order_id: u64) -> Result<Order, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::CancelOrder(market_id, order_id, tx))
            .await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to cancel order".into()))
    }

    pub async fn modify_order(&self, order: Order) -> Result<Order, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::ModifyOrder(order, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to modify order".into()))
    }

    pub async fn best_bid(&self, market_id: u64) -> Result<u64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBestBid(market_id, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to get best bid".into()))
    }

    pub async fn best_ask(&self, market_id: u64) -> Result<u64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBestAsk(market_id, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to get best ask".into()))
    }

    pub async fn get_orderbook(&self, market_id: u64) -> Result<OrderbookSnapshot, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetOrderBook(market_id, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to get orderbook".into()))
    }

    pub async fn get_event_orderbooks(
        &self,
        event_id: u64,
    ) -> Result<EventOrderbookSnapshot, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::GetOrderbooksByEvent(event_id, tx))
            .await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to get event orderbooks".into()))
    }

    pub async fn get_outcome_orderbooks(
        &self,
        outcome_id: u64,
    ) -> Result<OutcomeOrderbookSnapshot, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::GetOrderbooksByOutcome(outcome_id, tx))
            .await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to get outcome orderbooks".into()))
    }

    pub async fn get_user_open_orders(&self, user_id: u64) -> Result<Vec<Order>, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetUserOpenOrders(user_id, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to get user orders".into()))
    }

    pub async fn get_order_status(&self, order_id: u64) -> Result<Order, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetOrderStatus(order_id, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to get order status".into()))
    }

    pub async fn add_user(&self, user: User) -> Option<User> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::AddUser(user, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn get_user_by_email(&self, email: String) -> Option<User> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetUserByEmail(email, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn get_user_by_id(&self, id: u64) -> Option<User> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetUserById(id, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn get_balance(&self, id: u64) -> Result<i64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBalance(id, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("failed to get balance".into()))
    }

    pub async fn update_balance(&self, id: u64, amount: i64) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::UpdateBalance(id, amount, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("failed to update balance".into()))
    }

    pub async fn get_position(&self, user_id: u64, market_id: u64) -> Result<u64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::GetPosition(user_id, market_id, tx))
            .await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to get positions".into()))
    }

    pub async fn get_user_positions(&self, user_id: u64) -> Result<HashMap<u64, u64>, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetUserPositions(user_id, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to get user positions".into()))
    }

    pub async fn update_position(
        &self,
        user_id: u64,
        market_id: u64,
        amount: i64,
    ) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::UpdatePosition(user_id, market_id, amount, tx))
            .await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to update position".into()))
    }

    pub async fn check_position_sufficient(
        &self,
        user_id: u64,
        market_id: u64,
        required_qty: u64,
    ) -> Result<bool, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::CheckPositionSufficient(
                user_id,
                market_id,
                required_qty,
                tx,
            ))
            .await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to check position".into()))
    }

    pub async fn create_split_postion(
        &self,
        user_id: u64,
        market1_id: u64,
        market2_id: u64,
        amount: u64,
    ) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::CreateSplitPosition(
                user_id, market1_id, market2_id, amount, tx,
            ))
            .await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to create split position".into()))
    }

    pub async fn merge_position(
        &self,
        user_id: u64,
        market1_id: u64,
        market2_id: u64,
    ) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::MergePosition(user_id, market1_id, market2_id, tx))
            .await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to merge position".into()))
    }

    pub async fn init_markets(&self, metas: Vec<MarketMeta>) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::InitMarkets(metas, tx)).await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to init markets".into()))
    }

    pub async fn close_event_markets(
        &self,
        event_id: u64,
        winning_outcome_id: u64,
    ) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::CloseEventMarkets(event_id, winning_outcome_id, tx))
            .await;
        rx.await
            .unwrap_or_else(|_| Err("Failed to close event markets".into()))
    }
}
