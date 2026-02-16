use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BiteshipChangeStatusEvent {
    pub event: String,
    pub order_id: String,
    pub order_price: u32,
    pub courier_tracking_id: String,
    pub courier_waybill_id: String,
    pub courier_company: String,
    pub courier_type: String,
    pub courier_driver_name: String,
    pub courier_driver_phone: String,
    pub courier_driver_plate_number: String,
    pub courier_driver_photo_url: String,
    pub courier_link: String,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}
