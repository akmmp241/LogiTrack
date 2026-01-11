use crate::consumer::NotificationConsumer;
use crate::handler::NotificationHandler;
use crate::ports::email::EmailSmtpSender;
use crate::ports::telegram::TelegramSender;
use crate::ports::whatsapp::WhatsappSender;
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

    let wa_queue = env::var("WA_QUEUE").expect("WA_QUEUE env var not set");
    let tele_queue = env::var("TELE_QUEUE").expect("TELE_QUEUE env var not set");
    let email_queue = env::var("EMAIL_QUEUE").expect("EMAIL_QUEUE env var not set");

    let wa_handler = NotificationHandler::new(Arc::new(WhatsappSender::new())).await;
    let tele_handler = NotificationHandler::new(Arc::new(TelegramSender::new())).await;
    let email_handler = NotificationHandler::new(Arc::new(EmailSmtpSender::new().await)).await;

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
