use std::time::Duration;
use reqwest::Client;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn get_db_connection() -> Result<PgPool, sqlx::Error> {
    let user = std::env::var("POSTGRES_USER").expect("POSTGRES_USER not set");
    let pass = std::env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD not set");
    let db = std::env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
    let port = std::env::var("POSTGRES_PORT").expect("POSTGRES_PORT not set");
    let host = std::env::var("POSTGRES_HOST").expect("POSTGRES_HOST not set");

    let dsn = format!("postgresql://{}:{}@{}:{}/{}", user, pass, host, port, db);

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .max_lifetime(Duration::from_secs(30))
        .connect(&dsn)
        .await?;

    tracing::info!("DB connection established");

    Ok(pool)
}

pub fn get_reqwest_pool() -> Result<Client, reqwest::Error> {
    let pool = Client::builder()
        .pool_max_idle_per_host(10)
        .timeout(Duration::from_secs(30))
        .build()?;

    Ok(pool)
}