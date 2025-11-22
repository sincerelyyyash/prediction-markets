mod stream_consumer;
mod handlers;
mod dead_letter;
mod read_consumer;
mod models;

use std::env;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use redis_client::RedisManager;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create Postgres pool");

    info!("Connected to Postgres Database");

    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    let _redis_manager = RedisManager::init_global(&redis_url)
        .expect("Failed to initialize Redis manager");
    
    _redis_manager.connect().await
        .expect("Failed to connect to Redis");
    
    info!("Connected to Redis");

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        stream_consumer::start_db_event_consumer(pool_clone).await;
    });

    read_consumer::start_read_request_consumer(pool).await;

    Ok(())
}
