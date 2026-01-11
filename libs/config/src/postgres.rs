use std::time::Duration;
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