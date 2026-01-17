use crate::domain::{TemplateId, TrackingEventMsg, TrackingMsgPayload};

pub mod email;
pub mod telegram;
pub mod whatsapp;

#[async_trait::async_trait]
pub trait ChannelPort: Send + Sync {
    async fn send(
        &self,
        event: &TrackingEventMsg,
        content: String,
        subject: String,
    ) -> anyhow::Result<()>;

    /// return two parameters: rendered content and subject
    /// the first parameter is an HTML string content
    /// the second one is a subject
    ///
    ///
    /// maybe i'm gonna refactor this into a struct...maybe
    fn render(
        &self,
        template_id: TemplateId,
        data: &mut TrackingMsgPayload,
    ) -> anyhow::Result<(String, String)>;
}
