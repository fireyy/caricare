use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    Custom(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
