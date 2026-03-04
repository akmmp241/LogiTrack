use config::reqwest::get_reqwest_pool;
use jsonwebtoken::DecodingKey;
use redis::Client as RedisClient;
use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub redis: RedisClient,
    pub jwt_decoding_key: DecodingKey,
    pub auth_service_url: String,
}

impl AppState {
    pub fn new() -> Self {
        let client = get_reqwest_pool().expect("Failed to create reqwest pool");

        let redis = config::redis::get_redis_client().expect("Failed to create Redis client");

        let public_key_path = std::env::var("JWT_PUBLIC_KEY_PATH")
            .unwrap_or_else(|_| "./keys/public.pem".to_string());
        let public_key_pem = std::fs::read(&public_key_path)
            .unwrap_or_else(|_| panic!("Failed to read public key from {}", public_key_path));
        let jwt_decoding_key =
            DecodingKey::from_rsa_pem(&public_key_pem).expect("Invalid RSA public key");

        let auth_service_url =
            std::env::var("AUTH_SERVICE_URL").expect("AUTH_SERVICE_URL must be set");

        Self {
            client,
            redis,
            jwt_decoding_key,
            auth_service_url,
        }
    }
}
