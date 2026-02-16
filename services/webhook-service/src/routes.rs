use crate::handlers::biteship_handler::BiteshipHandler;
use crate::middlewares::auth::biteship_auth_middleware;
use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::post;
use std::sync::Arc;

pub fn register_biteship_routes(handler: Arc<BiteshipHandler>) -> Router {
    Router::new()
        .route("/biteship/status", post(BiteshipHandler::status_change))
        .route_layer(from_fn_with_state(
            handler.clone(),
            biteship_auth_middleware,
        ))
        .with_state(handler)
}
