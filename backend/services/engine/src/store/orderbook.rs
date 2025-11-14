use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, oneshot};
use std::collections::BTreeMap;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookData {
    pub market_id: u64,
    pub asks: BTreeMap<u64, u64>,
    pub bids: BTreeMap<u64, u64>,
    pub ask_queue: HashMap<u64, Vec<u64>>,
    pub bid_queue: HashMap<u64, Vec<u64>>,
    pub orders: HashMap<u64, Order>,  
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum OrderSide{
    Ask,
    Bid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSnapshot {
    pub market_id: u64,
    pub bids: Vec<Level>, 
    pub asks: Vec<Level>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub price: u64,
    pub quantity: u64,
}


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

pub fn spawn_orderbook_actor() -> Orderbook {
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

                    book.orders.insert(id, order.clone());

                    //TODO: Add matching logic

                    match order.side {
                        OrderSide::Bid => {
                            book.bid_queue.entry(order.price).or_default().push(id);
                            *book.bids.entry(order.price).or_default() += order.original_qty;
                        }
                        OrderSide::Ask => {
                            book.ask_queue.entry(order.price).or_default().push(id);
                            *book.asks.entry(order.price).or_default() += order.original_qty;
                        }
                    }

                    let _ = reply.send(Ok(order));
                },
                Command::CancelOrder(market_id, order_id,reply )=>{
                    let book = orderbooks.get_mut(&market_id).unwrap();

                    let Some(order) = book.orders.remove(&order_id) else {
                        let _ = reply.send(Err("Order not found".into()));
                        continue;
                    };

                    let price = order.price;
                    let qty = order.remaining_qty;

                    let queue_map = match order.side {
                        OrderSide::Ask =>  &mut book.ask_queue,
                        OrderSide::Bid =>  &mut book.bid_queue,
                    };

                    if let Some(queue) = queue_map.get_mut(&price) {
                        queue.retain(|&id| id != order_id);
                        if queue.is_empty() {
                            queue_map.remove(&price);
                        }
                    }

                    let map = match order.side {
                        OrderSide::Ask => &mut book.asks,
                        OrderSide::Bid => &mut book.bids,
                    };

                    if let Some(level_qty) = map.get_mut(&price) {
                        *level_qty -=qty;
                        if *level_qty ==0 {
                            map.remove(&price);
                        }
                    }

                    let _ = reply.send(Ok(order));
                },
                Command::ModifyOrder(order, reply ) => {

                },
                Command::GetBestBid(market_id, reply) => {

                },
                Command::GetBestAsk(market_id, reply) => {

                },
                Command::GetOrderBook(market_id,reply )=>{

                },
                Command::GetUserOpenOrders(user_id,reply )=>{

                },
                Command::GetOrderStatus(order_id,reply )=>{

                },
            }
        }
    });

    Orderbook { tx }
}