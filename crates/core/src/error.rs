use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Resource not found")]
    NotFound,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Upstream error: {0}")]
    Upstream(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type CoreResult<T> = Result<T, CoreError>;
