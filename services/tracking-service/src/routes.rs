use crate::app::AppState;
use crate::handlers::tracking::{
    create_shipments, delete_shipment_by_id, get_shipment_by_id, get_shipment_events, get_shipments,
};
use axum::Router;
use axum::routing::{delete, get, post};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/shipments", post(create_shipments))
        .route("/shipments", get(get_shipments))
        .route("/shipments/{id}", get(get_shipment_by_id))
        .route("/shipments/{id}", delete(delete_shipment_by_id))
        .route("/shipments/{id}/events", get(get_shipment_events))
        .with_state(state)
}
