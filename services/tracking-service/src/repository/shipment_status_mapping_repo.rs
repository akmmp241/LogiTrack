use crate::models::shipment::{ShipmentStatus, ShipmentStatusParse, StatusMapping};
use sqlx::{query_as, Pool, Postgres};
use std::error::Error;
use async_trait::async_trait;

#[derive(Clone)]
pub struct ShipmentStatusMappingRepository {
    pub pool: Pool<Postgres>,
}

impl ShipmentStatusMappingRepository {
    pub async fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShipmentStatusParse for ShipmentStatusMappingRepository {
    async fn map_external_status(
        &self,
        external_status: &str,
    ) -> Result<ShipmentStatus, Box<dyn Error>> {
        let status: Option<StatusMapping> = query_as(
            "SELECT id, platform, raw_status, normalized_status
                FROM status_mappings WHERE raw_status = $1",
        )
        .bind(external_status)
        .fetch_optional(&self.pool)
        .await?;

        match status {
            Some(status) => Ok(status.normalized_status),
            None => Ok(ShipmentStatus::Unknown),
        }
    }
}
