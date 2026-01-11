use crate::repository::shipment_repo::ShipmentRepository;
use crate::repository::shipment_status_mapping_repo::ShipmentStatusMappingRepository;
use crate::repository::shipment_subscription::ShipmentSubsRepository;
use crate::routes::routes;
use crate::service::tracking_service::TrackingService;
use axum::Router;
use biteship::BiteshipUseCase;
use config::postgres::get_db_connection;
use config::rabbitmq::create_channel;
use config::reqwest::get_reqwest_pool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub struct App {
    state: Arc<AppState>,
}

#[derive(Clone)]
pub struct AppState {
    pub service: TrackingService,
}

impl App {
    pub async fn new() -> Self {
        let db = get_db_connection()
            .await
            .expect("couldn't connect to database");

        let pool = get_reqwest_pool().expect("couldn't create reqwest pool");

        let rabbitmq_channel = create_channel()
            .await
            .expect("couldn't create rabbitmq channel");

        let repo = ShipmentRepository::new(db.clone()).await;
        let map_repo = ShipmentStatusMappingRepository::new(db.clone()).await;
        let shipment_subs_repo = ShipmentSubsRepository::new(db.clone()).await;

        let bs_uc = BiteshipUseCase::new(pool);

        let service =
            TrackingService::new(repo, shipment_subs_repo, map_repo, bs_uc, rabbitmq_channel).await;

        let state = Arc::new(AppState { service });

        Self { state }
    }

    pub async fn run(&self) {
        let router = Router::new().merge(routes(self.state.clone()));

        let listener = TcpListener::bind("0.0.0.0:3000")
            .await
            .expect("could not bind listener");

        info!("Listening on http://{}", listener.local_addr().unwrap());
        axum::serve(listener, router)
            .await
            .expect("could not start server");
    }
}
