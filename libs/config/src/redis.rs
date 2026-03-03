use redis::Client;

pub fn get_redis_client() -> Result<Client, redis::RedisError> {
    let url = std::env::var("REDIS_URL").expect("REDIS_URL not set");

    let client = Client::open(url)?;

    tracing::info!("Redis client created");
    Ok(client)
}
