use crate::app::AppState;
use axum::{Json, extract::State, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;

pub async fn get_notification_prices(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.notification_price_service.rehydrate_cache().await {
        Ok(prices) => Json(json!({
            "status": "success",
            "data": prices
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Failed to rehydrate notification prices: {:?}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": "Failed to retrieve notification prices"
                })),
            )
                .into_response()
        }
    }
}
