use crate::handlers::biteship_handler::BiteshipHandler;
use crate::handlers::xendit_handler::XenditHandler;
use crate::middlewares::auth::auth_from_header;
use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::post;
use std::sync::Arc;

pub fn register_biteship_routes(handler: Arc<BiteshipHandler>) -> Router {
    Router::new()
        .route(
            "/api/webhooks/biteship/status",
            post(BiteshipHandler::status_change),
        )
        .route_layer(from_fn_with_state(handler.clone(), auth_from_header))
        .with_state(handler)
}

pub fn register_xendit_routes(handler: Arc<XenditHandler>) -> Router {
    Router::new()
        .route(
            "/api/webhooks/xendit/payment-capture",
            post(XenditHandler::payment_capture),
        )
        .route(
            "/api/webhooks/xendit/pr-expiry",
            post(XenditHandler::pr_expiry),
        )
        .route_layer(from_fn_with_state(handler.clone(), auth_from_header))
        .with_state(handler)
}
