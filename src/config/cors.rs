use serde::{Deserialize, Serialize};
use super::{ConfigError, Validate, WithDefaults};

/// CORS (Cross-Origin Resource Sharing) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CorsConfig {
    /// Enable CORS
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Allowed origins (e.g., ["https://example.com", "*"])
    /// Use "*" for wildcard (allows any origin)
    #[serde(default = "default_origins")]
    pub origins: Vec<String>,

    /// Allowed HTTP methods
    #[serde(default = "default_methods")]
    pub methods: Vec<String>,

    /// Allowed HTTP headers
    #[serde(default = "default_headers")]
    pub headers: Vec<String>,

    /// Max age in seconds for preflight requests
    #[serde(default = "default_max_age")]
    pub max_age: u64,

    /// Allow credentials (cookies, authorization headers)
    /// Note: Cannot be true when origin is "*"
    #[serde(default = "default_allow_credentials")]
    pub allow_credentials: bool,

    /// Send wildcard header when origin is "*"
    #[serde(default = "default_send_wildcard")]
    pub send_wildcard: bool,
}

impl Validate for CorsConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.origins.is_empty() {
            return Err(ConfigError::ValidationError(
                "cors.origins cannot be empty when CORS is enabled".to_string()
            ));
        }

        if self.max_age == 0 {
            return Err(ConfigError::ValidationError(
                "cors.max_age must be > 0".to_string()
            ));
        }

        // Validate that allow_credentials is false when wildcard origin is used
        if self.allow_credentials && self.origins.contains(&"*".to_string()) {
            return Err(ConfigError::ValidationError(
                "cors.allow_credentials cannot be true when origin contains wildcard '*'".to_string()
            ));
        }

        // Validate methods are not empty
        if self.enabled && self.methods.is_empty() {
            return Err(ConfigError::ValidationError(
                "cors.methods cannot be empty when CORS is enabled".to_string()
            ));
        }

        // Validate headers are not empty
        if self.enabled && self.headers.is_empty() {
            return Err(ConfigError::ValidationError(
                "cors.headers cannot be empty when CORS is enabled".to_string()
            ));
        }

        Ok(())
    }
}

impl WithDefaults for CorsConfig {
    fn with_defaults() -> Self {
        Self {
            enabled: default_enabled(),
            origins: default_origins(),
            methods: default_methods(),
            headers: default_headers(),
            max_age: default_max_age(),
            allow_credentials: default_allow_credentials(),
            send_wildcard: default_send_wildcard(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self::with_defaults()
    }
}

fn default_enabled() -> bool {
    true
}

fn default_origins() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "PATCH".to_string(),
        "OPTIONS".to_string(),
    ]
}

fn default_headers() -> Vec<String> {
    vec![
        "Authorization".to_string(),
        "Content-Type".to_string(),
        "Accept".to_string(),
    ]
}

fn default_max_age() -> u64 {
    3600 // 1 hour
}

fn default_allow_credentials() -> bool {
    false
}

fn default_send_wildcard() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_config_defaults() {
        let config = CorsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.origins, vec!["*"]);
        assert_eq!(config.methods.len(), 6);
        assert!(config.methods.contains(&"GET".to_string()));
        assert!(config.methods.contains(&"POST".to_string()));
        assert_eq!(config.headers.len(), 3);
        assert!(config.headers.contains(&"Authorization".to_string()));
        assert!(config.headers.contains(&"Content-Type".to_string()));
        assert_eq!(config.max_age, 3600);
        assert!(!config.allow_credentials);
        assert!(config.send_wildcard);
    }

    #[test]
    fn test_cors_config_validation_empty_origins() {
        let config = CorsConfig {
            enabled: true,
            origins: vec![],
            ..CorsConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cors_config_validation_zero_max_age() {
        let config = CorsConfig {
            max_age: 0,
            ..CorsConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cors_config_validation_wildcard_with_credentials() {
        let config = CorsConfig {
            origins: vec!["*".to_string()],
            allow_credentials: true,
            ..CorsConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cors_config_validation_empty_methods() {
        let config = CorsConfig {
            enabled: true,
            methods: vec![],
            ..CorsConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cors_config_validation_empty_headers() {
        let config = CorsConfig {
            enabled: true,
            headers: vec![],
            ..CorsConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cors_config_validation_specific_origins_with_credentials() {
        let config = CorsConfig {
            origins: vec!["https://example.com".to_string()],
            allow_credentials: true,
            ..CorsConfig::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cors_config_disabled_with_empty_origins() {
        let config = CorsConfig {
            enabled: false,
            origins: vec![],
            methods: vec!["GET".to_string()],
            headers: vec!["Content-Type".to_string()],
            ..CorsConfig::default()
        };
        // Should be valid because CORS is disabled
        assert!(config.validate().is_ok());
    }
}
