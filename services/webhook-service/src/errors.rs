use errors::error::HttpError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShipmentServiceError {
    #[error("Shipment with ID {0} not found")]
    NotFound(String),

    #[error("Shipment cannot be cancelled in status: {0}")]
    InvalidStatusChange(String),

    #[error("Status mapping not found for platform: {0}")]
    StatusMappingNotFound(String),

    #[error("Event not supported: {0}")]
    UnsupportedEvent(String),

    #[error("An internal system error occurred: {0}")]
    Unexpected(#[from] anyhow::Error),
}

#[allow(clippy::from_over_into)]
impl Into<HttpError> for ShipmentServiceError {
    fn into(self) -> HttpError {
        match self {
            ShipmentServiceError::NotFound(msg) => HttpError::NotFound(msg),
            ShipmentServiceError::InvalidStatusChange(msg) => HttpError::BadRequest(msg),
            ShipmentServiceError::StatusMappingNotFound(msg) => HttpError::BadRequest(msg),
            ShipmentServiceError::UnsupportedEvent(msg) => HttpError::BadRequest(msg),
            ShipmentServiceError::Unexpected(e) => {
                HttpError::InternalServerError(anyhow::anyhow!("Internal error: {}", e))
            }
        }
    }
}
