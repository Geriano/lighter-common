pub mod database;
pub mod cache;
pub mod auth;
pub mod validation;
pub mod http;

pub use database::DatabaseError;
pub use cache::CacheError;
pub use auth::AuthError;
pub use validation::ValidationError;
pub use http::HttpError;

use thiserror::Error;

/// Top-level error type that can be converted to HTTP responses
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AppError {
    /// Create a new configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new internal server error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
