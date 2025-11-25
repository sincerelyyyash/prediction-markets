mod redis_manager;
use redis_manager::RedisManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = RedisManager::init_global("redis://127.0.0.1:6379")?;
    manager.connect().await?;
    Ok(())
}
