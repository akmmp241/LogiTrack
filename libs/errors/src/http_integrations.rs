use crate::error::HttpError;
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg),
            Self::InternalServerError(err) => {
                tracing::error!("Internal Error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<JsonRejection> for HttpError {
    fn from(re: JsonRejection) -> Self {
        Self::BadRequest(re.body_text())
    }
}
