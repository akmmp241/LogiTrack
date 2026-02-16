use crate::domain::biteship::BiteshipChangeStatusEvent;
use crate::domain::shipment::{Shipment, ShipmentStatus, StatusMapping, TrackingEventSource};
use crate::errors::ShipmentServiceError;
use crate::services::DefaultService;
use chrono::Utc;
use sqlx::types::Json;
use sqlx::{Error, PgPool, Postgres, Transaction};
use std::sync::Arc;

pub struct BiteshipService {
    db: Arc<PgPool>,
}

impl BiteshipService {
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    pub async fn handle_event(
        &self,
        event_type: &str,
        payload: &BiteshipChangeStatusEvent,
    ) -> Result<(), ShipmentServiceError> {
        match event_type {
            "order.status" => self.handle_status_change(payload).await,
            _ => Err(ShipmentServiceError::UnsupportedEvent(format!(
                "Unsupported event type: {}",
                event_type
            ))),
        }
    }

    async fn handle_status_change(
        &self,
        payload: &BiteshipChangeStatusEvent,
    ) -> Result<(), ShipmentServiceError> {
        let mut tx = self.db.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        let shipment = self
            .get_shipment_by_waybill_id(&mut tx, &payload.courier_waybill_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get shipment by waybill id: {}", e);
                ShipmentServiceError::Unexpected(e.into())
            })?
            .ok_or_else(|| ShipmentServiceError::NotFound(payload.courier_waybill_id.clone()))?;

        let status_mapped = self
            .get_status_mapped(&mut tx, &payload.status)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get status mapping: {}", e);
                ShipmentServiceError::Unexpected(e.into())
            })?
            .ok_or_else(|| {
                ShipmentServiceError::StatusMappingNotFound(payload.courier_waybill_id.clone())
            })?;

        self.update_shipment_status(&mut tx, &shipment, status_mapped.normalized_status.clone())
            .await
            .map_err(|e| match e {
                Error::RowNotFound => {
                    ShipmentServiceError::NotFound(payload.courier_waybill_id.clone())
                }
                _ => {
                    tracing::error!("Failed to update shipment status: {}", e);
                    ShipmentServiceError::Unexpected(e.into())
                }
            })?;

        self.log_tracking_event(&mut tx, shipment, status_mapped)
            .await
            .map_err(|e| {
                tracing::error!("failed to insert tracking event: {}", e);
                ShipmentServiceError::Unexpected(e.into())
            })?;

        self.log_webhook_event(&mut tx, payload)
            .await
            .map_err(|e| {
                tracing::error!("Failed to log webhook event: {}", e);
                ShipmentServiceError::Unexpected(e.into())
            })?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        Ok(())
    }

    async fn get_status_mapped(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        raw_status: &str,
    ) -> Result<Option<StatusMapping>, sqlx::Error> {
        let status_mapping: Option<StatusMapping> = sqlx::query_as(
            "SELECT id, platform, raw_status, normalized_status 
                    FROM status_mappings WHERE raw_status = $1",
        )
        .bind(raw_status)
        .fetch_optional(&mut **tx)
        .await?;

        Ok(status_mapping)
    }

    async fn update_shipment_status(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        shipment: &Shipment,
        new_status: ShipmentStatus,
    ) -> Result<(), sqlx::Error> {
        let res =
            sqlx::query("UPDATE shipments SET current_status = $1, updated_at = $2 WHERE id = $3")
                .bind(new_status)
                .bind(Utc::now())
                .bind(shipment.id)
                .execute(&mut **tx)
                .await?;

        if res.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn get_shipment_by_waybill_id(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        waybill_id: &str,
    ) -> Result<Option<Shipment>, sqlx::Error> {
        let res: Option<Shipment> = sqlx::query_as(
            "SELECT id, waybill_id, courier_code,
                    source, current_status, order_id,
                    external_order_ref, created_at, updated_at FROM shipments
                    WHERE waybill_id = $1",
        )
        .bind(waybill_id)
        .fetch_optional(&mut **tx)
        .await?;

        Ok(res)
    }
}

#[async_trait::async_trait]
impl DefaultService for BiteshipService {
    async fn log_tracking_event(
        &self,
        tx: &mut Transaction<Postgres>,
        shipment: Shipment,
        status_mapping: StatusMapping,
    ) -> Result<(), Error> {
        let res = sqlx::query(
            "
                INSERT INTO tracking_events
                    (shipment_id, raw_status, normalized_status,
                     description, occurred_at, source, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(shipment.id)
        .bind(status_mapping.raw_status)
        .bind(status_mapping.normalized_status)
        .bind("something")
        .bind(Utc::now())
        .bind(TrackingEventSource::Webhook)
        .bind(Utc::now())
        .execute(&mut **tx)
        .await?;

        if res.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }

    async fn log_webhook_event(
        &self,
        tx: &mut Transaction<Postgres>,
        payload: &BiteshipChangeStatusEvent,
    ) -> Result<(), Error> {
        let res = sqlx::query(
            "INSERT INTO webhook_logs (payload, processed_at, created_at) VALUES ($1, $2, $3)",
        )
        .bind(Json(payload))
        .bind(Utc::now())
        .bind(payload.updated_at)
        .execute(&mut **tx)
        .await?;

        if res.rows_affected() == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }
}
