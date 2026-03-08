use crate::app::AppState;
use crate::middlewares::CurrentUser;
use crate::models::notif_preferences::UpdateNotifPrefRequest;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use errors::error::HttpError;
use serde_json::json;
use std::sync::Arc;

pub async fn get_current_notif_preferences(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
) -> Result<impl IntoResponse, HttpError> {
    let res = state
        .notif_pref_service
        .get_current_preferences(user.id)
        .await?;

    Ok(Json(res))
}

pub async fn update_notif_pref(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<CurrentUser>,
    payload: Result<Json<UpdateNotifPrefRequest>, JsonRejection>,
) -> Result<impl IntoResponse, HttpError> {
    let req = payload?;

    state
        .notif_pref_service
        .update_notif_pref(user.id, req.channels.clone())
        .await?;

    Ok(Json(
        json!({"message": "User notification preferences updated successfully"}),
    ))
}
