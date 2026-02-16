use crate::models::shipment::ShipmentStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrackingEvent {
    pub id: Uuid,
    pub shipment_id: Uuid,
    pub raw_status: String,
    pub normalized_status: ShipmentStatus,
    pub description: String,
    pub occurred_at: DateTime<Utc>,
    pub source: TrackingEventSource,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "tracking_event_source")]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackingEventSource {
    Polling,
    Webhook,
}
