use crate::consumer::NotificationConsumer;
use crate::handler::NotificationHandler;
use crate::ports::email::EmailSmtpSender;
use crate::ports::telegram::TelegramSender;
use crate::ports::whatsapp::WhatsappSender;
use config::lettre::create_smtp_transport;
use config::postgres::get_db_connection;
use std::env;
use std::sync::Arc;

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

    let wa_queue = env::var("WA_QUEUE").expect("WA_QUEUE env var not set");
    let tele_queue = env::var("TELE_QUEUE").expect("TELE_QUEUE env var not set");
    let email_queue = env::var("EMAIL_QUEUE").expect("EMAIL_QUEUE env var not set");

    let wa_handler = NotificationHandler::new(Arc::new(WhatsappSender::new()), pool.clone()).await;
    let tele_handler =
        NotificationHandler::new(Arc::new(TelegramSender::new()), pool.clone()).await;
    let email_handler =
        NotificationHandler::new(Arc::new(EmailSmtpSender::new(mailer)), pool.clone()).await;

    let mut consumers = Vec::<NotificationConsumer>::new();
    consumers.push(NotificationConsumer::new(wa_handler, wa_queue).await);
    consumers.push(NotificationConsumer::new(tele_handler, tele_queue).await);
    consumers.push(NotificationConsumer::new(email_handler, email_queue).await);

    let mut tasks = Vec::<tokio::task::JoinHandle<()>>::new();
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
