use crate::app::AppState;
use crate::middlewares::CurrentUser;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use errors::error::HttpError;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_shipment_notification_logs(
    State(handler): State<Arc<AppState>>,
    Path(shipment_id): Path<String>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let shipment_id: Uuid = shipment_id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid uuid".into()))?;

    let res = handler
        .service
        .get_notification_logs_by_shipment(shipment_id, user.id)
        .await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn get_shipment_notification_log_by_id(
    State(handler): State<Arc<AppState>>,
    Path((shipment_id, notification_id)): Path<(String, String)>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let shipment_id: Uuid = shipment_id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid shipment uuid".into()))?;

    let notification_id: Uuid = notification_id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid notification uuid".into()))?;

    let res = handler
        .service
        .get_notification_log_by_id(notification_id, shipment_id, user.id)
        .await?;

    Ok((StatusCode::OK, Json(res)))
}
