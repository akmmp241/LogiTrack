use crate::app::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{Algorithm, Validation};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    jti: String,
}

#[derive(Debug, Deserialize)]
struct ValidateApiKeyResponse {
    valid: bool,
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
                Ok(user_id) => {
                    req.headers_mut()
                        .insert("x-identity-type", "user".parse().unwrap());
                    req.headers_mut()
                        .insert("x-user-id", user_id.parse().unwrap());
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
            Ok(client_id) => {
                req.headers_mut()
                    .insert("x-identity-type", "machine".parse().unwrap());
                req.headers_mut()
                    .insert("x-client-id", client_id.parse().unwrap());
                Ok(next.run(req).await)
            }
            Err(e) => Err(unauthorized_response(&e)),
        };
    }

    Err(unauthorized_response(
        "No authentication credentials provided",
    ))
}

fn validate_jwt(token: &str, state: &AppState) -> Result<String, String> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.set_required_spec_claims(&["sub", "exp", "iat"]);

    let token_data = jsonwebtoken::decode::<Claims>(token, &state.jwt_decoding_key, &validation)
        .map_err(|e| format!("Invalid JWT: {}", e))?;

    Ok(token_data.claims.sub)
}

async fn validate_api_key(api_key: &str, state: &AppState) -> Result<String, String> {
    let cache_key = format!("apikey:{}", &api_key[..8.min(api_key.len())]);

    // check Redis cache first
    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await
        && let Ok(cached) = conn.get::<&str, String>(&cache_key).await
    {
        if cached == "invalid" {
            return Err("Invalid API key".to_string());
        }
        return Ok(cached);
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
        return Err("Auth service validation failed".to_string());
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

    let client_id = body.client_id.unwrap_or_default();

    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let _: Result<(), _> = redis::cmd("SETEX")
            .arg(&cache_key)
            .arg(300)
            .arg(&client_id)
            .query_async(&mut conn)
            .await;
    }

    Ok(client_id)
}

fn unauthorized_response(message: &str) -> Response {
    let body = serde_json::to_string(&json!({"error": message})).unwrap_or_default();
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap()
}
