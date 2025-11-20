mod store;
mod types;

use store::market;
use store::orderbook;

fn main() {
    let market_store = market::MarketStore::new();
    let _orderbook = orderbook::spawn_orderbook_actor(market_store);

    println!("Engine services ready");
}
