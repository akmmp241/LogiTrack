use crate::app::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, Header, Validation};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    jti: String,
    // scopes
    scp: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InternalClaims {
    sub: String,
    user_id: String,
    exp: usize,
    iat: usize,
    scp: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ValidateApiKeyResponse {
    valid: bool,
    user_id: Option<String>,
    scopes: Option<Vec<String>>,
    client_id: Option<String>,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    if let Some(auth_header) = req.headers().get("authorization") {
        let auth_str = auth_header.to_str().unwrap_or("");

        if let Some(_token) = auth_str.strip_prefix("Bearer ") {
            // only get the token from bearer token
            let token = &auth_str[7..];

            return match validate_jwt(token, &state) {
                Ok(claims) => {
                    let user_id = claims.sub;
                    let scopes = claims.scp;

                    let internal_claims = generate_internal_claims("user", &user_id, scopes);

                    let jwt = generate_internal_jwt(&internal_claims, &state)
                        .map_err(|e| internal_error(&e))?;

                    req.headers_mut()
                        .insert("Authorization", format!("Bearer {}", jwt).parse().unwrap());

                    Ok(next.run(req).await)
                }
                Err(e) => Err(unauthorized_response(&e)),
            };
        }
    }

    if let Some(api_key_header) = req.headers().get("x-api-key") {
        let api_key = api_key_header
            .to_str()
            .map_err(|_| unauthorized_response("Invalid API key header"))?
            .to_string();

        return match validate_api_key(&api_key, &state).await {
            Ok(key) => {
                let user_id = key
                    .user_id
                    .ok_or_else(|| unauthorized_response("Missing user_id"))?;

                let scopes = key
                    .scopes
                    .ok_or_else(|| unauthorized_response("Missing scopes"))?;

                let internal_claims = generate_internal_claims("machine", &user_id, scopes);

                let jwt = generate_internal_jwt(&internal_claims, &state)
                    .map_err(|e| internal_error(&e))?;

                req.headers_mut()
                    .insert("Authorization", format!("Bearer {}", jwt).parse().unwrap());

                Ok(next.run(req).await)
            }
            Err(e) => Err(unauthorized_response(&e)),
        };
    }

    Err(unauthorized_response(
        "No authentication credentials provided",
    ))
}

fn validate_jwt(token: &str, state: &AppState) -> Result<Claims, String> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;

    let token_data = jsonwebtoken::decode::<Claims>(token, &state.jwt_decoding_key, &validation)
        .map_err(|e| format!("Invalid JWT: {}", e))?;

    Ok(token_data.claims)
}

async fn validate_api_key(
    api_key: &str,
    state: &AppState,
) -> Result<ValidateApiKeyResponse, String> {
    let cache_key = format!("apikey:{}", &api_key[..8.min(api_key.len())]);

    // check Redis cache first
    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await
        && let Ok(cached) = conn.get::<&str, String>(&cache_key).await
    {
        if cached == "invalid" {
            return Err("Invalid API key".to_string());
        }

        let api_key = serde_json::from_str::<ValidateApiKeyResponse>(&cached)
            .map_err(|e| format!("Failed to parse API key: {}", e))?;

        return Ok(api_key);
    }

    // call to auth service
    let url = format!("{}/internal/validate-api-key", state.auth_service_url);
    let response = state
        .client
        .post(&url)
        .json(&json!({"api_key": api_key}))
        .send()
        .await
        .map_err(|e| format!("Auth service error: {}", e))?;

    if !response.status().is_success() {
        return Err("Invalid api key".to_string());
    }

    let body: ValidateApiKeyResponse = response
        .json()
        .await
        .map_err(|e| format!("Invalid response from auth service: {}", e))?;

    if !body.valid {
        if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("SETEX")
                .arg(&cache_key)
                .arg(60)
                .arg("invalid")
                .query_async(&mut conn)
                .await;
        }
        return Err("Invalid API key".to_string());
    }

    let api_key_info =
        serde_json::to_string(&body).map_err(|e| format!("Invalid API key: {}", e))?;

    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let _: Result<(), _> = redis::cmd("SETEX")
            .arg(&cache_key)
            .arg(300)
            .arg(&api_key_info)
            .query_async(&mut conn)
            .await;
    }

    Ok(body)
}

fn unauthorized_response(message: &str) -> Response {
    let body = serde_json::to_string(&json!({"error": message})).unwrap_or_default();
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

fn internal_error(message: &str) -> Response {
    tracing::error!("{}", message);
    let body = serde_json::to_string(&json!({"error": "Something went wrong"})).unwrap_or_default();
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

fn generate_internal_jwt(claims: &InternalClaims, state: &AppState) -> Result<String, String> {
    let header = Header::new(Algorithm::RS256);
    let token = jsonwebtoken::encode(&header, claims, &state.jwt_encoding_key)
        .map_err(|e| format!("Failed to generate JWT: {}", e))?;

    Ok(token)
}

fn generate_internal_claims(sub: &str, user_id: &str, scp: Vec<String>) -> InternalClaims {
    InternalClaims {
        sub: sub.to_string(),
        user_id: user_id.to_string(),
        scp,
        exp: (Utc::now() + Duration::minutes(5)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    }
}
