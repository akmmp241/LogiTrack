use crate::domain::{TemplateId, TrackingEventMsg, TrackingMsgPayload};
use crate::ports::ChannelPort;
use anyhow::anyhow;
use config::reqwest::get_reqwest_pool;
use handlebars::Handlebars;
use waha::WahaUseCase;

pub struct WhatsappSender {
    client: WahaUseCase,
}

impl WhatsappSender {
    pub fn new() -> Self {
        let client_http = get_reqwest_pool().expect("Failed to create reqwest pool");

        let client = WahaUseCase::new(client_http);

        Self { client }
    }
}

#[async_trait::async_trait]
impl ChannelPort for WhatsappSender {
    async fn send(
        &self,
        event: &TrackingEventMsg,
        content: String,
        _subject: String,
    ) -> anyhow::Result<()> {
        self.client
            .send_text(event.recipient.clone(), content)
            .await?;

        Ok(())
    }

    fn render(
        &self,
        template_id: TemplateId,
        data: &mut TrackingMsgPayload,
    ) -> anyhow::Result<(String, String)> {
        let mut handlebars = Handlebars::new();

        let template = match template_id {
            TemplateId::TrackingCreatedWa => (
                "templates/tracking_added/chat.stub",
                "Your Shipment Is On Tracking",
            ),
            _ => return Err(anyhow!("invalid template")),
        };

        let root = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(root).join(template.0);

        handlebars.register_template_file("chat", path)?;

        data.courier = data.courier.to_uppercase();
        data.status = data.status.to_uppercase();
        let rendered = handlebars.render("chat", data)?;

        Ok((rendered, template.1.to_string()))
    }
}
