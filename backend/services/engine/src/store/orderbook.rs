use chrono::Utc;
use std::collections::BTreeMap;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::services::db_event_publisher::publish_db_event;
use crate::store::balance::reserve_balance;
use crate::store::balance::return_reserved_balance;
use crate::store::balance::return_unused_reservation;
use crate::store::market::MarketStore;
use crate::store::matching::match_order;
use crate::store::orderbook_actions::add_order_to_book;
use crate::store::orderbook_actions::remove_order_from_book;
use crate::types::db_event_types::{
    BalanceUpdatedEvent, DbEvent, OrderCancelledEvent, OrderModifiedEvent, OrderPlacedEvent,
    PositionUpdatedEvent,
};
use crate::types::market_types::MarketMeta;
use crate::types::orderbook_types::{
    EventOrderbookSnapshot, Level, MarketOrderbookSnapshot, Order, OrderSide, OrderbookData,
    OrderbookSnapshot, OutcomeOrderbookSnapshot,
};
use crate::types::user_types::User;

#[derive(Debug)]
enum Command {
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
    // GetOrderBookDepth(u64, u64, oneshot::Sender<u64>),
    GetUserOpenOrders(u64, oneshot::Sender<Result<Vec<Order>, String>>),
    GetOrderStatus(u64, oneshot::Sender<Result<Order, String>>),
    AddUser(User, oneshot::Sender<Option<User>>),
    GetUserByEmail(String, oneshot::Sender<Option<User>>),
    GetUserById(u64, oneshot::Sender<Option<User>>),
    GetBalance(u64, oneshot::Sender<Result<i64, String>>),
    UpdateBalance(u64, i64, oneshot::Sender<Result<(), String>>),
    GetPosition(u64, u64, oneshot::Sender<Result<u64, String>>),
    GetUserPositions(u64, oneshot::Sender<Result<HashMap<u64, u64>, String>>),
    UpdatePosition(u64, u64, i64, oneshot::Sender<Result<(), String>>),
    CheckPositionSufficient(u64, u64, u64, oneshot::Sender<Result<bool, String>>),
    CreateSplitPosition(u64, u64, u64, u64, oneshot::Sender<Result<(), String>>),
    MergePosition(u64, u64, u64, oneshot::Sender<Result<(), String>>),
    InitMarkets(Vec<MarketMeta>, oneshot::Sender<Result<(), String>>),
    CloseEventMarkets(u64, u64, oneshot::Sender<Result<(), String>>),
}

#[derive(Clone)]
pub struct Orderbook {
    tx: mpsc::Sender<Command>,
}

impl Orderbook {
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

fn normalize_order(order: &mut Order, market_store: &MarketStore) -> Result<u64, String> {
    let Some(market) = market_store.get_market(order.market_id) else {
        return Err("Market not found".into());
    };

    if order.price > 100 {
        return Err("Price must be between 0 and 100".into());
    }

    if let Some(side) = &market.side {
        use crate::types::market_types::MarketSide;
        match side {
            MarketSide::No => {
                if let Some(paired_id) = market.paired_market_id {
                    order.market_id = paired_id;
                    order.price = 100 - order.price;
                    order.side = match order.side {
                        OrderSide::Bid => OrderSide::Ask,
                        OrderSide::Ask => OrderSide::Bid,
                    };
                }
            }
            MarketSide::Yes => {}
        }
    }

    Ok(order.market_id)
}

fn denormalize_price(market_id: u64, canonical_price: u64, market_store: &MarketStore) -> u64 {
    let Some(market) = market_store.get_market(market_id) else {
        return canonical_price;
    };

    if let Some(side) = &market.side {
        use crate::types::market_types::MarketSide;
        match side {
            MarketSide::No => 100 - canonical_price,
            MarketSide::Yes => canonical_price,
        }
    } else {
        canonical_price
    }
}

pub fn spawn_orderbook_actor(market_store: MarketStore) -> Orderbook {
    let (tx, mut rx) = mpsc::channel::<Command>(1000);

    tokio::spawn(async move {
        let mut orderbooks: HashMap<u64, OrderbookData> = HashMap::new();
        let mut users: HashMap<u64, User> = HashMap::new();
        let mut alias_map: HashMap<u64, u64> = HashMap::new();
        let mut order_original_market: HashMap<u64, u64> = HashMap::new();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::PlaceOrder(mut order, reply) => {
                    let original_market_id = order.market_id;
                    let original_price = order.price;
                    let original_side = order.side.clone();

                    let canonical_market_id = match normalize_order(&mut order, &market_store) {
                        Ok(id) => id,
                        Err(e) => {
                            let _ = reply.send(Err(e));
                            continue;
                        }
                    };

                    let Some(book) = orderbooks.get_mut(&canonical_market_id) else {
                        let _ = reply.send(Err("Orderbook not found for market".into()));
                        continue;
                    };

                    let id = Uuid::new_v4().as_u128() as u64;
                    order.order_id = Some(id);
                    order_original_market.insert(id, original_market_id);

                    if let Err(e) = reserve_balance(&order, &mut users).await {
                        let _ = reply.send(Err(e));
                        continue;
                    }

                    if let Err(e) = match_order(&mut order, book, &mut users, &market_store).await {
                        let _ = return_reserved_balance(&order, &mut users).await;
                        order_original_market.remove(&id);
                        let _ = reply.send(Err(e));
                        continue;
                    }

                    if order.remaining_qty > 0 {
                        add_order_to_book(id, &order, book);
                    } else {
                        let _ = return_unused_reservation(&order, &mut users).await;
                        order_original_market.remove(&id);
                    }

                    let side_str = match original_side {
                        OrderSide::Bid => "Bid",
                        OrderSide::Ask => "Ask",
                    };
                    let _ = publish_db_event(DbEvent::OrderPlaced(OrderPlacedEvent {
                        order_id: id,
                        user_id: order.user_id,
                        market_id: original_market_id,
                        side: side_str.to_string(),
                        price: original_price,
                        original_qty: order.original_qty,
                        remaining_qty: order.remaining_qty,
                        timestamp: Utc::now(),
                    }))
                    .await;

                    let mut response_order = order.clone();
                    response_order.market_id = original_market_id;
                    response_order.price = original_price;
                    response_order.side = original_side;
                    let _ = reply.send(Ok(response_order));
                }
                Command::CancelOrder(market_id, order_id, reply) => {
                    let original_market_id = order_original_market
                        .get(&order_id)
                        .copied()
                        .unwrap_or(market_id);
                    let canonical_id = alias_map
                        .get(&original_market_id)
                        .copied()
                        .unwrap_or(original_market_id);
                    let Some(book) = orderbooks.get_mut(&canonical_id) else {
                        let _ = reply.send(Err("Market not found".into()));
                        continue;
                    };

                    let Some(order) = book.orders.get(&order_id).cloned() else {
                        let _ = reply.send(Err("Order not found".into()));
                        continue;
                    };

                    order_original_market.remove(&order_id);
                    let original_price =
                        denormalize_price(original_market_id, order.price, &market_store);
                    let original_side =
                        if let Some(market) = market_store.get_market(original_market_id) {
                            if let Some(side) = &market.side {
                                use crate::types::market_types::MarketSide;
                                match side {
                                    MarketSide::No => match order.side {
                                        OrderSide::Bid => OrderSide::Ask,
                                        OrderSide::Ask => OrderSide::Bid,
                                    },
                                    MarketSide::Yes => order.side.clone(),
                                }
                            } else {
                                order.side.clone()
                            }
                        } else {
                            order.side.clone()
                        };

                    remove_order_from_book(order_id, &order, book);

                    let _ = return_reserved_balance(&order, &mut users).await;

                    let _ = publish_db_event(DbEvent::OrderCancelled(OrderCancelledEvent {
                        order_id,
                        user_id: order.user_id,
                        market_id: original_market_id,
                        timestamp: Utc::now(),
                    }))
                    .await;

                    let mut response_order = order;
                    response_order.market_id = original_market_id;
                    response_order.price = original_price;
                    response_order.side = original_side;
                    let _ = reply.send(Ok(response_order));
                }
                Command::ModifyOrder(mut order, reply) => {
                    let original_market_id = order.market_id;
                    let original_price = order.price;
                    let original_side = order.side.clone();

                    let canonical_market_id = match normalize_order(&mut order, &market_store) {
                        Ok(id) => id,
                        Err(e) => {
                            let _ = reply.send(Err(e));
                            continue;
                        }
                    };

                    let Some(book) = orderbooks.get_mut(&canonical_market_id) else {
                        let _ = reply.send(Err("Market not found".into()));
                        continue;
                    };

                    let Some(order_id) = order.order_id else {
                        let _ = reply.send(Err("Order id not found".into()));
                        continue;
                    };

                    let Some(existing_order) = book.orders.get(&order_id).cloned() else {
                        let _ = reply.send(Err("Order not found".into()));
                        continue;
                    };

                    let old_original_market_id = order_original_market.get(&order_id).copied();

                    remove_order_from_book(order_id, &existing_order, book);

                    let _ = return_reserved_balance(&existing_order, &mut users).await;

                    order.order_id = Some(order_id);
                    order.remaining_qty = order.original_qty;
                    if let Some(old_market) = old_original_market_id {
                        order_original_market.insert(order_id, old_market);
                    }

                    if let Err(e) = reserve_balance(&order, &mut users).await {
                        let _ = reply.send(Err(e));
                        continue;
                    }

                    if let Err(e) = match_order(&mut order, book, &mut users, &market_store).await {
                        let _ = return_reserved_balance(&order, &mut users).await;
                        let _ = reply.send(Err(e));
                        continue;
                    }

                    if order.remaining_qty > 0 {
                        add_order_to_book(order_id, &order, book);
                    } else {
                        let _ = return_unused_reservation(&order, &mut users).await;
                    }

                    let _ = publish_db_event(DbEvent::OrderModified(OrderModifiedEvent {
                        order_id,
                        user_id: order.user_id,
                        market_id: original_market_id,
                        price: original_price,
                        original_qty: order.original_qty,
                        remaining_qty: order.remaining_qty,
                        timestamp: Utc::now(),
                    }))
                    .await;

                    let mut response_order = order;
                    response_order.market_id = original_market_id;
                    response_order.price = original_price;
                    response_order.side = original_side;
                    let _ = reply.send(Ok(response_order));
                }
                Command::GetBestBid(market_id, reply) => {
                    let canonical_id = alias_map.get(&market_id).copied().unwrap_or(market_id);
                    let Some(book) = orderbooks.get(&canonical_id) else {
                        let _ = reply.send(Err("Market not found".into()));
                        continue;
                    };

                    let Some((best_bid_price, _)) = book.bids.last_key_value() else {
                        let _ = reply.send(Err("No bids available".into()));
                        continue;
                    };

                    let denormalized_price =
                        denormalize_price(market_id, *best_bid_price, &market_store);
                    let _ = reply.send(Ok(denormalized_price));
                }
                Command::GetBestAsk(market_id, reply) => {
                    let canonical_id = alias_map.get(&market_id).copied().unwrap_or(market_id);
                    let Some(book) = orderbooks.get(&canonical_id) else {
                        let _ = reply.send(Err("Market not found".into()));
                        continue;
                    };

                    let Some((best_ask_price, _)) = book.asks.first_key_value() else {
                        let _ = reply.send(Err(" No asks available".into()));
                        continue;
                    };

                    let denormalized_price =
                        denormalize_price(market_id, *best_ask_price, &market_store);
                    let _ = reply.send(Ok(denormalized_price));
                }
                Command::GetOrderBook(market_id, reply) => {
                    match build_orderbook_snapshot(
                        market_id,
                        &alias_map,
                        &orderbooks,
                        &market_store,
                    ) {
                        Ok(snapshot) => {
                            let _ = reply.send(Ok(snapshot));
                        }
                        Err(e) => {
                            let _ = reply.send(Err(e));
                        }
                    }
                }
                Command::GetOrderbooksByEvent(event_id, reply) => {
                    let market_ids = market_store.get_markets_by_event(event_id);
                    if market_ids.is_empty() {
                        let _ = reply.send(Err("No markets found for event".into()));
                        continue;
                    }

                    let mut outcomes: HashMap<u64, OutcomeOrderbookSnapshot> = HashMap::new();
                    let mut error: Option<String> = None;

                    for market_id in market_ids {
                        let Some(market_meta) = market_store.get_market(market_id) else {
                            continue;
                        };
                        let Some(outcome_id) = market_meta.outcome_id else {
                            continue;
                        };

                        match build_orderbook_snapshot(
                            market_id,
                            &alias_map,
                            &orderbooks,
                            &market_store,
                        ) {
                            Ok(snapshot) => {
                                let entry = outcomes.entry(outcome_id).or_insert_with(|| {
                                    OutcomeOrderbookSnapshot {
                                        outcome_id,
                                        event_id: market_meta.event_id,
                                        markets: Vec::new(),
                                    }
                                });
                                entry.markets.push(MarketOrderbookSnapshot {
                                    market_id,
                                    side: market_meta.side.clone(),
                                    snapshot,
                                });
                            }
                            Err(e) => {
                                error = Some(e);
                                break;
                            }
                        }
                    }

                    if let Some(err) = error {
                        let _ = reply.send(Err(err));
                        continue;
                    }

                    if outcomes.is_empty() {
                        let _ = reply.send(Err("No orderbooks found for event".into()));
                        continue;
                    }

                    for outcome in outcomes.values_mut() {
                        outcome.markets.sort_by_key(|m| m.market_id);
                    }

                    let mut ordered: Vec<OutcomeOrderbookSnapshot> =
                        outcomes.into_values().collect();
                    ordered.sort_by_key(|o| o.outcome_id);

                    let response = EventOrderbookSnapshot {
                        event_id,
                        outcomes: ordered,
                    };

                    let _ = reply.send(Ok(response));
                }
                Command::GetOrderbooksByOutcome(outcome_id, reply) => {
                    let market_ids = market_store.get_markets_by_outcome(outcome_id);
                    if market_ids.is_empty() {
                        let _ = reply.send(Err("No markets found for outcome".into()));
                        continue;
                    }

                    let mut markets: Vec<MarketOrderbookSnapshot> = Vec::new();
                    let mut event_id_for_outcome: Option<u64> = None;
                    let mut error: Option<String> = None;

                    for market_id in market_ids {
                        let Some(market_meta) = market_store.get_market(market_id) else {
                            continue;
                        };
                        if event_id_for_outcome.is_none() {
                            event_id_for_outcome = market_meta.event_id;
                        }

                        match build_orderbook_snapshot(
                            market_id,
                            &alias_map,
                            &orderbooks,
                            &market_store,
                        ) {
                            Ok(snapshot) => {
                                markets.push(MarketOrderbookSnapshot {
                                    market_id,
                                    side: market_meta.side.clone(),
                                    snapshot,
                                });
                            }
                            Err(e) => {
                                error = Some(e);
                                break;
                            }
                        }
                    }

                    if let Some(err) = error {
                        let _ = reply.send(Err(err));
                        continue;
                    }

                    if markets.is_empty() {
                        let _ = reply.send(Err("No orderbooks found for outcome".into()));
                        continue;
                    }

                    markets.sort_by_key(|m| m.market_id);

                    let response = OutcomeOrderbookSnapshot {
                        outcome_id,
                        event_id: event_id_for_outcome,
                        markets,
                    };

                    let _ = reply.send(Ok(response));
                }
                Command::GetUserOpenOrders(user_id, reply) => {
                    let mut user_orders = Vec::new();

                    for (_canonical_market_id, book) in &orderbooks {
                        for order in book.orders.values() {
                            if order.user_id == user_id {
                                if let Some(order_id) = order.order_id {
                                    if let Some(original_market_id) =
                                        order_original_market.get(&order_id)
                                    {
                                        let mut denormalized_order = order.clone();
                                        if let Some(market) =
                                            market_store.get_market(*original_market_id)
                                        {
                                            if let Some(side) = &market.side {
                                                use crate::types::market_types::MarketSide;
                                                match side {
                                                    MarketSide::No => {
                                                        denormalized_order.market_id =
                                                            *original_market_id;
                                                        denormalized_order.price =
                                                            100 - denormalized_order.price;
                                                        denormalized_order.side =
                                                            match denormalized_order.side {
                                                                OrderSide::Bid => OrderSide::Ask,
                                                                OrderSide::Ask => OrderSide::Bid,
                                                            };
                                                    }
                                                    MarketSide::Yes => {
                                                        denormalized_order.market_id =
                                                            *original_market_id;
                                                    }
                                                }
                                            } else {
                                                denormalized_order.market_id = *original_market_id;
                                            }
                                        } else {
                                            denormalized_order.market_id = *original_market_id;
                                        }
                                        user_orders.push(denormalized_order);
                                    }
                                }
                            }
                        }
                    }

                    let _ = reply.send(Ok(user_orders));
                }
                Command::GetOrderStatus(order_id, reply) => {
                    let mut found_order: Option<Order> = None;

                    for (_canonical_id, book) in &orderbooks {
                        if let Some(order) = book.orders.get(&order_id) {
                            found_order = Some(order.clone());
                            break;
                        }
                    }

                    match found_order {
                        Some(mut order) => {
                            if let Some(original_market_id) = order_original_market.get(&order_id) {
                                if let Some(market) = market_store.get_market(*original_market_id) {
                                    if let Some(side) = &market.side {
                                        use crate::types::market_types::MarketSide;
                                        match side {
                                            MarketSide::No => {
                                                order.market_id = *original_market_id;
                                                order.price = 100 - order.price;
                                                order.side = match order.side {
                                                    OrderSide::Bid => OrderSide::Ask,
                                                    OrderSide::Ask => OrderSide::Bid,
                                                };
                                            }
                                            MarketSide::Yes => {
                                                order.market_id = *original_market_id;
                                            }
                                        }
                                    } else {
                                        order.market_id = *original_market_id;
                                    }
                                } else {
                                    order.market_id = *original_market_id;
                                }
                            }
                            let _ = reply.send(Ok(order));
                        }
                        None => {
                            let _ = reply.send(Err("Order not found".into()));
                        }
                    }
                }
                Command::AddUser(user, reply) => {
                    let id = user.id;
                    users.insert(id, user.clone());
                    let _ = reply.send(Some(user));
                }
                Command::GetUserById(id, reply) => {
                    let user = users.get(&id).cloned();
                    let _ = reply.send(user);
                }
                Command::GetUserByEmail(email, reply) => {
                    let user = users.values().find(|u| u.email == email).cloned();
                    let _ = reply.send(user);
                }
                Command::GetBalance(id, reply) => {
                    let res = users
                        .get(&id)
                        .map(|u| Ok(u.balance))
                        .unwrap_or_else(|| Err("User not found".into()));
                    let _ = reply.send(res);
                }
                Command::UpdateBalance(id, amount, reply) => {
                    if let Some(u) = users.get_mut(&id) {
                        u.balance += amount;
                        let _ = publish_db_event(DbEvent::BalanceUpdated(BalanceUpdatedEvent {
                            user_id: id,
                            balance: u.balance,
                            timestamp: Utc::now(),
                        }))
                        .await;
                        let _ = reply.send(Ok(()));
                    } else {
                        let _ = reply.send(Err("User not found".into()));
                    }
                }
                Command::GetPosition(user_id, market_id, reply) => {
                    let position = users
                        .get(&user_id)
                        .and_then(|u| u.positions.get(&market_id))
                        .copied()
                        .unwrap_or(0);

                    let _ = reply.send(Ok(position));
                }
                Command::GetUserPositions(user_id, reply) => {
                    let positions = users
                        .get(&user_id)
                        .map(|u| u.positions.clone())
                        .unwrap_or_default();
                    let _ = reply.send(Ok(positions));
                }
                Command::UpdatePosition(user_id, market_id, amount, reply) => {
                    if let Some(user) = users.get_mut(&user_id) {
                        let current = user.positions.entry(market_id).or_insert(0);

                        if amount < 0 && (*current as i64) < -amount {
                            let _ = reply.send(Err("Insufficient position".into()));
                        } else {
                            *current = ((*current as i64) + amount) as u64;
                            let final_qty = if *current == 0 {
                                user.positions.remove(&market_id);
                                0
                            } else {
                                *current
                            };
                            let _ =
                                publish_db_event(DbEvent::PositionUpdated(PositionUpdatedEvent {
                                    user_id,
                                    market_id,
                                    quantity: final_qty,
                                    timestamp: Utc::now(),
                                }))
                                .await;
                            let _ = reply.send(Ok(()));
                        }
                    } else {
                        let _ = reply.send(Err("User not found".into()));
                    }
                }
                Command::CheckPositionSufficient(user_id, market_id, required_qty, reply) => {
                    let position = users
                        .get(&user_id)
                        .and_then(|u| u.positions.get(&market_id))
                        .copied()
                        .unwrap_or(0);

                    let _ = reply.send(Ok(position >= required_qty));
                }
                Command::CreateSplitPosition(user_id, market1_id, market2_id, amount, reply) => {
                    let Some(user) = users.get_mut(&user_id) else {
                        let _ = reply.send(Err("User not found".into()));
                        continue;
                    };

                    if user.balance < amount as i64 {
                        let _ = reply.send(Err("Insufficient balance".into()));
                        continue;
                    }

                    user.balance -= amount as i64;
                    *user.positions.entry(market1_id).or_insert(0) += amount;
                    *user.positions.entry(market2_id).or_insert(0) += amount;

                    let _ = publish_db_event(DbEvent::BalanceUpdated(BalanceUpdatedEvent {
                        user_id,
                        balance: user.balance,
                        timestamp: Utc::now(),
                    }))
                    .await;
                    let _ = publish_db_event(DbEvent::PositionUpdated(PositionUpdatedEvent {
                        user_id,
                        market_id: market1_id,
                        quantity: user.positions[&market1_id],
                        timestamp: Utc::now(),
                    }))
                    .await;
                    let _ = publish_db_event(DbEvent::PositionUpdated(PositionUpdatedEvent {
                        user_id,
                        market_id: market2_id,
                        quantity: user.positions[&market2_id],
                        timestamp: Utc::now(),
                    }))
                    .await;

                    let _ = reply.send(Ok(()));
                }
                Command::MergePosition(user_id, market1_id, market2_id, reply) => {
                    let Some(user) = users.get_mut(&user_id) else {
                        let _ = reply.send(Err("User not found".into()));
                        continue;
                    };

                    let position1 = user.positions.get(&market1_id).copied().unwrap_or(0);
                    let position2 = user.positions.get(&market2_id).copied().unwrap_or(0);

                    let merge_qty = position1.min(position2);
                    if merge_qty == 0 {
                        let _ = reply.send(Err("Cannot merge, Insufficient positions".into()));
                        continue;
                    }

                    user.positions
                        .entry(market1_id)
                        .and_modify(|p| *p -= merge_qty);
                    let pos1_final = if user.positions[&market1_id] == 0 {
                        user.positions.remove(&market1_id);
                        0
                    } else {
                        user.positions[&market1_id]
                    };

                    user.positions
                        .entry(market2_id)
                        .and_modify(|p| *p -= merge_qty);
                    let pos2_final = if user.positions[&market2_id] == 0 {
                        user.positions.remove(&market2_id);
                        0
                    } else {
                        user.positions[&market2_id]
                    };

                    let _ = publish_db_event(DbEvent::PositionUpdated(PositionUpdatedEvent {
                        user_id,
                        market_id: market1_id,
                        quantity: pos1_final,
                        timestamp: Utc::now(),
                    }))
                    .await;
                    let _ = publish_db_event(DbEvent::PositionUpdated(PositionUpdatedEvent {
                        user_id,
                        market_id: market2_id,
                        quantity: pos2_final,
                        timestamp: Utc::now(),
                    }))
                    .await;

                    let _ = reply.send(Ok(()));
                }
                Command::InitMarkets(metas, reply) => {
                    let mut error_msg: Option<String> = None;
                    for meta in &metas {
                        if let Err(e) = market_store.register_market_pair(meta.clone()) {
                            error_msg = Some(format!("Failed to register market pair: {}", e));
                            break;
                        }

                        alias_map.insert(meta.yes_market_id, meta.yes_market_id);
                        alias_map.insert(meta.no_market_id, meta.yes_market_id);

                        orderbooks.insert(
                            meta.yes_market_id,
                            OrderbookData {
                                market_id: meta.yes_market_id,
                                asks: BTreeMap::new(),
                                bids: BTreeMap::new(),
                                ask_queue: HashMap::new(),
                                bid_queue: HashMap::new(),
                                orders: HashMap::new(),
                                last_price: None,
                            },
                        );
                    }
                    match error_msg {
                        Some(e) => {
                            let _ = reply.send(Err(e));
                        }
                        None => {
                            let _ = reply.send(Ok(()));
                        }
                    }
                }
                Command::CloseEventMarkets(event_id, winning_outcome_id, reply) => {
                    let market_ids = market_store.get_markets_by_event(event_id);
                    let mut canonical_ids = std::collections::HashSet::new();

                    for market_id in &market_ids {
                        if let Some(canonical_id) = alias_map.get(market_id) {
                            canonical_ids.insert(*canonical_id);
                        }
                    }

                    for canonical_id in canonical_ids {
                        if let Some(book) = orderbooks.get(&canonical_id) {
                            for (order_id, order) in &book.orders {
                                let _ = return_reserved_balance(order, &mut users).await;
                                let original_market_id = order_original_market
                                    .get(order_id)
                                    .copied()
                                    .unwrap_or(order.market_id);
                                let _ = publish_db_event(DbEvent::OrderCancelled(
                                    OrderCancelledEvent {
                                        order_id: *order_id,
                                        user_id: order.user_id,
                                        market_id: original_market_id,
                                        timestamp: Utc::now(),
                                    },
                                ))
                                .await;
                                order_original_market.remove(order_id);
                            }
                        }
                        orderbooks.remove(&canonical_id);
                    }

                    for market_id in &market_ids {
                        alias_map.remove(market_id);
                    }

                    let market_ids_set: std::collections::HashSet<u64> =
                        market_ids.iter().cloned().collect();

                    for user in users.values_mut() {
                        let mut positions_to_remove = Vec::new();
                        let mut total_payout = 0i64;

                        for (market_id, quantity) in &user.positions.clone() {
                            if !market_ids_set.contains(market_id) {
                                continue;
                            }

                            if let Some(market) = market_store.get_market(*market_id) {
                                if let (Some(market_outcome_id), Some(side)) =
                                    (market.outcome_id, &market.side)
                                {
                                    use crate::types::market_types::MarketSide;
                                    let is_winning = match side {
                                        MarketSide::Yes => market_outcome_id == winning_outcome_id,
                                        MarketSide::No => market_outcome_id != winning_outcome_id,
                                    };

                                    if is_winning {
                                        let payout = (*quantity as i64) * 100;
                                        total_payout += payout;
                                    }
                                    positions_to_remove.push(*market_id);
                                }
                            }
                        }

                        if total_payout > 0 {
                            user.balance += total_payout;
                            let _ =
                                publish_db_event(DbEvent::BalanceUpdated(BalanceUpdatedEvent {
                                    user_id: user.id,
                                    balance: user.balance,
                                    timestamp: Utc::now(),
                                }))
                                .await;
                        }

                        for market_id in positions_to_remove {
                            user.positions.remove(&market_id);
                            let _ =
                                publish_db_event(DbEvent::PositionUpdated(PositionUpdatedEvent {
                                    user_id: user.id,
                                    market_id,
                                    quantity: 0,
                                    timestamp: Utc::now(),
                                }))
                                .await;
                        }
                    }

                    let _ = market_store.update_status_bulk(
                        market_ids.clone(),
                        crate::types::market_types::MarketStatus::Resolved,
                    );
                    let _ = market_store.remove_markets_by_event(event_id);
                    let _ = reply.send(Ok(()));
                }
            }
        }
    });

    Orderbook { tx }
}

fn build_orderbook_snapshot(
    market_id: u64,
    alias_map: &HashMap<u64, u64>,
    orderbooks: &HashMap<u64, OrderbookData>,
    market_store: &MarketStore,
) -> Result<OrderbookSnapshot, String> {
    let canonical_id = alias_map.get(&market_id).copied().unwrap_or(market_id);
    let book = orderbooks
        .get(&canonical_id)
        .ok_or_else(|| "Market not found".to_string())?;

    let bids: Vec<Level> = book
        .bids
        .iter()
        .rev()
        .map(|(price, quantity)| Level {
            price: denormalize_price(market_id, *price, market_store),
            quantity: *quantity,
        })
        .collect();

    let asks: Vec<Level> = book
        .asks
        .iter()
        .map(|(price, quantity)| Level {
            price: denormalize_price(market_id, *price, market_store),
            quantity: *quantity,
        })
        .collect();

    let last_price = book
        .last_price
        .map(|p| denormalize_price(market_id, p, market_store));

    Ok(OrderbookSnapshot {
        market_id,
        bids,
        asks,
        last_price,
    })
}
