use serde::{Deserialize, Serialize};
use super::{ConfigError, Validate, WithDefaults};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,

    /// Maximum number of connections in pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of connections in pool
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection acquisition timeout (seconds)
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,

    /// Idle connection timeout (seconds)
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    /// Maximum connection lifetime (seconds)
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u64,

    /// Enable SQL query logging
    #[serde(default)]
    pub log_queries: bool,

    /// Slow query threshold for logging (milliseconds)
    #[serde(default = "default_slow_query_threshold")]
    pub slow_query_threshold: u64,

    /// Migration settings
    #[serde(default)]
    pub migration: MigrationConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MigrationConfig {
    /// Automatically run migrations on startup
    #[serde(default)]
    pub auto_migrate: bool,

    /// Fail startup if migration fails
    #[serde(default = "default_true")]
    pub strict: bool,
}

impl Validate for DatabaseConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.url.is_empty() {
            return Err(ConfigError::MissingRequired("database.url".to_string()));
        }

        if self.max_connections < self.min_connections {
            return Err(ConfigError::ValidationError(
                "database.max_connections must be >= min_connections".to_string()
            ));
        }

        if self.connect_timeout == 0 {
            return Err(ConfigError::ValidationError(
                "database.connect_timeout must be > 0".to_string()
            ));
        }

        Ok(())
    }
}

impl WithDefaults for DatabaseConfig {
    fn with_defaults() -> Self {
        Self {
            url: "postgres://postgres:postgres@localhost:5432/lighter_auth".to_string(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout: default_connect_timeout(),
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
            log_queries: false,
            slow_query_threshold: default_slow_query_threshold(),
            migration: MigrationConfig::default(),
        }
    }
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            auto_migrate: false,
            strict: true,
        }
    }
}

fn default_max_connections() -> u32 { 100 }
fn default_min_connections() -> u32 { 5 }
fn default_connect_timeout() -> u64 { 30 }
fn default_idle_timeout() -> u64 { 600 }
fn default_max_lifetime() -> u64 { 3600 }
fn default_slow_query_threshold() -> u64 { 1000 }
fn default_true() -> bool { true }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig::with_defaults();
        assert!(!config.url.is_empty());
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.connect_timeout, 30);
        assert_eq!(config.idle_timeout, 600);
        assert_eq!(config.max_lifetime, 3600);
        assert!(!config.log_queries);
        assert_eq!(config.slow_query_threshold, 1000);
    }

    #[test]
    fn test_database_config_validation_empty_url() {
        let config = DatabaseConfig {
            url: "".to_string(),
            ..DatabaseConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_database_config_validation_max_less_than_min() {
        let config = DatabaseConfig {
            max_connections: 5,
            min_connections: 10,
            ..DatabaseConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_database_config_validation_zero_connect_timeout() {
        let config = DatabaseConfig {
            connect_timeout: 0,
            ..DatabaseConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_database_config_validation_valid() {
        let config = DatabaseConfig {
            url: "postgres://user:pass@localhost/db".to_string(),
            max_connections: 100,
            min_connections: 5,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            log_queries: false,
            slow_query_threshold: 1000,
            migration: MigrationConfig::default(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_config_validation_equal_min_max() {
        let config = DatabaseConfig {
            max_connections: 10,
            min_connections: 10,
            ..DatabaseConfig::with_defaults()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_migration_config_defaults() {
        let config = MigrationConfig::default();
        assert!(!config.auto_migrate);
        assert!(config.strict);
    }
}
