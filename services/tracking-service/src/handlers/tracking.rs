use crate::app::AppState;
use crate::helper::extract_user_id;
use crate::models::dto::AddTrackingRequest;
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use errors::error::HttpError;
use std::sync::Arc;
use uuid::Uuid;

pub async fn create_shipments(
    State(handler): State<Arc<AppState>>,
    headers: HeaderMap,
    payload: Result<Json<AddTrackingRequest>, JsonRejection>,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = extract_user_id(&headers)?;

    let Json(data) = payload?;

    let res = handler.service.add_track(user_id, &data).await?;

    Ok(res)
}

pub async fn get_shipments(
    State(handler): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = extract_user_id(&headers)?;

    let res = handler.service.get_shipments(user_id).await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn get_shipment_by_id(
    State(handler): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HttpError> {
    let id: Uuid = id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid uuid".into()))?;

    let user_id = extract_user_id(&headers)?;

    let res = handler.service.get_shipment_by_id(id, user_id).await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn delete_shipment_by_id(
    State(handler): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HttpError> {
    let id: Uuid = id
        .parse()
        .map_err(|_| HttpError::BadRequest("invalid uuid".into()))?;

    let user_id = extract_user_id(&headers)?;

    handler.service.delete_shipment_by_id(id, user_id).await?;

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
