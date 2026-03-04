use crate::repository::api_key_repo::ApiKeyRepository;
use crate::repository::user_notif_preference_repo::UserNotifPreferenceRepository;
use crate::repository::user_repo::UserRepository;
use crate::routes::routes;
use crate::service::auth_service::AuthService;
use crate::service::user_notif_preferences_service::UserNotificationPreferencesService;
use axum::Router;
use config::postgres::get_db_connection;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

pub struct App {
    state: Arc<AppState>,
}

#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthService,
    pub notif_pref_service: UserNotificationPreferencesService,
}

impl App {
    pub async fn new() -> Self {
        let db = get_db_connection()
            .await
            .expect("couldn't connect to database");

        let private_key_path = std::env::var("JWT_PRIVATE_KEY_PATH")
            .unwrap_or_else(|_| "./keys/private.pem".to_string());
        let public_key_path = std::env::var("JWT_PUBLIC_KEY_PATH")
            .unwrap_or_else(|_| "./keys/public.pem".to_string());
        let jwt_expiration: u64 = std::env::var("JWT_EXPIRATION_SECS")
            .unwrap_or_else(|_| "900".to_string())
            .parse()
            .expect("JWT_EXPIRATION_SECS must be a number");

        let private_key_pem = std::fs::read(&private_key_path)
            .unwrap_or_else(|_| panic!("Failed to read private key from {}", private_key_path));
        let public_key_pem = std::fs::read(&public_key_path)
            .unwrap_or_else(|_| panic!("Failed to read public key from {}", public_key_path));

        let encoding_key =
            EncodingKey::from_rsa_pem(&private_key_pem).expect("Invalid RSA private key");
        let decoding_key =
            DecodingKey::from_rsa_pem(&public_key_pem).expect("Invalid RSA public key");

        let user_repo = UserRepository::new(db.clone());
        let api_key_repo = ApiKeyRepository::new(db.clone());
        let notif_pref_repo = UserNotifPreferenceRepository::new(db.clone());

        let auth_service = AuthService::new(
            user_repo,
            api_key_repo,
            notif_pref_repo.clone(),
            encoding_key,
            decoding_key,
            jwt_expiration,
        );
        let notif_pref_service = UserNotificationPreferencesService::new(notif_pref_repo);

        let state = Arc::new(AppState {
            auth_service,
            notif_pref_service,
        });

        Self { state }
    }

    pub async fn run(&self) {
        let port = std::env::var("AUTH_SERVICE_PORT").unwrap_or_else(|_| "3001".to_string());
        let addr = format!("0.0.0.0:{}", port);

        let router = Router::new().merge(routes(self.state.clone()));

        let listener = TcpListener::bind(&addr)
            .await
            .expect("could not bind listener");

        info!(
            "Auth service listening on http://{}",
            listener.local_addr().unwrap()
        );
        axum::serve(listener, router)
            .await
            .expect("could not start server");
    }
}
