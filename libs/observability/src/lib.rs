use axum::extract::MatchedPath;
use axum::{http::Request, response::Response};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::Lazy;
use std::time::Instant;
use axum::body::Body;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

static METRICS_HANDLE: Lazy<PrometheusHandle> = Lazy::new(|| {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder")
});

pub fn init(service_name: &str) {
    // Initialize Tracing
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    // Force initialization of metrics handle
    let _ = &*METRICS_HANDLE;

    tracing::info!("{} observability initialized", service_name);
}

pub async fn metrics_middleware(req: Request<Body>, next: axum::middleware::Next) -> Response
{
    let start = Instant::now();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str())
        .unwrap_or("/")
        .to_owned();
    let method = req.method().to_string();

    let response = next.run(req).await;
    
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method),
        ("path", path),
        ("status", status),
    ];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_request_duration_seconds", &labels).record(latency);

    response
}

pub async fn metrics_handler() -> String {
    METRICS_HANDLE.render()
}

