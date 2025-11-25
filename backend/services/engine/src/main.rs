mod services;
mod store;
mod types;

use dotenvy::dotenv;
use redis_client::RedisManager;
use services::request_consumer::start_request_consumer;
use std::env;
use store::market;
use store::orderbook;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let market_store = market::MarketStore::new();
    let orderbook = orderbook::spawn_orderbook_actor(market_store);

    start_request_consumer(orderbook).await;

    println!("Engine services ready");

    tokio::signal::ctrl_c().await?;
    Ok(())
}
