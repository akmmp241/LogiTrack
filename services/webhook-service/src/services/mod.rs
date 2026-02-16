use crate::domain::biteship::BiteshipChangeStatusEvent;
use crate::domain::shipment::{Shipment, StatusMapping};
use sqlx::{Postgres, Transaction};

pub mod biteship_service;

#[async_trait::async_trait]
pub trait DefaultService {
    async fn log_tracking_event(
        &self,
        tx: &mut Transaction<Postgres>,
        shipment: Shipment,
        status_mapping: StatusMapping,
    ) -> Result<(), sqlx::Error>;
    async fn log_webhook_event(
        &self,
        tx: &mut Transaction<Postgres>,
        payload: &BiteshipChangeStatusEvent,
    ) -> Result<(), sqlx::Error>;
}
