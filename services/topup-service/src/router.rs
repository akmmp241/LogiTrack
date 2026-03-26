use crate::app::AppState;
use crate::handlers::{billing_handler, internal_handler, transaction_handler};
use crate::middleware;
use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn create_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(billing_routes(state.clone()))
        .merge(transaction_routes(state.clone()))
        .merge(internal_routes(state.clone()))
}

fn billing_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/billing/wallet", get(billing_handler::get_wallet))
        .route("/api/billing/topup", post(billing_handler::topup))
        .route(
            "/api/billing/transactions",
            get(billing_handler::get_transactions),
        )
        .layer(from_fn_with_state(
            vec!["billing.manage".into()],
            middleware::require_scopes,
        ))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ))
        .with_state(state)
}

fn transaction_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/api/transactions",
            get(transaction_handler::get_transactions),
        )
        .route(
            "/api/transactions/{id}",
            get(transaction_handler::get_transaction),
        )
        // only for development phase
        .route(
            "/api/transactions/{id}/simulate",
            post(transaction_handler::simulate_transaction),
        )
        .layer(from_fn_with_state(
            vec!["billing.manage".into()],
            middleware::require_scopes,
        ))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ))
        .with_state(state)
}

fn internal_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/internal/api/pricing/notifications",
            get(internal_handler::get_notification_prices),
        )
        .with_state(state)
}
