use crate::app::AppState;
use axum::Extension;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{Algorithm, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InternalClaims {
    sub: String,
    user_id: String,
    exp: usize,
    iat: usize,
    scp: Vec<String>,
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        let auth_str = auth_header.to_str().unwrap_or("");

        if let Some(_token) = auth_str.strip_prefix("Bearer ") {
            // only get the token from bearer token
            let token = &auth_str[7..];

            return match validate_jwt(token, &state) {
                Ok(claims) => {
                    let user_id = Uuid::from_str(&claims.user_id).map_err(|e| {
                        tracing::error!("failed to parse user id to uuid");
                        internal_error(e.to_string().as_str())
                    })?;

                    let scopes = claims.scp;

                    let user = CurrentUser {
                        id: user_id,
                        scopes,
                    };

                    req.extensions_mut().insert(user);

                    Ok(next.run(req).await)
                }
                Err(e) => Err(unauthorized_response(&e)),
            };
        };
    };

    Err(unauthorized_response(
        "No authentication credentials provided",
    ))
}

pub async fn require_scopes(
    State(state): State<Vec<String>>,
    Extension(user): Extension<CurrentUser>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    if user.scopes.is_empty() {
        return Err(unauthorized_response("No scopes provided"));
    }

    if user.scopes.iter().any(|scope| state.contains(scope)) {
        return Ok(next.run(req).await);
    }

    Err(unauthorized_response("Invalid scopes provided"))
}

fn validate_jwt(token: &str, state: &AppState) -> Result<InternalClaims, String> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;

    let token_data =
        jsonwebtoken::decode::<InternalClaims>(token, &state.decoding_key, &validation)
            .map_err(|e| format!("Invalid JWT: {}", e))?;

    Ok(token_data.claims)
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
