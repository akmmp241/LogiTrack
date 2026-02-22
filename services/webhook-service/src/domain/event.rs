use crate::domain::shipment::NotificationChannel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackingEventMsgType {
    #[serde(rename = "tracking.added")]
    TrackingAdded,
    #[serde(rename = "tracking.status_updated")]
    TrackingStatusUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEventMsg {
    pub message_id: Uuid,
    pub event_type: TrackingEventMsgType,
    pub channel: NotificationChannel,
    pub user_id: Uuid,
    pub recipient: String,
    pub template_code: String,
    pub payload: TrackingMsgPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingMsgPayload {
    pub waybill_id: String,
    pub status: String,
    pub courier: String,
}
