use crate::models::shipment::{Shipment, ShipmentStatus};
use biteship::error::TrackingError;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct ShipmentRepository {
    pub pool: Pool<Postgres>,
}

impl ShipmentRepository {
    pub async fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn save(&self, shipment: Shipment) -> Result<(), Option<TrackingError>> {
        let res = sqlx::query(
            "INSERT INTO  shipments
                (id, waybill_id, courier_code,
                 source, current_status, order_id,
                 external_order_ref, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
            .bind(shipment.id)
            .bind(shipment.waybill_id)
            .bind(shipment.courier_code)
            .bind(shipment.source)
            .bind(shipment.current_status)
            .bind(shipment.order_id)
            .bind(shipment.external_ref_id)
            .bind(shipment.created_at)
            .bind(shipment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|er| {
            self.handle_db_err(er)
        });

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn get_all(&self, user_id: Uuid) -> Result<Vec<Shipment>, sqlx::Error> {
        let res: Vec<Shipment> = sqlx::query_as(
            "SELECT id, waybill_id, courier_code,
                    source, current_status, order_id,
                    external_order_ref, created_at, updated_at FROM shipments WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn get_by_id(
        &self,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Shipment>, sqlx::Error> {
        let res: Option<Shipment> = sqlx::query_as(
            "SELECT id, waybill_id, courier_code,
                    source, current_status, order_id,
                    external_order_ref, created_at, updated_at FROM shipments
                    WHERE user_id = $1 AND id = $2",
        )
        .bind(user_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn delete_by_id(&self, user_id: Uuid, id: Uuid) -> Result<u64, sqlx::Error> {
        let res = sqlx::query("DELETE FROM shipments WHERE user_id = $1 AND id = $2")
            .bind(user_id)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected())
    }

    fn handle_db_err(&self, e: sqlx::Error) -> Option<TrackingError> {
        if let Some(db_err) = e.as_database_error() {
            match db_err.code().map(|c| c.to_string()).as_deref() {
                Some("23505") => return Some(TrackingError::DuplicateTrackingNumber),
                Some("23503") => return Some(TrackingError::NotFound),
                _ => {}
            }
        }

        tracing::error!("Internal DB Error: {:?}", e);
        None
    }
}
