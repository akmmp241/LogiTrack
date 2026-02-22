use crate::domain;
use crate::domain::TrackingEventMsg;
use crate::ports::ChannelPort;
use anyhow::anyhow;
use config::lettre::create_smtp_transport;
use domain::{TemplateId, TrackingMsgPayload};
use handlebars::Handlebars;
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::env;

pub struct EmailSmtpSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl EmailSmtpSender {
    pub async fn new() -> Self {
        let mailer = create_smtp_transport()
            .await
            .expect("Failed to create smtp transport");

        let from = env::var("SMTP_FROM_EMAIL").expect("SMTP_FROM_EMAIL must be set");

        Self {
            mailer,
            from: from.parse().expect("Failed to parse SMTP_FROM_EMAIL"),
        }
    }
}

#[async_trait::async_trait]
impl ChannelPort for EmailSmtpSender {
    async fn send(
        &self,
        event: &TrackingEventMsg,
        content: String,
        subject: String,
    ) -> anyhow::Result<()> {
        let email = Message::builder()
            .from(self.from.clone())
            .to(event.recipient.parse()?)
            .subject(subject.as_str())
            .header(ContentType::TEXT_HTML)
            .body(content)?;

        self.mailer.send(email).await?;

        Ok(())
    }

    fn render(
        &self,
        template_id: TemplateId,
        data: &mut TrackingMsgPayload,
    ) -> anyhow::Result<(String, String)> {
        let mut handlebars = Handlebars::new();

        let template = match template_id {
            TemplateId::TrackingCreatedEmail => (
                "templates/tracking_added/email.mustache",
                "Your Shipment Is On Tracking",
            ),
            TemplateId::TrackingStatusUpdatedEmail => (
                "templates/tracking_status_updated/email.mustache",
                "Your Shipment Status Has Been Updated",
            ),
            _ => return Err(anyhow!("invalid template")),
        };

        let root = std::env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(root).join(template.0);

        handlebars.register_template_file("email", path)?;

        let rendered = handlebars.render("email", data)?;

        Ok((rendered, template.1.into()))
    }
}
