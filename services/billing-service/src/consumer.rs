use crate::domain::BillingDeductionEvent;
use crate::handler::BillingHandler;
use config::rabbitmq::create_channel;
use futures_util::StreamExt;
use lapin::Channel;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions};
use lapin::types::FieldTable;

pub struct BillingConsumer {
    channel: Channel,
    handler: BillingHandler,
    queue: String,
}

impl BillingConsumer {
    pub async fn new(handler: BillingHandler, queue: String) -> Self {
        let channel = create_channel()
            .await
            .expect("Failed to create RabbitMQ channel");

        Self {
            channel,
            handler,
            queue,
        }
    }

    pub async fn start(&self) -> Result<(), anyhow::Error> {
        tracing::info!("starting billing consumer for queue: {}", self.queue);

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

            let event: BillingDeductionEvent = match serde_json::from_slice(&delivery.data) {
                Ok(evt) => evt,
                Err(e) => {
                    tracing::error!(
                        "failed to deserialize billing event: {}, queue: {}",
                        e,
                        self.queue
                    );
                    delivery
                        .nack(BasicNackOptions {
                            requeue: false,
                            ..Default::default()
                        })
                        .await?;
                    continue;
                }
            };

            match self.handler.handle(&event).await {
                Ok(_) => {
                    delivery.ack(BasicAckOptions::default()).await?;
                }
                Err(e) => {
                    tracing::error!(
                        user_id = %event.user_id,
                        shipment_id = %event.shipment_id,
                        "failed to handle billing event: {}",
                        e,
                    );
                    // NACK with requeue so it can be retried
                    delivery
                        .nack(BasicNackOptions {
                            requeue: true,
                            ..Default::default()
                        })
                        .await?;
                }
            }
        }

        Ok(())
    }
}
