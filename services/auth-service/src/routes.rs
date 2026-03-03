use crate::app::AppState;
use crate::handlers::auth_handler;
use axum::Router;
use axum::routing::{delete, get, post};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/auth/register", post(auth_handler::register))
        .route("/api/auth/login", post(auth_handler::login))
        .route("/api/auth/api-keys", post(auth_handler::create_api_key))
        .route("/api/auth/api-keys", get(auth_handler::list_api_keys))
        .route(
            "/api/auth/api-keys/{id}",
            delete(auth_handler::revoke_api_key),
        )
        // internal endpoint for gateway API key validation
        .route(
            "/internal/validate-api-key",
            post(auth_handler::validate_api_key),
        )
        .with_state(state)
}
