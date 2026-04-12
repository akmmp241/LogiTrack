use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use redis::{RedisWrite, ToRedisArgs};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "notification_channel", rename_all = "UPPERCASE")]
pub enum NotificationChannel {
    Whatsapp,
    Email,
    Telegram,
}

impl Display for NotificationChannel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            NotificationChannel::Whatsapp => write!(f, "Whatsapp"),
            NotificationChannel::Email => write!(f, "Email"),
            NotificationChannel::Telegram => write!(f, "Telegram"),
        }
    }
}

impl ToRedisArgs for NotificationChannel {
    fn write_redis_args<W: RedisWrite + ?Sized>(&self, out: &mut W) {
        let s = match self {
            NotificationChannel::Whatsapp => "Whatsapp",
            NotificationChannel::Email => "Email",
            NotificationChannel::Telegram => "Telegram",
        };
        out.write_arg(s.as_bytes());
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "wallet_transaction", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalletTransaction {
    Topup,
    AwbCreation,
    EmailNotification,
    WhatsappNotification,
}

impl From<NotificationChannel> for WalletTransaction {
    fn from(channel: NotificationChannel) -> Self {
        match channel {
            NotificationChannel::Whatsapp => Self::WhatsappNotification,
            NotificationChannel::Email => Self::EmailNotification,
            _ => Self::AwbCreation,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingDeductionEvent {
    pub user_id: Uuid,
    pub shipment_id: Uuid,
    pub amount: i64,
    pub transaction_type: WalletTransaction,
    pub deducted_at: DateTime<Utc>,
}
