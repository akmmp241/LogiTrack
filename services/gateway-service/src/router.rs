use crate::app::AppState;
use crate::middleware::auth_middleware;
use crate::proxy::reverse_proxy;
use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::middleware as axum_mw;
use axum::routing::any;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct ServiceUrls {
    pub auth_url: String,
    pub tracking_url: String,
    pub webhook_url: String,
}

impl ServiceUrls {
    pub fn from_env() -> Self {
        Self {
            auth_url: std::env::var("AUTH_SERVICE_URL").expect("AUTH_SERVICE_URL must be set"),
            tracking_url: std::env::var("TRACKING_SERVICE_URL")
                .expect("TRACKING_SERVICE_URL must be set"),
            webhook_url: std::env::var("WEBHOOK_SERVICE_URL")
                .expect("WEBHOOK_SERVICE_URL must be set"),
        }
    }
}

pub fn create_router(urls: ServiceUrls, state: AppState) -> Router {
    let public_auth_routes = Router::new()
        .route("/api/auth/register", {
            let target = urls.auth_url.clone();
            let state = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state, req, target))
        })
        .route("/api/auth/login", {
            let target = urls.auth_url.clone();
            let state = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state, req, target))
        })
        .route("/api/webhooks/{*path}", {
            let target = urls.webhook_url.clone();
            let state_inner = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state_inner, req, target))
        });

    let protected_routes = Router::new()
        .route("/api/auth/{*path}", {
            let target = urls.auth_url.clone();
            let state_inner = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state_inner, req, target))
        })
        .route("/api/user/{*path}", {
            let target = urls.auth_url.clone();
            let state_inner = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state_inner, req, target))
        })
        .route("/api/shipments/{*path}", {
            let target = urls.tracking_url.clone();
            let state_inner = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state_inner, req, target))
        })
        .layer(axum_mw::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .merge(public_auth_routes)
        .merge(protected_routes)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
