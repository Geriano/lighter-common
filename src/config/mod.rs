use thiserror::Error;

pub mod database;
pub mod server;
pub mod cache;
pub mod metrics;
pub mod observability;
pub mod health;
pub mod cors;

pub use database::DatabaseConfig;
pub use server::ServerConfig;
pub use cache::CacheConfig;
pub use metrics::MetricsConfig;
pub use observability::ObservabilityConfig;
pub use health::HealthConfig;
pub use cors::CorsConfig;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Configuration parsing error: {0}")]
    ParseError(String),

    #[error("Configuration validation error: {0}")]
    ValidationError(String),

    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error(transparent)]
    ConfigCrateError(#[from] config::ConfigError),
}

/// Trait for configuration validation
pub trait Validate {
    fn validate(&self) -> Result<(), ConfigError>;
}

/// Trait for configuration with defaults
pub trait WithDefaults {
    fn with_defaults() -> Self;
}
