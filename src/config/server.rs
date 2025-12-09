use serde::{Deserialize, Serialize};
use super::{ConfigError, Validate, WithDefaults, CorsConfig};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ServerConfig {
    /// HTTP server host binding
    #[serde(default = "default_host")]
    pub host: String,

    /// HTTP server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Number of worker threads (0 = CPU count)
    #[serde(default)]
    pub workers: usize,

    /// Request payload size limit (bytes, 0 = unlimited)
    #[serde(default = "default_max_payload_size")]
    pub max_payload_size: usize,

    /// Connection keep-alive timeout (seconds)
    #[serde(default = "default_keep_alive")]
    pub keep_alive: u64,

    /// Client request timeout (seconds)
    #[serde(default = "default_client_timeout")]
    pub client_timeout: u64,

    /// Enable HTTP/2
    #[serde(default = "default_true")]
    pub http2: bool,

    /// TLS configuration
    #[serde(default)]
    pub tls: TlsConfig,

    /// CORS configuration
    #[serde(default)]
    pub cors: CorsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct TlsConfig {
    /// Enable TLS
    #[serde(default)]
    pub enabled: bool,

    /// TLS certificate file path
    #[serde(default)]
    pub cert: String,

    /// TLS private key file path
    #[serde(default)]
    pub key: String,

    /// TLS certificate password (if encrypted)
    #[serde(default)]
    pub password: Option<String>,
}

impl Validate for ServerConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate port range (1-65535)
        if self.port == 0 {
            return Err(ConfigError::ValidationError(
                "server.port must be > 0".to_string()
            ));
        }
        // Port 0 is already checked, u16 max is 65535 so we're good

        if self.tls.enabled {
            if self.tls.cert.is_empty() {
                return Err(ConfigError::MissingRequired("server.tls.cert".to_string()));
            }
            if self.tls.key.is_empty() {
                return Err(ConfigError::MissingRequired("server.tls.key".to_string()));
            }

            // Validate TLS cert/key files exist
            if !std::path::Path::new(&self.tls.cert).exists() {
                return Err(ConfigError::ValidationError(
                    format!("server.tls.cert file does not exist: {}", self.tls.cert)
                ));
            }
            if !std::path::Path::new(&self.tls.key).exists() {
                return Err(ConfigError::ValidationError(
                    format!("server.tls.key file does not exist: {}", self.tls.key)
                ));
            }
        }

        // Validate CORS configuration
        self.cors.validate()?;

        Ok(())
    }
}

impl WithDefaults for ServerConfig {
    fn with_defaults() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: 0,
            max_payload_size: default_max_payload_size(),
            keep_alive: default_keep_alive(),
            client_timeout: default_client_timeout(),
            http2: true,
            tls: TlsConfig::default(),
            cors: CorsConfig::default(),
        }
    }
}

fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8080 }
fn default_max_payload_size() -> usize { 10 * 1024 * 1024 } // 10 MB
fn default_keep_alive() -> u64 { 75 }
fn default_client_timeout() -> u64 { 60 }
fn default_true() -> bool { true }

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::with_defaults();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.workers, 0);
        assert_eq!(config.max_payload_size, 10 * 1024 * 1024);
        assert_eq!(config.keep_alive, 75);
        assert_eq!(config.client_timeout, 60);
        assert!(config.http2);
        assert!(!config.tls.enabled);
    }

    #[test]
    fn test_server_config_validation_zero_port() {
        let config = ServerConfig {
            port: 0,
            ..ServerConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_server_config_validation_valid_port_range() {
        // Test minimum valid port
        let config = ServerConfig {
            port: 1,
            ..ServerConfig::with_defaults()
        };
        assert!(config.validate().is_ok());

        // Test maximum valid port (u16 max is 65535)
        let config = ServerConfig {
            port: 65535,
            ..ServerConfig::with_defaults()
        };
        assert!(config.validate().is_ok());

        // Test common port
        let config = ServerConfig {
            port: 8080,
            ..ServerConfig::with_defaults()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_server_config_validation_tls_enabled_missing_cert() {
        let config = ServerConfig {
            tls: TlsConfig {
                enabled: true,
                cert: "".to_string(),
                key: "/path/to/key.pem".to_string(),
                password: None,
            },
            ..ServerConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_server_config_validation_tls_enabled_missing_key() {
        let config = ServerConfig {
            tls: TlsConfig {
                enabled: true,
                cert: "/path/to/cert.pem".to_string(),
                key: "".to_string(),
                password: None,
            },
            ..ServerConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_server_config_validation_tls_enabled_files_not_exist() {
        let config = ServerConfig {
            tls: TlsConfig {
                enabled: true,
                cert: "/nonexistent/cert.pem".to_string(),
                key: "/nonexistent/key.pem".to_string(),
                password: None,
            },
            ..ServerConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_server_config_validation_tls_enabled_files_exist() {
        // Create temporary files
        let temp_dir = std::env::temp_dir();
        let cert_path = temp_dir.join("test_cert.pem");
        let key_path = temp_dir.join("test_key.pem");

        {
            let mut cert_file = std::fs::File::create(&cert_path).unwrap();
            cert_file.write_all(b"fake cert").unwrap();

            let mut key_file = std::fs::File::create(&key_path).unwrap();
            key_file.write_all(b"fake key").unwrap();
        }

        let config = ServerConfig {
            tls: TlsConfig {
                enabled: true,
                cert: cert_path.to_str().unwrap().to_string(),
                key: key_path.to_str().unwrap().to_string(),
                password: None,
            },
            ..ServerConfig::with_defaults()
        };
        assert!(config.validate().is_ok());

        // Cleanup
        std::fs::remove_file(cert_path).ok();
        std::fs::remove_file(key_path).ok();
    }

    #[test]
    fn test_server_config_validation_tls_disabled() {
        let config = ServerConfig {
            tls: TlsConfig {
                enabled: false,
                cert: "".to_string(),
                key: "".to_string(),
                password: None,
            },
            ..ServerConfig::with_defaults()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_tls_config_defaults() {
        let config = TlsConfig::default();
        assert!(!config.enabled);
        assert!(config.cert.is_empty());
        assert!(config.key.is_empty());
        assert!(config.password.is_none());
    }
}
