use serde::{Deserialize, Serialize};
use super::{ConfigError, Validate, WithDefaults};

/// Observability configuration for logging and tracing
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ObservabilityConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Log output format
    #[serde(default = "default_log_format")]
    pub log_format: LogFormat,

    /// Enable distributed tracing
    #[serde(default = "default_true")]
    pub tracing_enabled: bool,

    /// Trace sampler type
    #[serde(default = "default_trace_sampler")]
    pub trace_sampler: TraceSampler,

    /// Trace sampling rate (0.0 - 1.0, only used for probabilistic sampler)
    #[serde(default = "default_sampling_rate")]
    pub trace_sampling_rate: f64,

    /// Tracing exporter type
    #[serde(default = "default_tracing_exporter")]
    pub tracing_exporter: TracingExporter,

    /// OTLP endpoint URL
    #[serde(default = "default_otlp_endpoint")]
    pub otlp_endpoint: String,

    /// Service name for tracing
    #[serde(default = "default_service_name")]
    pub service_name: String,

    /// Service version
    #[serde(default = "default_service_version")]
    pub service_version: String,

    /// Service instance ID (for distributed systems)
    #[serde(default = "default_service_instance_id")]
    pub service_instance_id: String,
}

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable pretty format
    Pretty,

    /// JSON structured logging
    Json,

    /// Compact format
    Compact,
}

/// Trace sampling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TraceSampler {
    /// Always sample all traces
    Always,

    /// Never sample any traces
    Never,

    /// Sample probabilistically based on rate
    Probabilistic,
}

/// Tracing exporter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TracingExporter {
    /// Jaeger exporter
    Jaeger,

    /// Zipkin exporter
    Zipkin,

    /// OpenTelemetry Protocol (OTLP)
    Otlp,

    /// No exporter (tracing disabled)
    None,
}

impl Validate for ObservabilityConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate service name is required
        if self.service_name.is_empty() {
            return Err(ConfigError::MissingRequired("observability.service_name".to_string()));
        }

        // Validate service version is required
        if self.service_version.is_empty() {
            return Err(ConfigError::MissingRequired("observability.service_version".to_string()));
        }

        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.log_level.to_lowercase().as_str()) {
            return Err(ConfigError::ValidationError(
                format!(
                    "observability.log_level must be one of: {}",
                    valid_levels.join(", ")
                )
            ));
        }

        // Validate sampling rate for probabilistic sampler
        if self.trace_sampler == TraceSampler::Probabilistic {
            if !(0.0..=1.0).contains(&self.trace_sampling_rate) {
                return Err(ConfigError::ValidationError(
                    "observability.trace_sampling_rate must be between 0.0 and 1.0".to_string()
                ));
            }
        }

        // Validate OTLP endpoint if using OTLP exporter
        if self.tracing_enabled && self.tracing_exporter == TracingExporter::Otlp {
            if self.otlp_endpoint.is_empty() {
                return Err(ConfigError::MissingRequired(
                    "observability.otlp_endpoint".to_string()
                ));
            }
        }

        Ok(())
    }
}

impl WithDefaults for ObservabilityConfig {
    fn with_defaults() -> Self {
        Self {
            log_level: default_log_level(),
            log_format: default_log_format(),
            tracing_enabled: true,
            trace_sampler: default_trace_sampler(),
            trace_sampling_rate: default_sampling_rate(),
            tracing_exporter: default_tracing_exporter(),
            otlp_endpoint: default_otlp_endpoint(),
            service_name: default_service_name(),
            service_version: default_service_version(),
            service_instance_id: default_service_instance_id(),
        }
    }
}

// Default value functions
fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> LogFormat { LogFormat::Pretty }
fn default_trace_sampler() -> TraceSampler { TraceSampler::Always }
fn default_sampling_rate() -> f64 { 1.0 }
fn default_tracing_exporter() -> TracingExporter { TracingExporter::Otlp }
fn default_otlp_endpoint() -> String { "http://localhost:4317".to_string() }
fn default_service_name() -> String { "lighter".to_string() }
fn default_service_version() -> String { "0.1.0".to_string() }
fn default_service_instance_id() -> String { "lighter-1".to_string() }
fn default_true() -> bool { true }
