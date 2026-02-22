use crate::provider::BiteshipProvider;
use crate::repository::PollingRepository;
use crate::service::PollingService;
use biteship::BiteshipUseCase;
use config::postgres::get_db_connection;
use config::rabbitmq::create_channel;
use config::reqwest::get_reqwest_pool;
use domain::LogisticsProvider;
use std::env;
use std::sync::Arc;
use tokio::time::{self, Duration};

mod domain;
mod provider;
mod repository;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    observability::init("polling-service");

    let tick_interval_secs: u64 = env::var("POLLING_TICK_INTERVAL_SECS")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .expect("POLLING_TICK_INTERVAL_SECS is not valid");

    let db = get_db_connection()
        .await
        .expect("couldn't connect to database");

    let reqwest_pool = get_reqwest_pool().expect("couldn't create reqwest pool");

    let rabbitmq_channel = create_channel()
        .await
        .expect("couldn't create rabbitmq channel");

    let repo = PollingRepository::new(db);

    let biteship_uc = BiteshipUseCase::new(reqwest_pool);
    let provider: Arc<dyn LogisticsProvider> = Arc::new(BiteshipProvider::new(biteship_uc));

    let polling_service = PollingService::new(repo, provider, rabbitmq_channel);

    tracing::info!(
        "polling worker started — tick interval: {}s",
        tick_interval_secs
    );

    let mut interval = time::interval(Duration::from_secs(tick_interval_secs));

    loop {
        interval.tick().await;
        polling_service.poll_due_shipments().await;
    }
}
