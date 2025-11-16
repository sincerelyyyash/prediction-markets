mod store;
mod types;

use store::orderbook;
use store::user;

fn main() {
    let user_store = user::spawn_user_actor();
    let _orderbook = orderbook::spawn_orderbook_actor(user_store);

    println!("Engine services ready");
}
