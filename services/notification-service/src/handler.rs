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

        let (content, subject) = self.sender.render(template, &event.payload)?;

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
            _ => Err(anyhow::anyhow!("Unsupported channel")),
        }
    }
}
