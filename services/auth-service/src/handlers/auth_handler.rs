use crate::app::AppState;
use crate::models::api_key::{CreateApiKeyRequest, ValidateApiKeyRequest};
use crate::models::user::{LoginRequest, RegisterRequest};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use errors::error::HttpError;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, HttpError> {
    state.service.register(req).await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"message": "User registered successfully"})),
    ))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let response = state.service.login(req).await?;
    Ok(Json(response))
}

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = extract_user_id(&headers)?;
    let response = state.service.create_api_key(user_id, req).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = extract_user_id(&headers)?;
    let keys = state.service.list_api_keys(user_id).await?;
    Ok(Json(keys))
}

pub async fn revoke_api_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = extract_user_id(&headers)?;
    state.service.revoke_api_key(user_id, id).await?;
    Ok(Json(json!({"message": "API key revoked"})))
}

pub async fn validate_api_key(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ValidateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let response = state.service.validate_api_key(&req.api_key).await?;
    Ok(Json(response))
}

fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, HttpError> {
    let user_id_str = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| HttpError::Unauthorized("Missing x-user-id header".to_string()))?;

    Uuid::parse_str(user_id_str)
        .map_err(|_| HttpError::BadRequest("Invalid user ID format".to_string()))
}
