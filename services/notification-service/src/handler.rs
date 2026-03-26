use crate::billing::{self, BillingResult};
use crate::domain::{
    BillingDeductionEvent, NotificationChannel, NotificationLog, TemplateId, TrackingEventMsg,
    TrackingEventMsgType, WalletTransaction,
};
use crate::ports::ChannelPort;
use chrono::Utc;
use lapin::BasicProperties;
use lapin::options::BasicPublishOptions;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

static BILLING_EXCHANGE: &str = "billing.events";
static BILLING_ROUTING_KEY: &str = "billing.deduction.created";

pub struct NotificationHandler {
    sender: Arc<dyn ChannelPort>,
    pool: PgPool,
    redis: deadpool_redis::Pool,
    rabbitmq_channel: lapin::Channel,
    http_client: reqwest::Client,
    topup_url: String,
}

impl NotificationHandler {
    pub async fn new(
        sender: Arc<dyn ChannelPort>,
        pool: PgPool,
        redis: deadpool_redis::Pool,
        rabbitmq_channel: lapin::Channel,
        http_client: reqwest::Client,
        topup_url: String,
    ) -> Self {
        Self {
            sender,
            pool,
            redis,
            rabbitmq_channel,
            http_client,
            topup_url,
        }
    }

    pub async fn handle(&self, event: &TrackingEventMsg) -> anyhow::Result<()> {
        let mut billing_result =
            billing::check_and_deduct_balance(&self.redis, &event.user_id, &event.channel).await?;

        if let BillingResult::PriceNotFound = billing_result {
            tracing::warn!(
                "cache miss for notification prices, attempting to rehydrate cache from topup-service"
            );
            let rehydrate_url = format!("{}/internal/api/pricing/notifications", self.topup_url);

            match self.http_client.get(&rehydrate_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    billing_result = billing::check_and_deduct_balance(
                        &self.redis,
                        &event.user_id,
                        &event.channel,
                    )
                    .await?;

                    if let BillingResult::PriceNotFound = billing_result {
                        tracing::error!("cache miss persists even after rehydration");
                        return Err(anyhow::anyhow!(
                            "failed to retrieve notification price after cache rehydration"
                        ));
                    }
                }
                Ok(resp) => {
                    tracing::error!(
                        "failed to rehydrate cache: topup-service returned status {}",
                        resp.status()
                    );
                    return Err(anyhow::anyhow!(
                        "failed to rehydrate cache: HTTP {}",
                        resp.status()
                    ));
                }
                Err(e) => {
                    tracing::error!("failed to reach topup-service for cache rehydration: {}", e);
                    return Err(anyhow::anyhow!("topup-service unavailable: {}", e));
                }
            }
        }

        match billing_result {
            BillingResult::PriceNotFound => {
                unreachable!("PriceNotFound should have been handled by the retry loop");
            }
            BillingResult::InsufficientBalance {
                required_price,
                current_balance,
            } => {
                return Err(anyhow::anyhow!(
                    "insufficient balance: required {} but only {} available",
                    required_price,
                    current_balance
                ));
            }
            BillingResult::Success {
                price_deducted,
                remaining_balance,
            } => {
                let template = self.resolve_template(event).map_err(|e| {
                    tracing::error!("error while resolving template: {:?}", e);
                    e
                })?;

                let (content, subject) =
                    self.sender.render(template, &mut event.payload.clone())?;

                self.sender.send(event, content.clone(), subject).await?;

                self.create_notification_log(&NotificationLog {
                    id: event.message_id,
                    shipment_id: event.payload.shipment_id,
                    event_id: event.message_id,
                    channel: event.channel.clone(),
                    recipient_to: event.recipient.clone(),
                    message_content: content,
                    status: "SENT".to_string(),
                    error_message: "-".to_string(),
                    sent_at: Utc::now(),
                    created_at: Utc::now(),
                })
                .await?;

                self.publish_billing_event(
                    &event.user_id,
                    &event.payload.shipment_id,
                    price_deducted,
                    event.channel.clone(),
                )
                .await?;
            }
        }

        Ok(())
    }

    fn resolve_template(&self, event: &TrackingEventMsg) -> anyhow::Result<TemplateId> {
        match (&event.event_type, &event.channel) {
            (TrackingEventMsgType::TrackingAdded, NotificationChannel::Whatsapp) => {
                Ok(TemplateId::TrackingCreatedWa)
            }
            (TrackingEventMsgType::TrackingAdded, NotificationChannel::Email) => {
                Ok(TemplateId::TrackingCreatedEmail)
            }
            (TrackingEventMsgType::TrackingAdded, NotificationChannel::Telegram) => {
                Ok(TemplateId::TrackingCreatedTele)
            }
            (TrackingEventMsgType::TrackingStatusUpdated, NotificationChannel::Whatsapp) => {
                Ok(TemplateId::TrackingStatusUpdatedWa)
            }
            (TrackingEventMsgType::TrackingStatusUpdated, NotificationChannel::Email) => {
                Ok(TemplateId::TrackingStatusUpdatedEmail)
            }
            (TrackingEventMsgType::TrackingStatusUpdated, NotificationChannel::Telegram) => {
                Ok(TemplateId::TrackingStatusUpdatedTele)
            }
        }
    }

    async fn create_notification_log(
        &self,
        payload: &NotificationLog,
    ) -> Result<bool, sqlx::Error> {
        let rows = sqlx::query(
            r#"
                INSERT INTO notification_logs 
                    (shipment_id, event_id, channel, 
                     recipient_to, message_content, status,
                     error_message, sent_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                "#,
        )
        .bind(payload.shipment_id)
        .bind(payload.event_id)
        .bind(payload.channel.clone())
        .bind(payload.recipient_to.clone())
        .bind(payload.message_content.clone())
        .bind(payload.status.clone())
        .bind(payload.error_message.clone())
        .execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() == 1)
    }

    async fn publish_billing_event(
        &self,
        user_id: &Uuid,
        shipment_id: &Uuid,
        amount: i64,
        channel: NotificationChannel,
    ) -> anyhow::Result<()> {
        let event = BillingDeductionEvent {
            user_id: *user_id,
            shipment_id: *shipment_id,
            amount,
            transaction_type: WalletTransaction::from(channel),
            deducted_at: Utc::now(),
        };

        let payload = serde_json::to_vec(&event).map_err(|e| {
            tracing::error!("failed to serialize billing event: {}", e);
            anyhow::anyhow!("serialization error: {}", e)
        })?;

        let confirm = self
            .rabbitmq_channel
            .basic_publish(
                BILLING_EXCHANGE,
                BILLING_ROUTING_KEY,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await
            .map_err(|e| {
                tracing::error!("failed to publish billing event: {}", e);
                anyhow::anyhow!("rabbitmq publish error: {}", e)
            })?;

        confirm.await.map_err(|e| {
            tracing::error!("billing event publish not confirmed: {}", e);
            anyhow::anyhow!("rabbitmq confirm error: {}", e)
        })?;

        tracing::info!(
            user_id = %user_id,
            shipment_id = %shipment_id,
            amount = amount,
            "billing deduction event published"
        );

        Ok(())
    }
}
