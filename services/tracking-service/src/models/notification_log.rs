use crate::models::notification::NotificationChannel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
