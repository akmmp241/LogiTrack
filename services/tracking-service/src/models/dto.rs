use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use crate::models::notification::NotificationChannel;

#[derive(Serialize, Deserialize, Debug)]
pub struct AddTrackingRequest {
    pub awb: String,
    pub courier_code: String,
    pub is_internal: bool,
    pub notify_on: Vec<NotificationChannel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddTrackingResponse {
    pub message: String,
}

impl IntoResponse for AddTrackingResponse {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}