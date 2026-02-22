use crate::domain::{
    ShipmentStatus, ShipmentSubscription, ShipmentWithJob, StatusMapping, TrackingEventSource,
};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct PollingRepository {
    pool: Pool<Postgres>,
}

impl PollingRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn fetch_due_jobs(&self) -> Result<Vec<ShipmentWithJob>, sqlx::Error> {
        let jobs: Vec<ShipmentWithJob> = sqlx::query_as(
            r#"
            SELECT
                s.id AS shipment_id,
                s.waybill_id,
                s.courier_code,
                s.current_status,
                tj.next_run_at,
                tj.interval_minutes,
                tj.attempt
            FROM tracking_jobs tj
            JOIN shipments s ON s.id = tj.shipment_id
            WHERE tj.is_active = true
              AND tj.next_run_at <= NOW()
            ORDER BY tj.next_run_at ASC
            FOR UPDATE OF tj SKIP LOCKED
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }

    pub async fn update_job_schedule(
        &self,
        shipment_id: Uuid,
        next_run_at: DateTime<Utc>,
        interval_minutes: i32,
        attempt: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE tracking_jobs
            SET next_run_at = $1,
                interval_minutes = $2,
                attempt = $3,
                updated_at = NOW()
            WHERE shipment_id = $4
            "#,
        )
        .bind(next_run_at)
        .bind(interval_minutes)
        .bind(attempt)
        .bind(shipment_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn deactivate_job(&self, shipment_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE tracking_jobs
            SET is_active = false,
                updated_at = NOW()
            WHERE shipment_id = $1
            "#,
        )
        .bind(shipment_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_shipment_status(
        &self,
        shipment_id: Uuid,
        new_status: &ShipmentStatus,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE shipments
            SET current_status = $1,
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(new_status)
        .bind(shipment_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_tracking_event(
        &self,
        shipment_id: Uuid,
        raw_status: &str,
        normalized_status: &ShipmentStatus,
        description: &str,
        occurred_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO tracking_events
                (shipment_id, raw_status, normalized_status,
                 description, occurred_at, source, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#,
        )
        .bind(shipment_id)
        .bind(raw_status)
        .bind(normalized_status)
        .bind(description)
        .bind(occurred_at)
        .bind(TrackingEventSource::Polling)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_status_mapping(
        &self,
        raw_status: &str,
    ) -> Result<Option<StatusMapping>, sqlx::Error> {
        let mapping: Option<StatusMapping> = sqlx::query_as(
            r#"
            SELECT id, platform, raw_status, normalized_status
            FROM status_mappings
            WHERE raw_status = $1
            "#,
        )
        .bind(raw_status)
        .fetch_optional(&self.pool)
        .await?;

        Ok(mapping)
    }

    pub async fn get_shipment_subscriptions(
        &self,
        shipment_id: Uuid,
    ) -> Result<Vec<ShipmentSubscription>, sqlx::Error> {
        let subs: Vec<ShipmentSubscription> = sqlx::query_as(
            r#"
            SELECT id, user_id, shipment_id,
                   subscribed_statuses, subscribed_channels,
                   label, created_at, updated_at
            FROM shipment_subscriptions
            WHERE shipment_id = $1
            "#,
        )
        .bind(shipment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(subs)
    }
}
