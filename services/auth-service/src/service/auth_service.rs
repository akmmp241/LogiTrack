use crate::models::api_key::{
    ApiKeyInfo, CreateApiKeyRequest, CreateApiKeyResponse, ValidateApiKeyResponse,
};
use crate::models::user::{Claims, LoginRequest, LoginResponse, RegisterRequest};
use crate::repository::api_key_repo::ApiKeyRepository;
use crate::repository::user_notif_preference_repo::UserNotifPreferenceRepository;
use crate::repository::user_repo::UserRepository;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use errors::error::HttpError;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use sha2::{Digest, Sha256};
use std::convert::Into;
use std::sync::LazyLock;
use uuid::Uuid;

pub static USER_SCOPES: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "api-key.manage".into(),
        "notif-pref.manage".into(),
        "shipment.manage".into(),
    ]
});

pub static APIKEY_SCOPES: LazyLock<Vec<String>> =
    LazyLock::new(|| vec!["tracking.manage".into(), "shipment.manage".into()]);

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    api_key_repo: ApiKeyRepository,
    notif_pref_repo: UserNotifPreferenceRepository,
    jwt_expiration: u64,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        api_key_repo: ApiKeyRepository,
        notif_pref_repo: UserNotifPreferenceRepository,
        jwt_expiration: u64,
    ) -> Self {
        Self {
            user_repo,
            api_key_repo,
            notif_pref_repo,
            jwt_expiration,
        }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<(), HttpError> {
        if self
            .user_repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?
            .is_some()
        {
            return Err(HttpError::Conflict("Email already registered".to_string()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| {
                HttpError::InternalServerError(anyhow::anyhow!("Password hash error: {}", e))
            })?
            .to_string();

        let user = self
            .user_repo
            .create_user(&req.email, &password_hash, &req.name, &req.phone_number)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?;

        if !self
            .notif_pref_repo
            .create_notif_preferences(user.id)
            .await
            .map_err(|e| {
                tracing::warn!(
                    "Notification Preferences not created for user id {}",
                    user.id
                );
                HttpError::InternalServerError(e.into())
            })?
        {
            tracing::warn!(
                "Notification Preferences not created for user id {}",
                user.id
            );
        }

        Ok(())
    }

    pub async fn login(
        &self,
        req: LoginRequest,
        encoding_key: &EncodingKey,
    ) -> Result<LoginResponse, HttpError> {
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?
            .ok_or_else(|| HttpError::Unauthorized("Invalid email or password".to_string()))?;

        let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
            HttpError::InternalServerError(anyhow::anyhow!("Hash parse error: {}", e))
        })?;

        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| HttpError::Unauthorized("Invalid email or password".to_string()))?;

        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user.id.to_string(),
            exp: now + self.jwt_expiration as usize,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            scp: USER_SCOPES.to_vec(),
        };

        let header = Header::new(Algorithm::RS256);
        let token = jsonwebtoken::encode(&header, &claims, encoding_key).map_err(|e| {
            HttpError::InternalServerError(anyhow::anyhow!("JWT encode error: {}", e))
        })?;

        Ok(LoginResponse {
            access_token: token,
            expires_in: self.jwt_expiration,
        })
    }

    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        req: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse, HttpError> {
        let raw_key = Uuid::new_v4().to_string() + "-" + &Uuid::new_v4().to_string();

        let hashed = hash_api_key(&raw_key);

        let api_key = self
            .api_key_repo
            .create(user_id, &req.name, &hashed, APIKEY_SCOPES.clone())
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?;

        Ok(CreateApiKeyResponse {
            api_key: raw_key,
            client_id: api_key.id,
        })
    }

    pub async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKeyInfo>, HttpError> {
        let keys = self
            .api_key_repo
            .list_by_user(user_id)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?;

        Ok(keys
            .into_iter()
            .map(|k| ApiKeyInfo {
                id: k.id,
                name: k.name,
                active: k.active,
                created_at: k.created_at,
            })
            .collect())
    }

    pub async fn revoke_api_key(&self, user_id: Uuid, key_id: Uuid) -> Result<(), HttpError> {
        let revoked = self
            .api_key_repo
            .revoke(key_id, user_id)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?;

        if !revoked {
            return Err(HttpError::NotFound("API key not found".to_string()));
        }

        Ok(())
    }

    pub async fn validate_api_key(
        &self,
        raw_key: &str,
    ) -> Result<ValidateApiKeyResponse, HttpError> {
        let hashed = hash_api_key(raw_key);

        let result = self
            .api_key_repo
            .find_active_by_hash(&hashed)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?;

        match result {
            Some(key) => Ok(ValidateApiKeyResponse {
                valid: true,
                user_id: Some(key.user_id),
                scopes: Some(key.scopes),
                client_id: Some(key.id),
            }),
            None => Ok(ValidateApiKeyResponse {
                valid: false,
                user_id: None,
                scopes: None,
                client_id: None,
            }),
        }
    }
}

fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}
