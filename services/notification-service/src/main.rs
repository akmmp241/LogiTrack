use crate::consumer::NotificationConsumer;
use crate::handler::NotificationHandler;
use crate::ports::email::EmailSmtpSender;
use crate::ports::telegram::TelegramSender;
use crate::ports::whatsapp::WhatsappSender;
use config::lettre::create_smtp_transport;
use config::postgres::get_db_connection;
use config::rabbitmq::create_channel;
use config::redis::create_redis_pool;
use config::reqwest::get_reqwest_pool;
use std::env;
use std::sync::Arc;

mod billing;
mod consumer;
mod domain;
mod handler;
mod ports;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    observability::init("notification-service");

    let pool = get_db_connection()
        .await
        .expect("Failed to get database connection");

    let mailer = create_smtp_transport()
        .await
        .expect("Failed to create smtp transport");

    let redis_pool = create_redis_pool().await;

    let wa_queue = env::var("WA_QUEUE").expect("WA_QUEUE env var not set");
    let tele_queue = env::var("TELE_QUEUE").expect("TELE_QUEUE env var not set");
    let email_queue = env::var("EMAIL_QUEUE").expect("EMAIL_QUEUE env var not set");

    let wa_rmq_channel = create_channel()
        .await
        .expect("Failed to create WA RabbitMQ channel");
    let tele_rmq_channel = create_channel()
        .await
        .expect("Failed to create Tele RabbitMQ channel");
    let email_rmq_channel = create_channel()
        .await
        .expect("Failed to create Email RabbitMQ channel");

    let http_client = get_reqwest_pool().expect("Failed to get HTTP client");
    let topup_url = env::var("TOPUP_SERVICE_URL").expect("TOPUP_SERVICE_URL env var not set");

    let wa_handler = NotificationHandler::new(
        Arc::new(WhatsappSender::new()),
        pool.clone(),
        redis_pool.clone(),
        wa_rmq_channel,
        http_client.clone(),
        topup_url.clone(),
    )
    .await;
    let tele_handler = NotificationHandler::new(
        Arc::new(TelegramSender::new()),
        pool.clone(),
        redis_pool.clone(),
        tele_rmq_channel,
        http_client.clone(),
        topup_url.clone(),
    )
    .await;
    let email_handler = NotificationHandler::new(
        Arc::new(EmailSmtpSender::new(mailer)),
        pool.clone(),
        redis_pool.clone(),
        email_rmq_channel,
        http_client.clone(),
        topup_url.clone(),
    )
    .await;

    let mut consumers = Vec::<NotificationConsumer>::new();
    consumers.push(NotificationConsumer::new(wa_handler, wa_queue).await);
    consumers.push(NotificationConsumer::new(tele_handler, tele_queue).await);
    consumers.push(NotificationConsumer::new(email_handler, email_queue).await);

    let mut tasks = Vec::<tokio::task::JoinHandle<()>>::new();
    
    // Metrics server task
    let metrics_task = tokio::spawn(async move {
        let app = axum::Router::new()
            .route("/metrics", axum::routing::get(observability::metrics_handler));
        
        let port = env::var("NOTIFICATION_SERVICE_METRICS_PORT").unwrap_or_else(|_| "3006".to_string());
        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind metrics server");
        
        tracing::info!("Metrics server listening on http://{}", addr);
        axum::serve(listener, app).await.expect("Failed to start metrics server");
    });
    tasks.push(metrics_task);

    for consumer in consumers {
        let task = tokio::spawn(async move { consumer.start().await.unwrap() });
        tasks.push(task);
    }

    for task in tasks {
        match task.await {
            Ok(_) => {}
            Err(e) => tracing::error!("error: {}", e),
        }
    }

    Ok(())
}
