use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::fmt::Display;
use uuid::Uuid;

#[derive(Type, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[sqlx(type_name = "shipment_status", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipmentStatus {
    Created,
    Received,
    InTransit,
    OutForDelivery,
    Delivered,
    Failed,
    Returned,
    Cancelled,
    Unknown,
}

impl Display for ShipmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "tracking_event_source")]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackingEventSource {
    Polling,
    Webhook,
}

#[derive(Type, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "notification_channel", rename_all = "UPPERCASE")]
pub enum NotificationChannel {
    Whatsapp,
    Email,
}

impl Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// tracking_jobs + shipments
#[derive(FromRow, Debug, Clone)]
#[allow(dead_code)]
pub struct ShipmentWithJob {
    // shipment fields
    pub shipment_id: Uuid,
    pub waybill_id: String,
    pub courier_code: String,
    pub current_status: ShipmentStatus,
    // tracking job fields
    pub next_run_at: DateTime<Utc>,
    pub interval_minutes: i32,
    pub attempt: i32,
}

#[derive(FromRow, Debug, Clone)]
#[allow(dead_code)]
pub struct StatusMapping {
    pub id: Uuid,
    pub platform: String,
    pub raw_status: String,
    pub normalized_status: ShipmentStatus,
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct ShipmentSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub shipment_id: Uuid,
    pub subscribed_statuses: Vec<ShipmentStatus>,
    pub subscribed_channels: Vec<NotificationChannel>,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackingEventMsgType {
    #[serde(rename = "tracking.status_updated")]
    TrackingStatusUpdated,
}

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

pub fn get_interval_minutes(status: &ShipmentStatus) -> Option<i32> {
    match status {
        ShipmentStatus::Created => Some(360),       // 6 hours
        ShipmentStatus::Received => Some(180),      // 3 hours
        ShipmentStatus::InTransit => Some(60),      // 1 hour
        ShipmentStatus::OutForDelivery => Some(60), // 1 hour
        ShipmentStatus::Unknown => Some(360),       // safe default, 6 hours
        ShipmentStatus::Delivered
        | ShipmentStatus::Failed
        | ShipmentStatus::Returned
        | ShipmentStatus::Cancelled => None,
    }
}

#[derive(Debug, Clone)]
pub struct TrackingResult {
    pub raw_status: String,
    pub description: String,
    pub occurred_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait LogisticsProvider: Send + Sync {
    async fn fetch_tracking(
        &self,
        waybill_id: &str,
        courier_code: &str,
    ) -> Result<TrackingResult, anyhow::Error>;
}
