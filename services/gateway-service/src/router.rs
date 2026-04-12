use crate::app::AppState;
use crate::middleware::auth_middleware;
use crate::proxy::reverse_proxy;
use axum::Router;
use axum::body::Body;
use axum::http::Request;
use axum::middleware as axum_mw;
use axum::response::Html;
use axum::routing::{any, get};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct ServiceUrls {
    pub auth_url: String,
    pub tracking_url: String,
    pub webhook_url: String,
    pub topup_url: String,
}

impl ServiceUrls {
    pub fn from_env() -> Self {
        Self {
            auth_url: std::env::var("AUTH_SERVICE_URL").expect("AUTH_SERVICE_URL must be set"),
            tracking_url: std::env::var("TRACKING_SERVICE_URL")
                .expect("TRACKING_SERVICE_URL must be set"),
            webhook_url: std::env::var("WEBHOOK_SERVICE_URL")
                .expect("WEBHOOK_SERVICE_URL must be set"),
            topup_url: std::env::var("TOPUP_SERVICE_URL").expect("TOPUP_SERVICE_URL must be set"),
        }
    }
}

const OPENAPI_YAML: &str = include_str!("../../../openapi.yaml");

const SCALAR_UI_HTML: &str = r#"
<!doctype html>
<html>
  <head>
    <title>LogiTrack API Reference</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      body { margin: 0; }
    </style>
  </head>
  <body>
    <script id="api-reference" data-url="/openapi.yaml"></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
  </body>
</html>
"#;

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
        .route("/api/shipments", {
            let target = urls.tracking_url.clone();
            let state_inner = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state_inner, req, target))
        })
        .route("/api/shipments/{*path}", {
            let target = urls.tracking_url.clone();
            let state_inner = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state_inner, req, target))
        })
        .route("/api/billing/{*path}", {
            let target = urls.topup_url.clone();
            let state_inner = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state_inner, req, target))
        })
        .route("/api/transactions", {
            let target = urls.topup_url.clone();
            let state_inner = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state_inner, req, target))
        })
        .route("/api/transactions/{*path}", {
            let target = urls.topup_url.clone();
            let state_inner = state.clone();
            any(move |req: Request<Body>| reverse_proxy(state_inner, req, target))
        })
        .layer(axum_mw::from_fn_with_state(state.clone(), auth_middleware));

    let docs_routes = Router::new()
        .route("/openapi.yaml", get(|| async {
            (
                [("content-type", "text/yaml")],
                OPENAPI_YAML
            )
        }))
        .route("/docs", get(|| async {
            Html(SCALAR_UI_HTML)
        }))
        .route("/swagger", get(|| async {
            Html(SCALAR_UI_HTML)
        }));

    Router::new()
        .route("/metrics", axum::routing::get(observability::metrics_handler))
        .merge(docs_routes)
        .merge(public_auth_routes)
        .merge(protected_routes)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(observability::metrics_middleware))
}

