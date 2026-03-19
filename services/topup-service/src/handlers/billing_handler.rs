use crate::app::AppState;
use crate::domain::dto::TopupRequest;
use crate::middleware::CurrentUser;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use errors::error::HttpError;
use std::sync::Arc;

pub async fn get_wallet(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let res = state.topup_service.get_wallet(user.id).await?;

    Ok((StatusCode::OK, Json(res)))
}

pub async fn topup(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
    req: Result<Json<TopupRequest>, JsonRejection>,
) -> Result<impl IntoResponse, HttpError> {
    let Json(request) = req?;

    let res = state.topup_service.topup(user.id, request).await?;

    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn get_transactions(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let res = state.topup_service.get_transactions(user.id).await?;

    Ok((StatusCode::OK, Json(res)))
}
