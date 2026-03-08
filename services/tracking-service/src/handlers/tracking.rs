use crate::app::AppState;
use crate::middlewares::CurrentUser;
use crate::models::dto::{AddTrackingRequest, UpdateShipmentPreferencesReq};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use errors::error::HttpError;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn create_shipments(
    State(handler): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
    payload: Result<Json<AddTrackingRequest>, JsonRejection>,
) -> Result<impl IntoResponse, HttpError> {
    let Json(data) = payload?;

    let res = handler.service.add_track(user.id, &data).await?;

    Ok(res)
}

pub async fn get_shipments(
    State(handler): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    tracing::info!("test");

    let res = handler.service.get_shipments(user.id).await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn get_shipment_by_id(
    State(handler): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let id: Uuid = id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid uuid".into()))?;

    let res = handler.service.get_shipment_by_id(id, user.id).await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn delete_shipment_by_id(
    State(handler): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let id: Uuid = id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid uuid".into()))?;

    handler.service.delete_shipment_by_id(id, user.id).await?;

    Ok(StatusCode::OK)
}

pub async fn get_shipment_events(
    State(handler): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let id: Uuid = id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid uuid".into()))?;

    let res = handler.service.get_shipment_events(id).await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn get_shipment_pref(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let id = id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid uuid".into()))?;

    let res = state.service.get_shipment_preferences(user.id, id).await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn update_shipment_pref(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(user): Extension<CurrentUser>,
    req: Result<Json<UpdateShipmentPreferencesReq>, JsonRejection>,
) -> Result<impl IntoResponse, HttpError> {
    let Json(data) = req?;

    let id = id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid uuid".into()))?;

    let _res = state
        .service
        .update_shipment_preferences(user.id, id, data)
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({"message": "Update preferences successfully"})),
    ))
}
