use crate::models::dto::{
    AddTrackingRequest, AddTrackingResponse, GetShipmentPreferencesResponse, GetShipmentResponse,
    GetShipmentsResponse, TrackingEventRes, UpdateShipmentPreferencesReq,
};
use crate::models::notification::{
    NotificationChannel, TrackingEventMsg, TrackingEventMsgType, TrackingMsgPayload,
};
use crate::models::notification_log;
use crate::models::shipment::{
    Shipment, ShipmentSource, ShipmentStatus, ShipmentStatusParse, ShipmentSubscription,
    TrackingJob,
};
use crate::models::user::UserWithNotifPref;
use crate::repository::notification_log_repo::NotificationLogRepository;
use crate::repository::shipment_repo::ShipmentRepository;
use crate::repository::shipment_status_mapping_repo::ShipmentStatusMappingRepository;
use crate::repository::shipment_subscription::ShipmentSubsRepository;
use crate::repository::tracking_event_repo::TrackingEventRepo;
use crate::repository::tracking_job_repo::TrackingJobRepository;
use crate::repository::user_repo::UserRepository;
use anyhow::anyhow;
use biteship::BiteshipUseCase;
use chrono::{Duration, Utc};
use errors::error::HttpError;
use lapin::BasicProperties;
use lapin::options::BasicPublishOptions;
use notification_log::NotificationLog;
use uuid::Uuid;

static EXCHANGE_NAME: &str = "notification.events";

#[derive(Clone)]
pub struct Repositories {
    pub shipment_repository: ShipmentRepository,
    pub shipment_subs_repo: ShipmentSubsRepository,
    pub map_status_repo: ShipmentStatusMappingRepository,
    pub tracking_event_repo: TrackingEventRepo,
    pub tracking_job_repo: TrackingJobRepository,
    pub user_repo: UserRepository,
    pub notification_log_repo: NotificationLogRepository,
}

#[derive(Clone)]
pub struct TrackingService {
    pub repos: Repositories,
    pub biteship_uc: BiteshipUseCase,
    pub rabbitmq_channel: lapin::Channel,
}

impl TrackingService {
    pub async fn new(
        repos: Repositories,
        biteship_uc: BiteshipUseCase,
        rabbitmq_channel: lapin::Channel,
    ) -> Self {
        Self {
            repos,
            biteship_uc,
            rabbitmq_channel,
        }
    }

    pub async fn add_track(
        &self,
        user_id: Uuid,
        req: &AddTrackingRequest,
    ) -> Result<AddTrackingResponse, HttpError> {
        let user_with_notif_pref = self.get_user(&user_id).await?;

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
            .repos
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

        self.repos
            .shipment_repository
            .save(shipment.clone())
            .await
            .map_err(|e| match e {
                Some(err) => HttpError::BadRequest(err.to_string()),
                None => HttpError::InternalServerError(anyhow::anyhow!("error from db")),
            })?;

        let subs = ShipmentSubscription {
            id: Uuid::new_v4(),
            user_id,
            shipment_id: shipment.id,
            subscribed_channels: req.notify_on.clone(),
            subscribed_statues: vec![
                ShipmentStatus::InTransit,
                ShipmentStatus::OutForDelivery,
                ShipmentStatus::Delivered,
                ShipmentStatus::Delivered,
            ],
            label: req.label.clone(),
            created_at: current_time,
            updated_at: current_time,
        };

        self.repos
            .shipment_subs_repo
            .save(subs)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

        let job = TrackingJob {
            shipment_id: shipment.id,
            next_run_at: Utc::now() + Duration::hours(6),
            interval_minutes: Duration::hours(6).num_minutes(),
        };

        self.repos
            .tracking_job_repo
            .save(&job)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

        for ch in req.notify_on.iter() {
            if !user_with_notif_pref.default_channels.contains(ch) {
                continue;
            }

            let recipient = match ch {
                NotificationChannel::Whatsapp => user_with_notif_pref.phone_number.clone(),
                NotificationChannel::Email => user_with_notif_pref.email.clone(),
            };

            let payload = TrackingEventMsg {
                message_id: Uuid::new_v4(),
                event_type: TrackingEventMsgType::TrackingAdded,
                channel: ch.clone(),
                user_id,
                recipient: recipient.to_string(),
                template_code: "TRACKING_STATUS".to_string(),
                payload: TrackingMsgPayload {
                    shipment_id: shipment.id,
                    waybill_id: req.awb.clone(),
                    status: shipment.current_status.to_string().to_lowercase(),
                    courier: shipment.courier_code.clone(),
                },
            };

            let payload = serde_json::to_vec(&payload).map_err(|e| {
                HttpError::InternalServerError(anyhow!("failed to serialize msg payload"))
            })?;

            let sent = self
                .rabbitmq_channel
                .basic_publish(
                    EXCHANGE_NAME,
                    format!(
                        "notification.tracking_added.{}",
                        ch.to_string().to_lowercase()
                    )
                    .as_str(),
                    BasicPublishOptions::default(),
                    &payload,
                    BasicProperties::default().with_delivery_mode(2),
                )
                .await
                .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

            sent.await
                .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;
        }

        let response = AddTrackingResponse {
            message: "Successfully add new tracking".into(),
        };

        Ok(response)
    }

    pub async fn get_shipments(&self, user_id: Uuid) -> Result<GetShipmentsResponse, HttpError> {
        let res = self
            .repos
            .shipment_repository
            .get_all(user_id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

        Ok(res)
    }

    pub async fn get_shipment_by_id(
        &self,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<GetShipmentResponse, HttpError> {
        let shipment = self
            .repos
            .shipment_repository
            .get_by_id(user_id, id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?
            .ok_or_else(|| HttpError::NotFound("shipment not found".into()))?;

        let events_res = self
            .repos
            .tracking_event_repo
            .get_by_shipment_id(shipment.id)
            .await
            .map_err(|e| {
                tracing::error!("failed to get events for shipment: {}", e);
                HttpError::InternalServerError(anyhow::anyhow!(e.to_string()))
            })?;

        let mut events: Vec<TrackingEventRes> = Vec::new();
        events_res.iter().for_each(|e| {
            events.push(TrackingEventRes {
                normalized_status: e.normalized_status.to_string(),
                description: e.description.clone(),
                occurred_at: e.occurred_at,
            })
        });

        let res = GetShipmentResponse { shipment, events };

        Ok(res)
    }

    pub async fn delete_shipment_by_id(&self, id: Uuid, user_id: Uuid) -> Result<(), HttpError> {
        self.repos
            .shipment_subs_repo
            .delete_by_shipment_id(user_id, id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

        let rows_affected = self
            .repos
            .shipment_repository
            .delete_by_id(user_id, id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

        if rows_affected == 0 {
            return Err(HttpError::NotFound("shipment not found".into()));
        }

        Ok(())
    }

    pub async fn get_shipment_events(&self, id: Uuid) -> Result<Vec<TrackingEventRes>, HttpError> {
        let res = self
            .repos
            .tracking_event_repo
            .get_by_shipment_id(id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e)))?;

        let mut events: Vec<TrackingEventRes> = Vec::new();
        res.iter().for_each(|e| {
            events.push(TrackingEventRes {
                normalized_status: e.normalized_status.to_string(),
                description: e.description.clone(),
                occurred_at: e.occurred_at,
            })
        });

        Ok(events)
    }

    pub async fn get_notification_logs_by_shipment(
        &self,
        shipment_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<NotificationLog>, HttpError> {
        self.repos
            .shipment_repository
            .get_by_id(user_id, shipment_id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?
            .ok_or_else(|| HttpError::NotFound("shipment not found".into()))?;

        let logs = self
            .repos
            .notification_log_repo
            .get_by_shipment_id(shipment_id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

        Ok(logs)
    }

    pub async fn get_notification_log_by_id(
        &self,
        notification_id: Uuid,
        shipment_id: Uuid,
        user_id: Uuid,
    ) -> Result<NotificationLog, HttpError> {
        self.repos
            .shipment_repository
            .get_by_id(user_id, shipment_id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?
            .ok_or_else(|| HttpError::NotFound("shipment not found".into()))?;

        let log = self
            .repos
            .notification_log_repo
            .get_by_id(notification_id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?
            .ok_or_else(|| HttpError::NotFound("notification log not found".into()))?;

        if log.shipment_id != shipment_id {
            return Err(HttpError::NotFound("notification log not found".into()));
        }

        Ok(log)
    }

    pub async fn get_shipment_preferences(
        &self,
        user_id: Uuid,
        shipment_id: Uuid,
    ) -> Result<GetShipmentPreferencesResponse, HttpError> {
        let res = self
            .repos
            .shipment_subs_repo
            .find_by_shipment_id(&user_id, &shipment_id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?
            .ok_or_else(|| HttpError::NotFound("shipment not found".into()))?;

        Ok(GetShipmentPreferencesResponse {
            subscribed_channels: res.subscribed_channels,
            subscribed_statues: res.subscribed_statues,
        })
    }

    pub async fn update_shipment_preferences(
        &self,
        user_id: Uuid,
        shipment_id: Uuid,
        req: UpdateShipmentPreferencesReq,
    ) -> Result<(), HttpError> {
        let result = self
            .repos
            .shipment_subs_repo
            .update_by_shipment_id(
                &user_id,
                &shipment_id,
                &req.subscribed_statues,
                &req.subscribed_channels,
            )
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?;

        if !result {
            return Err(HttpError::NotFound("shipment not found".into()));
        }

        Ok(())
    }

    async fn get_user(&self, user_id: &Uuid) -> Result<UserWithNotifPref, HttpError> {
        let res = self
            .repos
            .user_repo
            .get_by_id(user_id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e.to_string())))?
            .ok_or_else(|| HttpError::Unauthorized("User unauthorized".to_string()))?;

        Ok(res)
    }
}
