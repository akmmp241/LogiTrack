use crate::domain::{TemplateId, TrackingEventMsg, TrackingMsgPayload};
use crate::ports::ChannelPort;
use config::reqwest::get_reqwest_pool;
use reqwest::Client;

pub struct TelegramSender {
    client: Client,
}

impl TelegramSender {
    pub fn new() -> Self {
        let client = get_reqwest_pool().expect("Failed to create reqwest pool");

        Self { client }
    }
}

#[async_trait::async_trait]
impl ChannelPort for TelegramSender {
    async fn send(
        &self,
        event: &TrackingEventMsg,
        content: String,
        subject: String,
    ) -> anyhow::Result<()> {
        tracing::info!("sending tracking event to whatsapp");
        todo!()
    }

    fn render(
        &self,
        template_id: TemplateId,
        data: &mut TrackingMsgPayload,
    ) -> anyhow::Result<(String, String)> {
        todo!()
    }
}
