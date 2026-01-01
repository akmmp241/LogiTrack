use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Type, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "notification_channel", rename_all = "UPPERCASE")]
pub enum NotificationChannel {
    Whatsapp,
    Email,
    Push,
}
