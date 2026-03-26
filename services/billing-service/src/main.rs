use crate::consumer::BillingConsumer;
use crate::handler::BillingHandler;
use config::postgres::get_db_connection;
use std::env;

mod consumer;
mod domain;
mod handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    observability::init("billing-service");

    let pool = get_db_connection()
        .await
        .expect("Failed to get database connection");

    let queue = env::var("BILLING_QUEUE").unwrap_or_else(|_| "billing.deduction".to_string());

    let handler = BillingHandler::new(pool);
    let consumer = BillingConsumer::new(handler, queue).await;

    consumer.start().await?;

    Ok(())
}
