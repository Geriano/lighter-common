use serde::{Deserialize, Serialize};
use super::{ConfigError, Validate, WithDefaults};

/// Health check configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct HealthConfig {
    /// Health check endpoint path
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// Readiness check endpoint path
    #[serde(default = "default_readiness_endpoint")]
    pub readiness_endpoint: String,

    /// Enable liveness checks
    #[serde(default = "default_true")]
    pub liveness_enabled: bool,

    /// Enable readiness checks
    #[serde(default = "default_true")]
    pub readiness_enabled: bool,

    /// Database health check timeout (seconds)
    #[serde(default = "default_db_check_timeout")]
    pub db_check_timeout: u64,

    /// Cache health check timeout (seconds)
    #[serde(default = "default_cache_check_timeout")]
    pub cache_check_timeout: u64,
}

impl Validate for HealthConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate health endpoint
        if self.liveness_enabled && self.endpoint.is_empty() {
            return Err(ConfigError::MissingRequired("health.endpoint".to_string()));
        }

        if self.liveness_enabled && !self.endpoint.starts_with('/') {
            return Err(ConfigError::ValidationError(
                "health.endpoint must start with '/'".to_string()
            ));
        }

        // Validate readiness endpoint
        if self.readiness_enabled && self.readiness_endpoint.is_empty() {
            return Err(ConfigError::MissingRequired("health.readiness_endpoint".to_string()));
        }

        if self.readiness_enabled && !self.readiness_endpoint.starts_with('/') {
            return Err(ConfigError::ValidationError(
                "health.readiness_endpoint must start with '/'".to_string()
            ));
        }

        // Validate timeouts
        if self.db_check_timeout == 0 {
            return Err(ConfigError::ValidationError(
                "health.db_check_timeout must be > 0".to_string()
            ));
        }

        if self.cache_check_timeout == 0 {
            return Err(ConfigError::ValidationError(
                "health.cache_check_timeout must be > 0".to_string()
            ));
        }

        Ok(())
    }
}

impl WithDefaults for HealthConfig {
    fn with_defaults() -> Self {
        Self {
            endpoint: default_endpoint(),
            readiness_endpoint: default_readiness_endpoint(),
            liveness_enabled: true,
            readiness_enabled: true,
            db_check_timeout: default_db_check_timeout(),
            cache_check_timeout: default_cache_check_timeout(),
        }
    }
}

// Default value functions
fn default_endpoint() -> String { "/health".to_string() }
fn default_readiness_endpoint() -> String { "/ready".to_string() }
fn default_db_check_timeout() -> u64 { 5 }
fn default_cache_check_timeout() -> u64 { 2 }
fn default_true() -> bool { true }
