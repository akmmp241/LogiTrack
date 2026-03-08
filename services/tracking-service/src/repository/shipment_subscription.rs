use crate::models::notification::NotificationChannel;
use crate::models::shipment::{ShipmentStatus, ShipmentSubscription};
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

    pub async fn find_by_shipment_id(
        &self,
        user_id: &Uuid,
        shipment_id: &Uuid,
    ) -> Result<Option<ShipmentSubscription>, sqlx::Error> {
        let res = sqlx::query_as::<_, ShipmentSubscription>(
            r#"
                SELECT id, user_id, shipment_id,
                       subscribed_statuses, subscribed_channels, label,
                       created_at, updated_at
                FROM shipment_subscriptions
                    WHERE user_id = $1 AND shipment_id = $2
                "#,
        )
        .bind(user_id)
        .bind(shipment_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn update_by_shipment_id(
        &self,
        user_id: &Uuid,
        shipment_id: &Uuid,
        subscribed_statues: &Vec<ShipmentStatus>,
        subscribed_channels: &Vec<NotificationChannel>,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            r#"
                UPDATE shipment_subscriptions SET subscribed_statuses = $1, subscribed_channels = $2, updated_at = NOW()
                WHERE user_id = $3 AND shipment_id = $4
                "#,
        )
            .bind(subscribed_statues)
            .bind(subscribed_channels)
            .bind(user_id)
            .bind(shipment_id)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() == 1)
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
