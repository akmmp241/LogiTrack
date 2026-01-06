use errors::error::HttpError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrackingError {
    #[error("tracking number not found")]
    NotFound,

    #[error("courier not supported")]
    UnsupportedCourier,

    #[error("external service error: {0}")]
    ExternalService(String),

    #[error("invalid response format")]
    InvalidResponse,

    #[error("duplicate tracking number")]
    DuplicateTrackingNumber,
}

impl From<TrackingError> for HttpError {
    fn from(err: TrackingError) -> Self {
        match err {
            TrackingError::DuplicateTrackingNumber => HttpError::BadRequest(err.to_string()),
            TrackingError::NotFound => HttpError::NotFound(err.to_string()),
            TrackingError::UnsupportedCourier => HttpError::BadRequest(err.to_string()),
            _ => HttpError::InternalServerError(err.into()),
        }
    }
}