mod store;
mod types;

use store::orderbook;
use store::user;
use store::market;

fn main() {
    let user_store = user::spawn_user_actor();
    let market_store = market::MarketStore::new();
    let _orderbook = orderbook::spawn_orderbook_actor(user_store, market_store);

    println!("Engine services ready");
}
