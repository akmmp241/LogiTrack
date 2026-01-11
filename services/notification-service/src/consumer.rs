use crate::domain::TrackingEventMsg;
use crate::handler::NotificationHandler;
use config::rabbitmq::create_channel;
use futures_util::StreamExt;
use lapin::Channel;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;

pub struct NotificationConsumer {
    channel: Channel,
    handler: NotificationHandler,
    queue: String,
}

impl NotificationConsumer {
    pub async fn new(handler: NotificationHandler, queue: String) -> Self {
        let channel = create_channel().await.expect("Failed to create channel");

        Self {
            channel,
            handler,
            queue,
        }
    }

    pub async fn start(&self) -> Result<(), anyhow::Error> {
        tracing::info!("starting consumer for queue {}", self.queue);

        let mut consumer = self
            .channel
            .basic_consume(
                self.queue.as_str(),
                format!("{}-consumer", self.queue).as_str(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        while let Some(delivery) = consumer.next().await {
            let delivery = delivery?;

            let event: TrackingEventMsg = serde_json::from_slice(&delivery.data).map_err(|e| {
                anyhow::anyhow!(
                    "failed to deserialize event: {}, consumer: {}",
                    e,
                    self.queue
                )
            })?;

            let result = self.handler.handle(&event).await;

            match result {
                Ok(_) => {
                    delivery.ack(BasicAckOptions::default()).await?;
                }
                Err(e) => {
                    tracing::error!("failed to handle event: {}, consumer: {}", e, self.queue);
                    continue;
                }
            }
        }

        Ok(())
    }
}
