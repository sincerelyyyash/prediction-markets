use tokio::sync::{mpsc, oneshot};
use std::collections::BTreeMap;
use std::collections::HashMap;
use uuid::Uuid;

use crate::store::balance::reserve_balance;
use crate::store::balance::return_reserved_balance;
use crate::store::balance::return_unused_reservation;
use crate::store::market::MarketStore;
use crate::store::matching::match_order;
use crate::store::orderbook_actions::add_order_to_book;
use crate::store::orderbook_actions::remove_order_from_book;
use crate::store::user::UserStore;
use crate::types::orderbook_types::{
    OrderbookData, Order, OrderbookSnapshot, Level
};


#[derive(Debug)]
enum Command {
    PlaceOrder(Order, oneshot::Sender<Result<Order, String>>),
    CancelOrder(u64, u64, oneshot::Sender<Result<Order, String>>),
    ModifyOrder(Order, oneshot::Sender<Result<Order, String>>),
    
    GetBestBid(u64, oneshot::Sender<Result<u64, String>>),
    GetBestAsk(u64, oneshot::Sender<Result<u64, String>>),
    GetOrderBook(u64, oneshot::Sender<Result<OrderbookSnapshot, String>>),
    // GetOrderBookDepth(u64, u64, oneshot::Sender<u64>),
    
    GetUserOpenOrders(u64, oneshot::Sender<Result<Vec<Order>, String>>),
    GetOrderStatus(u64, oneshot::Sender<Result<Order , String>>),

}

#[derive(Clone)]
pub struct Orderbook {
    tx: mpsc::Sender<Command>,
}

impl Orderbook {
    pub async fn place_order(&self, order: Order) -> Result<Order, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::PlaceOrder(order, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to place order".into()))
    }

    pub async fn cancel_order(&self, market_id: u64, order_id: u64) -> Result<Order, String> {
        let(tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::CancelOrder(market_id, order_id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to cancel order".into()))
    }

    pub async fn modify_order(&self, order: Order) -> Result<Order, String> {
        let(tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::ModifyOrder(order, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to modify order".into()))
    }

    pub async fn best_bid(&self, market_id: u64) -> Result<u64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBestBid(market_id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to get best bid".into()))
    }

    pub async fn best_ask(&self, market_id: u64) -> Result<u64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBestAsk(market_id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to get best ask".into()))
    }

    pub async fn get_orderbook(&self, market_id: u64) -> Result<OrderbookSnapshot, String> {
        let(tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetOrderBook(market_id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to get orderbook".into()))
    }

    pub async fn get_user_open_orders(&self, user_id: u64) -> Result<Vec<Order>, String> {
        let(tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetUserOpenOrders(user_id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to get user orders".into()))
    }

    pub async fn get_order_status(&self, order_id: u64) -> Result<Order , String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetOrderStatus(order_id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to get order status".into()))
    }

}

pub fn spawn_orderbook_actor(user_store: UserStore, market_store: MarketStore) -> Orderbook {
    let (tx, mut rx) = mpsc::channel::<Command>(1000);

    tokio::spawn(async move {
        let mut orderbooks: HashMap<u64, OrderbookData> = HashMap::new();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::PlaceOrder(mut order,reply )=> {
                    let book = orderbooks.entry(order.market_id).or_insert_with(|| OrderbookData {
                        market_id: order.market_id,
                        asks: BTreeMap::new(),
                        bids: BTreeMap::new(),
                        ask_queue: HashMap::new(),
                        bid_queue: HashMap::new(),
                        orders: HashMap::new(),
                     });
                    
                    let id = Uuid::new_v4().as_u128() as u64;
                    order.order_id = Some(id);

                    if let Err(e) = reserve_balance(&order, &user_store).await {
                        let _ = reply.send(Err(e));
                        continue;
                    }

                    if let Err(e) = match_order(&mut order, book, &user_store, &market_store).await {
                        let _ = return_reserved_balance(&order, &user_store).await;
                        let _ = reply.send(Err(e));
                        continue;
                    }

                    if order.remaining_qty > 0 {
                        add_order_to_book(id, &order, book);
                    } else {
                        let _ = return_unused_reservation(&order, &user_store).await;
                    }

                    let _ = reply.send(Ok(order));
                },
                Command::CancelOrder(market_id, order_id,reply )=>{
                    let Some(book) = orderbooks.get_mut(&market_id) else{
                        let _ = reply.send(Err("Market not found".into()));
                        continue;
                    };

                    let Some(order) = book.orders.get(&order_id).cloned() else {
                        let _ = reply.send(Err("Order not found".into()));
                        continue;
                    };

                    remove_order_from_book(order_id, &order, book);

                    let _ = return_reserved_balance(&order, &user_store).await;

                    let _ =reply.send(Ok(order));

                },
                Command::ModifyOrder(mut order, reply ) => {
                    let Some(book) = orderbooks.get_mut(&order.market_id) else {
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

                   remove_order_from_book(order_id, &existing_order, book);

                   let _ = return_reserved_balance(&existing_order, &user_store).await;

                   order.order_id = Some(order_id);
                   order.remaining_qty = order.original_qty;

                   if let Err(e) = reserve_balance(&order, &user_store).await {
                    let _ = reply.send(Err(e));
                    continue;
                }

                   if let Err(e) = match_order(&mut order, book, &user_store, &market_store).await {
                    let _ = return_reserved_balance(&order, &user_store).await;
                    let _ = reply.send(Err(e));
                    continue;
                   }

                   if order.remaining_qty > 0 {
                    add_order_to_book(order_id, &order, book);
                   } else {
                    let _ = return_unused_reservation(&order, &user_store).await;
                   }

                   let _ = reply.send(Ok(order));

                },
                Command::GetBestBid(market_id, reply) => {
                    let Some(book) = orderbooks.get(&market_id) else {
                        let _ = reply.send(Err("Market not found".into()));
                        continue;
                    };

                    let Some((best_bid_price, _)) = book.bids.last_key_value() else {
                        let _ = reply.send(Err("No bids available".into()));
                        continue;
                    };

                    let _ = reply.send(Ok(*best_bid_price));
                },
                Command::GetBestAsk(market_id, reply) => {
                    let Some(book) = orderbooks.get(&market_id) else {
                        let _ = reply.send(Err("Market not found".into()));
                        continue;
                    };

                    let Some((best_ask_price, _)) = book.asks.first_key_value() else {
                        let _ = reply.send(Err(" No asks available".into()));
                        continue;
                    };

                    let _ = reply.send(Ok(*best_ask_price));

                },
                Command::GetOrderBook(market_id,reply )=>{

                    let Some(book) = orderbooks.get(&market_id) else {
                        let _ = reply.send(Err("Market not found".into()));
                        continue;
                    };

                    let bids: Vec<Level> = book.bids
                    .iter()
                    .rev()
                    .map(|(price, quantity)| Level{
                        price: *price,
                        quantity: *quantity,
                    })
                    .collect();

                let asks: Vec<Level> = book.asks
                .iter()
                .map(|(price, quantity)| Level {
                    price: *price,
                    quantity: *quantity,
                })
                .collect();
                
                let snapshot = OrderbookSnapshot {
                    market_id,
                    bids,
                    asks,
                };

                let _ = reply.send(Ok(snapshot));

                },
                Command::GetUserOpenOrders(user_id,reply )=>{
                    let mut user_orders = Vec::new();

                    for book in orderbooks.values(){
                        for order in book.orders.values(){
                            if order.user_id == user_id {
                                user_orders.push(order.clone());
                            }
                        }
                    }
                    
                    let _ = reply.send(Ok(user_orders));
                },
                Command::GetOrderStatus(order_id, reply ) => {
                    let mut found_order: Option<Order> = None;

                    for book in orderbooks.values() {
                        if let Some(order) = book.orders.get(&order_id) {
                            found_order = Some(order.clone());
                            break;
                        }
                    }

                    match found_order {
                        Some(order) => {
                            let _ = reply.send(Ok(order));
                        }
                        None => {
                            let _ = reply.send(Err("Order not found".into()));
                        }
                    }
                },
            }
        }
    });

    Orderbook { tx }
}