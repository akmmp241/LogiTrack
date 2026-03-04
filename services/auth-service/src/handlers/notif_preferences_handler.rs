use crate::app::AppState;
use crate::helper::extract_user_id;
use crate::models::notif_preferences::UpdateNotifPrefRequest;
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use errors::error::HttpError;
use serde_json::json;
use std::sync::Arc;

pub async fn get_current_notif_preferences(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = extract_user_id(&headers)?;

    let res = state
        .notif_pref_service
        .get_current_preferences(user_id)
        .await?;

    Ok(Json(res))
}

pub async fn update_notif_pref(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    payload: Result<Json<UpdateNotifPrefRequest>, JsonRejection>,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = extract_user_id(&headers)?;

    let req = payload?;

    state
        .notif_pref_service
        .update_notif_pref(user_id, req.channels.clone())
        .await?;

    Ok(Json(
        json!({"message": "User notification preferences updated successfully"}),
    ))
}
