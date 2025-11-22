mod controllers;
mod types;
mod utils;
mod services;
mod middleware;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use std::env;
use dotenvy::dotenv;
use redis_client::RedisManager;
use crate::controllers::user_controller::{signup_user, signin_user, get_balance, onramp};
use crate::controllers::admin_auth_controller::{signin_admin};
use crate::controllers::admin_event_controller::{create_event,resolve_event, update_event, delete_event};
use crate::controllers::user_event_controller::{get_all_events, get_event_by_id, search_events};
use crate::controllers::order_controller::{place_order, cancel_order, modify_order, get_open_orders, get_order_status, get_order_history, get_orders_by_user, get_orders_by_market};
use crate::controllers::position_controller::{get_positions, get_position_by_market, get_portfolio, get_positions_history};
use crate::controllers::trade_controller::{get_trades, get_trade_by_id, get_trades_by_market, get_trades_by_user_id};
use crate::controllers::user_profile_controller::{get_user_by_id, get_all_users};
use crate::services::response_consumer::start_response_consumer;
use crate::services::db_read_response_consumer::start_db_read_response_consumer;
use crate::middleware::auth::AuthMiddleware;
use crate::middleware::admin::AdminMiddleware;


async fn health()-> impl Responder {
    HttpResponse::Ok()
    .content_type("application/json")
    .body(r#"{"status": "Ok"}"#)
}

async fn run()-> std::io::Result<()> {

    dotenv().ok();

    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    let _redis_manager = RedisManager::init_global(&redis_url)
        .expect("Failed to initialize Redis manager");
    
    _redis_manager.connect().await
        .expect("Failed to connect to Redis");
    
    println!("Connected to Redis");

    start_response_consumer().await;
    start_db_read_response_consumer().await;

    HttpServer::new(move|| {
        let public_scope = web::scope("")
            .service(signup_user)
            .service(signin_user)
            .service(signin_admin)
            .service(get_all_events)
            .service(get_event_by_id)
            .service(search_events)
            .route("/health", web::get().to(health));

        let protected_scope = web::scope("")
            .wrap(AuthMiddleware)
            .service(get_balance)
            .service(onramp)
            .service(place_order)
            .service(cancel_order)
            .service(modify_order)
            .service(get_open_orders)
            .service(get_order_status)
            .service(get_order_history)
            .service(get_orders_by_user)
            .service(get_orders_by_market)
            .service(get_positions)
            .service(get_position_by_market)
            .service(get_portfolio)
            .service(get_positions_history)
            .service(get_trades)
            .service(get_trade_by_id)
            .service(get_trades_by_market)
            .service(get_trades_by_user_id)
            .service(get_user_by_id)
            .service(get_all_users);

        let admin_scope = web::scope("")
            .wrap(AuthMiddleware)
            .wrap(AdminMiddleware)
            .service(create_event)
            .service(update_event)
            .service(resolve_event)
            .service(delete_event);

        App::new()
            .service(public_scope)
            .service(protected_scope)
            .service(admin_scope)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

fn main()-> std::io::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");
    runtime.block_on(run())
}