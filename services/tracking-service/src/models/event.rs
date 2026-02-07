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
    pub location: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: TrackingEventSource,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum TrackingEventSource {
    Polling,
    Webhook,
}
