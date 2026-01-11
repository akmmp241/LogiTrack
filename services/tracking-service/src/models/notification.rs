use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Type, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "notification_channel", rename_all = "UPPERCASE")]
pub enum NotificationChannel {
    Whatsapp,
    Email,
    Push,
}

impl Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
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
    pub template_code: String,
    pub payload: TrackingMsgPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingMsgPayload {
    pub waybill_id: String,
    pub status: String,
    pub courier: String,
}
