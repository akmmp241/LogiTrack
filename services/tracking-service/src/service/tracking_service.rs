use crate::models::dto::{AddTrackingRequest, AddTrackingResponse};
use crate::models::shipment::{
    Shipment, ShipmentSource, ShipmentStatus, ShipmentStatusParse, ShipmentSubscription,
};
use crate::repository::shipment_repo::ShipmentRepository;
use crate::repository::shipment_status_mapping_repo::ShipmentStatusMappingRepository;
use crate::repository::shipment_subscription::ShipmentSubsRepository;
use biteship::BiteshipUseCase;
use chrono::Utc;
use errors::error::HttpError;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct TrackingService {
    pub shipment_repository: ShipmentRepository,
    pub shipment_subs_repo: ShipmentSubsRepository,
    pub map_status_repo: ShipmentStatusMappingRepository,
    pub biteship_uc: BiteshipUseCase,
}

impl TrackingService {
    pub async fn new(
        shipment_repository: ShipmentRepository,
        shipment_subs_repo: ShipmentSubsRepository,
        map_status_repo: ShipmentStatusMappingRepository,
        biteship_uc: BiteshipUseCase,
    ) -> Self {
        Self {
            shipment_repository,
            shipment_subs_repo,
            map_status_repo,
            biteship_uc,
        }
    }

    pub async fn add_track(
        &self,
        req: &AddTrackingRequest,
    ) -> Result<AddTrackingResponse, HttpError> {
        let bs_resp = self
            .biteship_uc
            .fetch_public_tracking(req.awb.clone(), req.courier_code.clone())
            .await?;

        let external = match req.is_internal {
            true => ShipmentSource::Internal,
            false => ShipmentSource::External,
        };

        let current_time = Utc::now();

        let status = self
            .map_status_repo
            .map_external_status(bs_resp.status.as_str())
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

        let shipment = Shipment {
            id: Uuid::new_v4(),
            waybill_id: req.awb.clone(),
            courier_code: req.courier_code.clone(),
            source: external,
            order_id: None,
            external_ref_id: None,
            current_status: status,
            created_at: current_time,
            updated_at: current_time,
        };
        let shipment_id_clone = shipment.id.clone();

        self.shipment_repository
            .save(shipment)
            .await
            .map_err(|e| match e {
                Some(err) => HttpError::BadRequest(err.to_string()),
                None => HttpError::InternalServerError(anyhow::anyhow!("error from db")),
            })?;

        let user_uuid = Uuid::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let subs = ShipmentSubscription {
            id: Uuid::new_v4(),
            // this is a dummy user for the development phase
            user_id: user_uuid,
            shipment_id: shipment_id_clone,
            subscribed_statues: vec![
                ShipmentStatus::InTransit,
                ShipmentStatus::OutForDelivery,
                ShipmentStatus::Delivered,
                ShipmentStatus::Delivered,
            ],
            label: "default".into(),
            created_at: current_time,
            updated_at: current_time,
        };

        self.shipment_subs_repo
            .save(subs)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

        let response = AddTrackingResponse {
            message: "Successfully add new tracking".into(),
        };

        Ok(response)
    }
}
