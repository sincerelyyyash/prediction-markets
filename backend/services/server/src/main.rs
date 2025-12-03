mod controllers;
mod middleware;
mod services;
mod types;
mod utils;

use crate::controllers::admin_auth_controller::signin_admin;
use crate::controllers::admin_event_controller::{
    create_event, delete_event, resolve_event, update_event,
};
use crate::controllers::order_controller::{
    cancel_order, get_open_orders, get_order_history, get_order_status, get_orders_by_market,
    get_orders_by_user, merge_order, modify_order, place_order, split_order,
};
use crate::controllers::orderbook_controller::{
    get_orderbook_by_market, get_orderbooks_by_event, get_orderbooks_by_outcome,
};
use crate::controllers::position_controller::{
    get_portfolio, get_position_by_market, get_positions, get_positions_history,
};
use crate::controllers::trade_controller::{
    get_trade_by_id, get_trades, get_trades_by_market, get_trades_by_user_id,
};
use crate::controllers::user_bookmark_controller::{
    add_market_bookmark, get_for_you_markets, get_user_bookmarks, remove_market_bookmark,
};
use crate::controllers::user_controller::{get_balance, onramp, signin_user, signup_user};
use crate::controllers::user_event_controller::{get_all_events, get_event_by_id, search_events};
use crate::controllers::user_profile_controller::{get_all_users, get_user_by_id};
use crate::middleware::admin::AdminMiddleware;
use crate::middleware::auth::AuthMiddleware;
use crate::services::db_read_response_consumer::start_db_read_response_consumer;
use crate::services::response_consumer::start_response_consumer;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use redis_client::RedisManager;
use std::env;

async fn health() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(r#"{"status": "Ok"}"#)
}

async fn run() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let _redis_manager =
        RedisManager::init_global(&redis_url).expect("Failed to initialize Redis manager");

    _redis_manager
        .connect()
        .await
        .expect("Failed to connect to Redis");

    println!("Connected to Redis");

    start_response_consumer().await;
    start_db_read_response_consumer().await;

    HttpServer::new(move || {
        App::new()
            .service(signup_user)
            .service(signin_user)
            .service(signin_admin)
            .service(get_all_events)
            .service(search_events)
            .service(get_event_by_id)
            .service(get_orderbook_by_market)
            .service(get_orderbooks_by_event)
            .service(get_orderbooks_by_outcome)
            .route("/health", web::get().to(health))
            .service(
                web::scope("")
                    .wrap(AuthMiddleware)
                    .service(get_balance)
                    .service(onramp)
                    .service(place_order)
                    .service(cancel_order)
                    .service(modify_order)
                    .service(split_order)
                    .service(merge_order)
                    .service(get_open_orders)
                    .service(get_order_history)  
                    .service(get_order_status)   
                    .service(get_orders_by_user)
                    .service(get_orders_by_market)
                    .service(get_positions)
                    .service(get_portfolio)
                    .service(get_position_by_market)
                    .service(get_positions_history)
                    .service(add_market_bookmark)
                    .service(remove_market_bookmark)
                    .service(get_user_bookmarks)
                    .service(get_for_you_markets)
                    .service(get_trades)
                    .service(get_trade_by_id)
                    .service(get_trades_by_market)
                    .service(get_trades_by_user_id)
                    .service(get_user_by_id)
                    .service(
                        web::scope("")
                            .wrap(AdminMiddleware)
                            .service(create_event)
                            .service(update_event)
                            .service(resolve_event)
                            .service(delete_event)
                            .service(get_all_users),
                    ),
            )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

fn main() -> std::io::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");
    runtime.block_on(run())
}
