use axum::http::HeaderMap;
use errors::error::HttpError;
use uuid::Uuid;

pub fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, HttpError> {
    let user_id_str = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| HttpError::Unauthorized("Missing x-user-id header".to_string()))?;

    Uuid::parse_str(user_id_str)
        .map_err(|_| HttpError::BadRequest("Invalid user ID format".to_string()))
}
