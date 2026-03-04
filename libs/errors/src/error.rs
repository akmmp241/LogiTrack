use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("{0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error")]
    InternalServerError(#[from] anyhow::Error),
}
