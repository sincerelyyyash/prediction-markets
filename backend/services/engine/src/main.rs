mod store;
mod types;

use store::orderbook;
use store::user;

fn main() {
    let _orderbook = orderbook::spawn_orderbook_actor as fn() -> orderbook::Orderbook;
    let _user_store = user::spawn_user_actor as fn() -> user::UserStore;

    println!("Engine services ready");
}
