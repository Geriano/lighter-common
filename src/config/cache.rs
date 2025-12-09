use serde::{Deserialize, Serialize};
use super::{ConfigError, Validate, WithDefaults};

/// Cache configuration for local and distributed caching
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CacheConfig {
    /// Cache type to use
    #[serde(default = "default_cache_type", rename = "type")]
    pub cache_type: CacheType,

    /// Default TTL for cache entries (seconds)
    #[serde(default = "default_ttl")]
    pub default_ttl: u64,

    /// Maximum number of items in cache
    #[serde(default = "default_max_size")]
    pub max_size: usize,

    /// Cache eviction policy
    #[serde(default = "default_eviction_policy")]
    pub eviction_policy: EvictionPolicy,

    /// Local cache configuration
    #[serde(default)]
    pub local: LocalCacheConfig,

    /// Redis cache configuration
    #[serde(default)]
    pub redis: RedisCacheConfig,
}

/// Cache type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    /// Local in-memory cache only
    Local,

    /// Redis cache only
    Redis,

    /// Hybrid: local cache with Redis fallback
    Hybrid,

    /// No caching
    None,
}

/// Cache eviction policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EvictionPolicy {
    /// Least Recently Used
    Lru,

    /// Least Frequently Used
    Lfu,

    /// Time To Live based eviction
    Ttl,
}

/// Local cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LocalCacheConfig {
    /// Enable local cache
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Number of cache shards (0 = CPU count * 4)
    #[serde(default)]
    pub shards: usize,
}

/// Redis cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RedisCacheConfig {
    /// Enable Redis cache
    #[serde(default)]
    pub enabled: bool,

    /// Redis connection URL
    #[serde(default)]
    pub url: String,

    /// Redis database number
    #[serde(default)]
    pub database: u8,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,

    /// Connection timeout (seconds)
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Enable TLS for Redis connection
    #[serde(default)]
    pub tls: bool,
}

impl Validate for CacheConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate cache type specific requirements
        match self.cache_type {
            CacheType::Redis | CacheType::Hybrid => {
                if self.redis.enabled && self.redis.url.is_empty() {
                    return Err(ConfigError::MissingRequired("cache.redis.url".to_string()));
                }
            }
            _ => {}
        }

        // Validate TTL
        if self.default_ttl == 0 {
            return Err(ConfigError::ValidationError(
                "cache.default_ttl must be > 0".to_string()
            ));
        }

        // Validate max size
        if self.max_size == 0 {
            return Err(ConfigError::ValidationError(
                "cache.max_size must be > 0".to_string()
            ));
        }

        // Validate Redis settings if enabled
        if self.redis.enabled {
            if self.redis.url.is_empty() {
                return Err(ConfigError::MissingRequired("cache.redis.url".to_string()));
            }

            if self.redis.pool_size == 0 {
                return Err(ConfigError::ValidationError(
                    "cache.redis.pool_size must be > 0".to_string()
                ));
            }

            if self.redis.timeout == 0 {
                return Err(ConfigError::ValidationError(
                    "cache.redis.timeout must be > 0".to_string()
                ));
            }
        }

        Ok(())
    }
}

impl WithDefaults for CacheConfig {
    fn with_defaults() -> Self {
        Self {
            cache_type: default_cache_type(),
            default_ttl: default_ttl(),
            max_size: default_max_size(),
            eviction_policy: default_eviction_policy(),
            local: LocalCacheConfig::default(),
            redis: RedisCacheConfig::default(),
        }
    }
}

impl Default for LocalCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            shards: 0,
        }
    }
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: String::new(),
            database: 0,
            pool_size: default_pool_size(),
            timeout: default_timeout(),
            tls: false,
        }
    }
}

// Default value functions
fn default_cache_type() -> CacheType { CacheType::Local }
fn default_ttl() -> u64 { 300 } // 5 minutes
fn default_max_size() -> usize { 10_000 }
fn default_eviction_policy() -> EvictionPolicy { EvictionPolicy::Ttl }
fn default_pool_size() -> usize { 20 }
fn default_timeout() -> u64 { 5 }
fn default_true() -> bool { true }
