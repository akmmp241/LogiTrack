use crate::app::AppState;
use crate::handlers::notification_log::{
    get_shipment_notification_log_by_id, get_shipment_notification_logs,
};
use crate::handlers::tracking::{
    create_shipments, delete_shipment_by_id, get_shipment_by_id, get_shipment_events, get_shipments,
};
use axum::Router;
use axum::routing::{delete, get, post};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/shipments", post(create_shipments))
        .route("/api/shipments", get(get_shipments))
        .route("/api/shipments/{id}", get(get_shipment_by_id))
        .route("/api/shipments/{id}", delete(delete_shipment_by_id))
        .route("/api/shipments/{id}/events", get(get_shipment_events))
        .route(
            "/api/shipments/{id}/notifications",
            get(get_shipment_notification_logs),
        )
        .route(
            "/api/shipments/{id}/notifications/{notification_id}",
            get(get_shipment_notification_log_by_id),
        )
        .with_state(state)
}
