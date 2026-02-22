use crate::domain::{NotificationChannel, TemplateId, TrackingEventMsg, TrackingEventMsgType};
use crate::ports::ChannelPort;
use std::sync::Arc;

pub struct NotificationHandler {
    sender: Arc<dyn ChannelPort>,
}

impl NotificationHandler {
    pub async fn new(sender: Arc<dyn ChannelPort>) -> Self {
        Self { sender }
    }

    pub async fn handle(&self, event: &TrackingEventMsg) -> anyhow::Result<()> {
        let template = self.resolve_template(event)?;

        let (content, subject) = self.sender.render(template, &mut event.payload.clone())?;

        self.sender.send(event, content, subject).await?;

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
}
