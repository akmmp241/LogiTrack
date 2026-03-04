use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(rename_all = "UPPERCASE", type_name = "notification_channel")]
pub enum NotificationChannel {
    Whatsapp,
    Email,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserNotificationPreferences {
    pub user_id: Uuid,
    pub default_channels: Vec<NotificationChannel>,
    pub updated_at: DateTime<Utc>,
}

pub type GetCurrentPreferencesResponse = Vec<NotificationChannel>;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNotifPrefRequest {
    pub channels: Vec<NotificationChannel>,
}
