use crate::domain::{
    NotificationChannel, NotificationLog, TemplateId, TrackingEventMsg, TrackingEventMsgType,
};
use crate::ports::ChannelPort;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use std::sync::Arc;

pub struct NotificationHandler {
    sender: Arc<dyn ChannelPort>,
    pool: PgPool,
}

impl NotificationHandler {
    pub async fn new(sender: Arc<dyn ChannelPort>, pool: PgPool) -> Self {
        Self { sender, pool }
    }

    pub async fn handle(&self, event: &TrackingEventMsg) -> anyhow::Result<()> {
        let template = self.resolve_template(event)?;

        let (content, subject) = self.sender.render(template, &mut event.payload.clone())?;

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
            _ => Err(anyhow::anyhow!("Unsupported channel")),
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
}
