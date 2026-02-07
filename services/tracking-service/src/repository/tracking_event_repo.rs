use crate::models::event::TrackingEvent;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct TrackingEventRepo {
    pub pool: Pool<Postgres>,
}

impl TrackingEventRepo {
    pub async fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(
        &self,
        tracking_event_id: &Uuid,
    ) -> Result<Option<TrackingEvent>, sqlx::Error> {
        let res: Option<TrackingEvent> = sqlx::query_as(
            "SELECT id, shipment_id, raw_status
                        normalized_status, description, occurred_at
                        location, source, created_at
                FROM tracking_status WHERE id = $1",
        )
        .bind(tracking_event_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn get_by_shipment_id(
        &self,
        shipment_id: Uuid,
    ) -> Result<Vec<TrackingEvent>, sqlx::Error> {
        let res: Vec<TrackingEvent> = sqlx::query_as(
            "SELECT id, shipment_id, raw_status
                        normalized_status, description, occurred_at
                        location, source, created_at
                FROM tracking_events WHERE shipment_id = $1",
        )
        .bind(shipment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(res)
    }
}
