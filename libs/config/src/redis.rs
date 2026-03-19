use deadpool_redis::Runtime;
use redis::Client;

pub fn get_redis_client() -> Result<Client, redis::RedisError> {
    let url = std::env::var("REDIS_URL").expect("REDIS_URL not set");

    let client = Client::open(url)?;

    tracing::info!("Redis client created");
    Ok(client)
}

pub async fn create_redis_pool() -> deadpool_redis::Pool {
    let url = std::env::var("REDIS_URL").expect("REDIS_URL not set");

    let cfg = deadpool_redis::Config::from_url(&url);

    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Cannot create redis pool")
}
