use serde::{Deserialize, Serialize};
use sqlx::Type;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "notification_channel")]
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
    pub shipment_id: Uuid,
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

pub struct NotificationLog {
    pub id: Uuid,
    pub shipment_id: Uuid,
    pub event_id: Uuid,
    pub channel: NotificationChannel,
    pub recipient_to: String,
    pub message_content: String,
    pub status: String,
    pub error_message: String,
    pub sent_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
