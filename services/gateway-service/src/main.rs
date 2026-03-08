mod app;
mod middleware;
mod proxy;
mod router;

use crate::app::AppState;
use crate::router::ServiceUrls;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    observability::init("gateway-service");

    let service_urls = ServiceUrls::from_env();
    let state = AppState::new();

    let app = router::create_router(service_urls, state);

    let port = std::env::var("GATEWAY_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind server port");

    tracing::info!(
        "Gateway listening on http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
