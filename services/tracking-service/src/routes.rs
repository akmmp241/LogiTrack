use crate::app::AppState;
use crate::handlers::tracking::{create_shipments, get_shipment_by_id, get_shipments};
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/shipments", post(create_shipments))
        .route("/shipments", get(get_shipments))
        .route("/shipments/{id}", get(get_shipment_by_id))
        .with_state(state)
}
