use thiserror::Error;

#[derive(Error, Debug)]
pub enum WahaError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Client Error: {0}")]
    ClientError(String),

    #[error("Internal Server Waha Error")]
    InternalServerError(#[from] anyhow::Error),
}
