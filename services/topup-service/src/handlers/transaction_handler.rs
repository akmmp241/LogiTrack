use crate::app::AppState;
use crate::middleware::CurrentUser;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use errors::error::HttpError;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_transactions(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let res = state.transaction_service.get_transactions(user.id).await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    let res = state
        .transaction_service
        .get_transaction(user.id, id)
        .await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn simulate_transaction(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, HttpError> {
    state
        .transaction_service
        .simulate_transaction(user.id, id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(json!({"message": "Simulate payment successfully"})),
    ))
}
