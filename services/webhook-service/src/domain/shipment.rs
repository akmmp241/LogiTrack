use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::fmt::Display;
use uuid::Uuid;

#[derive(Type, Debug, Clone, Serialize, Deserialize)]
#[sqlx(type_name = "shipment_source", rename_all = "UPPERCASE")]
pub enum ShipmentSource {
    Internal,
    External,
}

#[derive(Type, Debug, Clone, Serialize, Deserialize)]
#[sqlx(type_name = "shipment_status", rename_all = "SCREAMING_SNAKE_CASE")]
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
        write!(f, "{}", format!("{:?}", self))
    }
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    pub id: Uuid,
    pub waybill_id: String,
    pub courier_code: String,
    pub source: ShipmentSource,
    pub order_id: Option<Uuid>,
    #[sqlx(rename = "external_order_ref")]
    pub external_ref_id: Option<String>,
    pub current_status: ShipmentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct StatusMapping {
    pub id: Uuid,
    pub platform: String,
    pub raw_status: String,
    pub normalized_status: ShipmentStatus,
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
        write!(f, "{}", format!("{:?}", self))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShipmentSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub shipment_id: Uuid,
    pub subscribed_statues: Vec<ShipmentStatus>,
    pub subscribed_channels: Vec<NotificationChannel>,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
