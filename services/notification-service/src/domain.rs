use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum NotificationChannel {
    Whatsapp,
    Email,
    Telegram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackingEventMsgType {
    #[serde(rename = "tracking.added")]
    TrackingAdded,
    #[serde(rename = "tracking.status_updated")]
    TrackingStatusUpdated,
}

// event message for rabbitmq :D
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEventMsg {
    pub message_id: Uuid,
    pub event_type: TrackingEventMsgType,
    pub channel: NotificationChannel,
    pub user_id: Uuid,
    pub recipient: String,
    pub payload: TrackingMsgPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingMsgPayload {
    pub waybill_id: String,
    pub status: String,
    pub courier: String,
}

#[derive(Debug)]
pub enum TemplateId {
    TrackingCreatedEmail,
    TrackingCreatedWa,
    TrackingCreatedTele,
    TrackingStatusUpdatedEmail,
    TrackingStatusUpdatedWa,
    TrackingStatusUpdatedTele,
}
