use crate::repositories::billing_repository::BillingRepository;
use crate::repositories::transaction_repository::TransactionRepository;
use crate::repositories::user_repo::UserRepository;
use crate::router::create_routes;
use crate::services::topup_service::TopupService;
use crate::services::transaction_service::TransactionService;
use config::postgres::get_db_connection;
use jsonwebtoken::{DecodingKey, EncodingKey};
use payment::xendit::XenditProvider;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct App {
    state: Arc<AppState>,
}

#[derive(Clone)]
pub struct AppState {
    pub topup_service: TopupService,
    pub transaction_service: TransactionService,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

impl App {
    pub async fn new() -> Self {
        let db = get_db_connection()
            .await
            .expect("Failed to get DB connection");

        let redis = config::redis::create_redis_pool().await;

        let reqwest = config::reqwest::get_reqwest_pool().expect("Failed to retrieve reqwest pool");

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

        let billing_repository = BillingRepository::new(db.clone());
        let transaction_repository = TransactionRepository::new(db.clone());
        let user_repository = UserRepository::new(db.clone());

        let xendit_uc = Arc::new(XenditProvider::new(reqwest));

        let topup_service = TopupService::new(
            billing_repository,
            redis,
            user_repository,
            transaction_repository.clone(),
            xendit_uc.clone(),
        );
        let transaction_service = TransactionService::new(transaction_repository, xendit_uc);

        let app_state = AppState {
            topup_service,
            transaction_service,
            encoding_key,
            decoding_key,
        };

        Self {
            state: Arc::new(app_state),
        }
    }

    pub async fn run(&self) {
        let port = std::env::var("TOPUP_SERVICE_PORT").expect("TOPUP_SERVICE_PORT must be set");

        let app = create_routes(self.state.clone());

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .expect("Could not bind to port");

        tracing::info!("topup-service listening on :{}", port);

        axum::serve(listener, app)
            .await
            .expect("Failed to run server");
    }
}
