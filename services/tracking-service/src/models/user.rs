use crate::models::notification::NotificationChannel;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserWithNotifPref {
    pub id: Uuid,
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub default_channels: Vec<NotificationChannel>,
}
