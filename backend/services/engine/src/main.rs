mod store;

use store::orderbook;
use store::user_store;

fn main() {
    let _orderbook = orderbook::spawn_orderbook_actor as fn() -> orderbook::Orderbook;
    let _user_store = user_store::spawn_user_actor as fn() -> user_store::UserStore;

    println!("Engine services ready");
}
