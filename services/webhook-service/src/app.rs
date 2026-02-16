use crate::handlers::biteship_handler::BiteshipHandler;
use crate::routes::register_biteship_routes;
use crate::services::biteship_service::BiteshipService;
use axum::Router;
use config::postgres::get_db_connection;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::classify::ServerErrorsFailureClass::StatusCode;
use tower_http::trace;
use trace::TraceLayer;

pub struct App {
    biteship_handler: Arc<BiteshipHandler>,
}

impl App {
    pub async fn new() -> Self {
        let db = Arc::new(
            get_db_connection()
                .await
                .expect("Failed to connect to database"),
        );

        let biteship_service = Arc::new(BiteshipService::new(db));

        let biteship_handler = Arc::new(BiteshipHandler::new(biteship_service));

        Self { biteship_handler }
    }

    pub async fn run(&self) {
        let biteship_router = register_biteship_routes(self.biteship_handler.clone());

        let app = Router::new()
            .route(
                "/",
                axum::routing::post(|| async { axum::http::StatusCode::OK }),
            )
            .nest("/api/webhook", biteship_router);

        let listener = TcpListener::bind("0.0.0.0:3000")
            .await
            .expect("could not bind listener");

        tracing::info!("Listening on http://{}", listener.local_addr().unwrap());
        axum::serve(listener, app)
            .await
            .expect("could not start server");
    }
}
