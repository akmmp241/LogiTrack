use crate::app::AppState;
use crate::handlers::notification_log::{
    get_shipment_notification_log_by_id, get_shipment_notification_logs,
};
use crate::handlers::tracking::{
    create_shipments, delete_shipment_by_id, get_shipment_by_id, get_shipment_events,
    get_shipment_pref, get_shipments, update_shipment_pref,
};
use crate::middlewares::{auth_middleware, require_scopes};
use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/shipments", post(create_shipments))
        .route("/api/shipments", get(get_shipments))
        .route("/api/shipments/{id}", get(get_shipment_by_id))
        .route("/api/shipments/{id}", delete(delete_shipment_by_id))
        .route("/api/shipments/{id}/events", get(get_shipment_events))
        .route("/api/shipments/{id}/preferences", get(get_shipment_pref))
        .route(
            "/api/shipments/{id}/preferences",
            patch(update_shipment_pref),
        )
        .route(
            "/api/shipments/{id}/notifications",
            get(get_shipment_notification_logs),
        )
        .route(
            "/api/shipments/{id}/notifications/{notification_id}",
            get(get_shipment_notification_log_by_id),
        )
        .layer(from_fn_with_state(
            vec!["shipment.manage".into()],
            require_scopes,
        ))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}
