use crate::models::status::ShipmentStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Shipment {
    pub id: Uuid,
    pub tracking_number: String,
    pub courier_code: String,
    pub courier_service: Option<String>,
    pub source: ShipmentSource,
    pub current_status: ShipmentStatus,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ShipmentSource {
    Internal,
    External,
}
