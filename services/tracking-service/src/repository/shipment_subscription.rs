use crate::models::shipment::ShipmentSubscription;
use sqlx::{Pool, Postgres};
use std::error::Error;
use uuid::Uuid;

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

    pub async fn delete_by_shipment_id(
        &self,
        user_id: Uuid,
        shipment_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let res = sqlx::query(
            "DELETE FROM shipment_subscriptions WHERE user_id = $1 AND shipment_id = $2",
        )
        .bind(user_id)
        .bind(shipment_id)
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }
}
