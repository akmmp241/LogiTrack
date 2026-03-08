use crate::app::AppState;
use crate::middlewares::CurrentUser;
use crate::models::api_key::{CreateApiKeyRequest, ValidateApiKeyRequest};
use crate::models::user::{LoginRequest, RegisterRequest};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use errors::error::HttpError;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, HttpError> {
    state.auth_service.register(req).await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"message": "User registered successfully"})),
    ))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let response = state.auth_service.login(req, &state.encoding_key).await?;
    Ok(Json(response))
}

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let response = state.auth_service.create_api_key(user.id, req).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let keys = state.auth_service.list_api_keys(user.id).await?;
    Ok(Json(keys))
}

pub async fn revoke_api_key(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    state.auth_service.revoke_api_key(user.id, id).await?;
    Ok(Json(json!({"message": "API key revoked"})))
}

pub async fn validate_api_key(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ValidateApiKeyRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let response = state.auth_service.validate_api_key(&req.api_key).await?;
    Ok(Json(response))
}
