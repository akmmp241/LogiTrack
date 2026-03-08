use crate::repository::notification_log_repo::NotificationLogRepository;
use crate::repository::shipment_repo::ShipmentRepository;
use crate::repository::shipment_status_mapping_repo::ShipmentStatusMappingRepository;
use crate::repository::shipment_subscription::ShipmentSubsRepository;
use crate::repository::tracking_event_repo::TrackingEventRepo;
use crate::repository::tracking_job_repo::TrackingJobRepository;
use crate::repository::user_repo::UserRepository;
use crate::routes::routes;
use crate::service::tracking_service::{Repositories, TrackingService};
use axum::Router;
use biteship::BiteshipUseCase;
use config::postgres::get_db_connection;
use config::rabbitmq::create_channel;
use config::reqwest::get_reqwest_pool;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub struct App {
    state: Arc<AppState>,
}

#[derive(Clone)]
pub struct AppState {
    pub service: TrackingService,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

impl App {
    pub async fn new() -> Self {
        let private_key_path = std::env::var("JWT_PRIVATE_KEY_PATH")
            .unwrap_or_else(|_| "./keys/private.pem".to_string());
        let public_key_path = std::env::var("JWT_PUBLIC_KEY_PATH")
            .unwrap_or_else(|_| "./keys/public.pem".to_string());

        let private_key_pem = std::fs::read(&private_key_path)
            .unwrap_or_else(|_| panic!("Failed to read private key from {}", private_key_path));
        let public_key_pem = std::fs::read(&public_key_path)
            .unwrap_or_else(|_| panic!("Failed to read public key from {}", public_key_path));

        let encoding_key =
            EncodingKey::from_rsa_pem(&private_key_pem).expect("Invalid RSA private key");
        let decoding_key =
            DecodingKey::from_rsa_pem(&public_key_pem).expect("Invalid RSA public key");

        let db = get_db_connection()
            .await
            .expect("couldn't connect to database");

        let pool = get_reqwest_pool().expect("couldn't create reqwest pool");

        let rabbitmq_channel = create_channel()
            .await
            .expect("couldn't create rabbitmq channel");

        let repos = Repositories {
            shipment_repository: ShipmentRepository::new(db.clone()).await,
            shipment_subs_repo: ShipmentSubsRepository::new(db.clone()).await,
            map_status_repo: ShipmentStatusMappingRepository::new(db.clone()).await,
            tracking_event_repo: TrackingEventRepo::new(db.clone()).await,
            tracking_job_repo: TrackingJobRepository::new(db.clone()),
            user_repo: UserRepository::new(db.clone()),
            notification_log_repo: NotificationLogRepository::new(db.clone()),
        };

        let bs_uc = BiteshipUseCase::new(pool);

        let service = TrackingService::new(repos, bs_uc, rabbitmq_channel).await;

        let state = Arc::new(AppState {
            service,
            encoding_key,
            decoding_key,
        });

        Self { state }
    }

    pub async fn run(&self) {
        let port = std::env::var("TRACKING_SERVICE_PORT").unwrap_or_else(|_| "3002".to_string());
        let addr = format!("0.0.0.0:{}", port);

        let router = Router::new().merge(routes(self.state.clone()));

        let listener = TcpListener::bind(&addr)
            .await
            .expect("could not bind listener");

        info!("Listening on http://{}", listener.local_addr().unwrap());
        axum::serve(listener, router)
            .await
            .expect("could not start server");
    }
}
