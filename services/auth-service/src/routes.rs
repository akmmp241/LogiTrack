use crate::app::AppState;
use crate::handlers::notif_preferences_handler::update_notif_pref;
use crate::handlers::{auth_handler, notif_preferences_handler};
use crate::middlewares::{auth_middleware, require_scopes};
use auth_handler::{
    create_api_key, list_api_keys, login, register, revoke_api_key, validate_api_key,
};
use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{delete, get, patch, post};
use notif_preferences_handler::get_current_notif_preferences;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(auth_routes(state.clone()))
        .merge(api_keys_routes(state.clone()))
        .merge(user_notif_preference_routes(state.clone()))
        .merge(internal_api(state.clone()))
}

fn auth_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .with_state(state)
}

fn api_keys_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/auth/api-keys", post(create_api_key))
        .route("/api/auth/api-keys", get(list_api_keys))
        .route("/api/auth/api-keys/{id}", delete(revoke_api_key))
        // internal endpoint for gateway API key validation
        .layer(from_fn_with_state(
            vec!["api-key.manage".into()],
            require_scopes,
        ))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

fn user_notif_preference_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/user/preferences/notification",
            get(get_current_notif_preferences),
        )
        .route(
            "/api/user/preferences/notification",
            patch(update_notif_pref),
        )
        .layer(from_fn_with_state(
            vec!["notif-pref.manage".into()],
            require_scopes,
        ))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

fn internal_api(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/internal/validate-api-key", post(validate_api_key))
        .with_state(state)
}
