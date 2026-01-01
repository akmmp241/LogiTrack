use std::sync::Arc;
use crate::app::AppState;
use crate::models::dto::AddTrackingRequest;
use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use errors::error::HttpError;

pub async fn create_shipments(
    State(handler): State<Arc<AppState>>,
    payload: Result<Json<AddTrackingRequest>, JsonRejection>,
) -> Result<impl IntoResponse, HttpError> {

    let Json(data) = payload?;

    let res = handler.service.add_track(&data).await?;

    Ok(res)
}

pub async fn get_shipments() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

pub async fn get_shipment_by_id() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

pub async fn delete_shipment_by_id() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

pub async fn get_shipment_events() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}
