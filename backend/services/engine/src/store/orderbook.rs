use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, oneshot};
use std::collections::BTreeMap;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookData {
    pub market_id: u64,
    pub asks: BTreeMap<u64, u64>,
    pub bids: BTreeMap<u64, u64>,
}

#[derive(Debug)]
enum Command {
    AddAsk(u64, u64, u64, oneshot::Sender<Option<OrderbookData>>),
    AddBid(u64, u64, u64, oneshot::Sender<Option<OrderbookData>>),
    UpdateBid(u64, u64, u64, oneshot::Sender<Option<OrderbookData>>),
    UpdateAsk(u64, u64, u64, oneshot::Sender<Option<OrderbookData>>),
    GetBestBid(u64, oneshot::Sender<Result<u64, String>>),
    GetBestAsk(u64, oneshot::Sender<Result<u64, String>>),
}

#[derive(Clone)]
pub struct Orderbook {
    tx: mpsc::Sender<Command>,
}

impl Orderbook {
    pub async fn add_ask(&self, market_id: u64, price: u64, quantity: u64) -> Option<OrderbookData> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::AddAsk(market_id, price, quantity, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn add_bid(&self, market_id: u64, price: u64, quantity: u64) -> Option<OrderbookData> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::AddBid(market_id, price, quantity, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn update_ask(&self, market_id: u64, price: u64, quantity: u64) -> Option<OrderbookData> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::UpdateAsk(market_id, price, quantity, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn update_bid(&self, market_id: u64, price: u64, quantity: u64) -> Option<OrderbookData> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::UpdateBid(market_id, price, quantity, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn best_bid(&self, market_id: u64) -> Result<u64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBestBid(market_id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("failed to get best bid".into()))
    }

    pub async fn best_ask(&self, market_id: u64) -> Result<u64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBestAsk(market_id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("failed to get best ask".into()))
    }
}

pub fn spawn_orderbook_actor() -> Orderbook {
    let (tx, mut rx) = mpsc::channel::<Command>(1000);

    tokio::spawn(async move {
        let mut orderbooks: HashMap<u64, OrderbookData> = HashMap::new();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::AddAsk(market_id, price, quantity, reply) => {

                }
                Command::AddBid(market_id, price, quantity, reply) => {

                }
                Command::UpdateBid(market_id, price, quantity, reply) => {

                }
                Command::UpdateAsk(market_id, price, quantity, reply) => {

                }
                Command::GetBestBid(market_id, reply) => {

                }
                Command::GetBestAsk(market_id, reply) => {

                }
            }
        }
    });

    Orderbook { tx }
}