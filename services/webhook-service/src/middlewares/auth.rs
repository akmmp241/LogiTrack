use crate::handlers::biteship_handler::BiteshipHandler;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use std::sync::Arc;

pub async fn biteship_auth_middleware(
    State(handler): State<Arc<BiteshipHandler>>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let auth_header = request
        .headers()
        .get(handler.get_webhook_key())
        .and_then(|header| header.to_str().ok());

    let verified = if let Some(token) = auth_header {
        token == handler.get_webhook_secret()
    } else {
        false
    };

    if !verified {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    next.run(request).await
}
