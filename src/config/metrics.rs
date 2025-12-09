use serde::{Deserialize, Serialize};
use super::{ConfigError, Validate, WithDefaults};

/// Metrics configuration for observability
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MetricsConfig {
    /// Enable metrics collection
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Metrics export format
    #[serde(default = "default_export_format")]
    pub export_format: ExportFormat,

    /// HTTP endpoint for metrics export
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// Histogram bucket boundaries (milliseconds)
    #[serde(default = "default_histogram_buckets")]
    pub histogram_buckets: Vec<f64>,

    /// Enable HTTP request metrics
    #[serde(default = "default_true")]
    pub request_metrics: bool,

    /// Enable database query metrics
    #[serde(default = "default_true")]
    pub database_metrics: bool,

    /// Enable cache operation metrics
    #[serde(default = "default_true")]
    pub cache_metrics: bool,

    /// Enable system metrics (CPU, memory, etc.)
    #[serde(default = "default_true")]
    pub system_metrics: bool,

    /// Prometheus-specific configuration
    #[serde(default)]
    pub prometheus: PrometheusConfig,
}

/// Metrics export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Prometheus text format
    Prometheus,

    /// JSON format
    Json,

    /// Custom format (implementation-defined)
    Custom,
}

/// Prometheus-specific metrics configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PrometheusConfig {
    /// Enable Prometheus metrics
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Metric name prefix
    #[serde(default = "default_prefix")]
    pub prefix: String,

    /// Include default process metrics
    #[serde(default = "default_true")]
    pub process_metrics: bool,

    /// Include default runtime metrics
    #[serde(default = "default_true")]
    pub runtime_metrics: bool,

    /// Metric labels to include in all metrics
    #[serde(default)]
    pub labels: Vec<String>,
}

impl Validate for MetricsConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate endpoint
        if self.enabled && self.endpoint.is_empty() {
            return Err(ConfigError::MissingRequired("metrics.endpoint".to_string()));
        }

        if self.enabled && !self.endpoint.starts_with('/') {
            return Err(ConfigError::ValidationError(
                "metrics.endpoint must start with '/'".to_string()
            ));
        }

        // Validate histogram buckets
        if self.enabled && self.histogram_buckets.is_empty() {
            return Err(ConfigError::ValidationError(
                "metrics.histogram_buckets must not be empty".to_string()
            ));
        }

        // Validate histogram buckets are in ascending order
        if self.enabled {
            for i in 1..self.histogram_buckets.len() {
                if self.histogram_buckets[i] <= self.histogram_buckets[i - 1] {
                    return Err(ConfigError::ValidationError(
                        "metrics.histogram_buckets must be in ascending order".to_string()
                    ));
                }
            }
        }

        Ok(())
    }
}

impl WithDefaults for MetricsConfig {
    fn with_defaults() -> Self {
        Self {
            enabled: true,
            export_format: default_export_format(),
            endpoint: default_endpoint(),
            histogram_buckets: default_histogram_buckets(),
            request_metrics: true,
            database_metrics: true,
            cache_metrics: true,
            system_metrics: true,
            prometheus: PrometheusConfig::default(),
        }
    }
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: default_prefix(),
            process_metrics: true,
            runtime_metrics: true,
            labels: Vec::new(),
        }
    }
}

// Default value functions
fn default_export_format() -> ExportFormat { ExportFormat::Prometheus }
fn default_endpoint() -> String { "/metrics".to_string() }
fn default_prefix() -> String { "lighter".to_string() }
fn default_histogram_buckets() -> Vec<f64> {
    vec![5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0]
}
fn default_true() -> bool { true }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_defaults() {
        let config = MetricsConfig::with_defaults();
        assert!(config.enabled);
        assert_eq!(config.export_format, ExportFormat::Prometheus);
        assert_eq!(config.endpoint, "/metrics");
        assert!(!config.histogram_buckets.is_empty());
        assert!(config.request_metrics);
        assert!(config.database_metrics);
        assert!(config.cache_metrics);
        assert!(config.system_metrics);
    }

    #[test]
    fn test_metrics_config_validation_empty_endpoint() {
        let config = MetricsConfig {
            enabled: true,
            endpoint: "".to_string(),
            ..MetricsConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_metrics_config_validation_endpoint_no_slash() {
        let config = MetricsConfig {
            enabled: true,
            endpoint: "metrics".to_string(),
            ..MetricsConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_metrics_config_validation_empty_histogram_buckets() {
        let config = MetricsConfig {
            enabled: true,
            histogram_buckets: vec![],
            ..MetricsConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_metrics_config_validation_histogram_buckets_not_ascending() {
        let config = MetricsConfig {
            enabled: true,
            histogram_buckets: vec![10.0, 5.0, 25.0],
            ..MetricsConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_metrics_config_validation_histogram_buckets_equal_values() {
        let config = MetricsConfig {
            enabled: true,
            histogram_buckets: vec![5.0, 10.0, 10.0, 25.0],
            ..MetricsConfig::with_defaults()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_metrics_config_validation_histogram_buckets_valid() {
        let config = MetricsConfig {
            enabled: true,
            histogram_buckets: vec![5.0, 10.0, 25.0, 50.0, 100.0],
            ..MetricsConfig::with_defaults()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_metrics_config_validation_disabled_with_valid_values() {
        let config = MetricsConfig {
            enabled: false,
            ..MetricsConfig::with_defaults()
        };
        // When disabled with valid values, validation should pass
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_metrics_config_validation_disabled_with_invalid_values() {
        let config = MetricsConfig {
            enabled: false,
            endpoint: "".to_string(),
            histogram_buckets: vec![],
            ..MetricsConfig::with_defaults()
        };
        // When disabled, validation should pass even with invalid values
        // because the checks only run when enabled is true
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_metrics_config_validation_valid() {
        let config = MetricsConfig::with_defaults();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_prometheus_config_defaults() {
        let config = PrometheusConfig::default();
        assert!(config.enabled);
        assert_eq!(config.prefix, "lighter");
        assert!(config.process_metrics);
        assert!(config.runtime_metrics);
        assert!(config.labels.is_empty());
    }

    #[test]
    fn test_export_format_variants() {
        assert_eq!(ExportFormat::Prometheus, ExportFormat::Prometheus);
        assert_ne!(ExportFormat::Prometheus, ExportFormat::Json);
        assert_ne!(ExportFormat::Json, ExportFormat::Custom);
    }
}
