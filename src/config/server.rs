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
        if self.port == 0 {
            return Err(ConfigError::ValidationError(
                "server.port must be > 0".to_string()
            ));
        }

        if self.tls.enabled {
            if self.tls.cert.is_empty() {
                return Err(ConfigError::MissingRequired("server.tls.cert".to_string()));
            }
            if self.tls.key.is_empty() {
                return Err(ConfigError::MissingRequired("server.tls.key".to_string()));
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
