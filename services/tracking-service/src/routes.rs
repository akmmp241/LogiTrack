use crate::app::AppState;
use crate::handlers::tracking::create_shipments;
use axum::routing::post;
use axum::Router;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/shipments", post(create_shipments))
        .with_state(state)
}
