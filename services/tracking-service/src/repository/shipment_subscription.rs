use crate::models::shipment::ShipmentSubscription;
use sqlx::{Pool, Postgres};
use std::error::Error;

#[derive(Clone)]
pub struct ShipmentSubsRepository {
    pub pool: Pool<Postgres>,
}

impl ShipmentSubsRepository {
    pub async fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn save(&self, shipment_subs: ShipmentSubscription) -> Result<(), Box<dyn Error>> {
        let _res = sqlx::query(
            "INSERT INTO  shipment_subscriptions (
                                     user_id, shipment_id,
                                     subscribed_statuses, label, created_at,
                                     updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(shipment_subs.user_id)
        .bind(shipment_subs.shipment_id)
        .bind(shipment_subs.subscribed_statues)
        .bind(shipment_subs.label)
        .bind(shipment_subs.created_at)
        .bind(shipment_subs.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
