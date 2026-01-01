use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::error::Error;
use uuid::Uuid;

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct StatusMapping {
    pub id: Uuid,
    pub platform: String,
    pub raw_status: String,
    pub normalized_status: ShipmentStatus,
}

#[derive(Type, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[sqlx(type_name = "shipment_source", rename_all = "UPPERCASE")]
pub enum ShipmentSource {
    Internal,
    External,
}

#[derive(Type, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[async_trait]
pub trait ShipmentStatusParse {
    async fn map_external_status(
        &self,
        external_status: &str,
    ) -> Result<ShipmentStatus, Box<dyn Error>>;
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Shipment {
    pub id: Uuid,
    pub waybill_id: String,
    pub courier_code: String,
    pub source: ShipmentSource,
    pub order_id: Option<Uuid>,
    pub external_ref_id: Option<String>,
    pub current_status: ShipmentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ShipmentSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub shipment_id: Uuid,
    pub subscribed_statues: Vec<ShipmentStatus>,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
