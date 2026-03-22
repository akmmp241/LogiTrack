use crate::handlers::biteship_handler::BiteshipHandler;
use crate::handlers::xendit_handler::XenditHandler;
use crate::routes::{register_biteship_routes, register_xendit_routes};
use crate::services::biteship_service::BiteshipService;
use crate::services::xendit_service::XenditService;
use axum::Router;
use config::postgres::get_db_connection;
use config::rabbitmq::create_channel;
use config::redis::create_redis_pool;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct App {
    biteship_handler: Arc<BiteshipHandler>,
    xendit_handler: Arc<XenditHandler>,
}

impl App {
    pub async fn new() -> Self {
        let db = Arc::new(
            get_db_connection()
                .await
                .expect("Failed to connect to database"),
        );

        let rabbitmq_channel = create_channel()
            .await
            .expect("couldn't create rabbitmq channel");

        let redis_pool = create_redis_pool().await;

        let biteship_service = Arc::new(BiteshipService::new(db.clone(), rabbitmq_channel));
        let biteship_handler = Arc::new(BiteshipHandler::new(biteship_service));

        let xendit_service = Arc::new(XenditService::new(db, redis_pool));
        let xendit_handler = Arc::new(XenditHandler::new(xendit_service));

        Self {
            biteship_handler,
            xendit_handler,
        }
    }

    pub async fn run(&self) {
        let port = std::env::var("WEBHOOK_SERVICE_PORT").unwrap_or_else(|_| "3003".to_string());
        let addr = format!("0.0.0.0:{}", port);

        let biteship_router = register_biteship_routes(self.biteship_handler.clone());
        let xendit_router = register_xendit_routes(self.xendit_handler.clone());

        let app = Router::new()
            .route(
                "/",
                axum::routing::post(|| async { axum::http::StatusCode::OK }),
            )
            .merge(biteship_router)
            .merge(xendit_router);

        let listener = TcpListener::bind(&addr)
            .await
            .expect("could not bind listener");

        tracing::info!("Listening on http://{}", listener.local_addr().unwrap());
        axum::serve(listener, app)
            .await
            .expect("could not start server");
    }
}
