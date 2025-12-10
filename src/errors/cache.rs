use thiserror::Error;

/// Cache-related errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache connection error: {0}")]
    ConnectionError(String),

    #[error("Cache serialization error: {0}")]
    SerializationError(String),

    #[error("Cache deserialization error: {0}")]
    DeserializationError(String),

    #[error("Cache operation timeout: {0}")]
    Timeout(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("Cache key not found: {0}")]
    KeyNotFound(String),

    #[error("Cache write error: {0}")]
    WriteError(String),

    #[error("Cache read error: {0}")]
    ReadError(String),

    #[error("Cache eviction error: {0}")]
    EvictionError(String),

    #[error("Cache pool error: {0}")]
    PoolError(String),
}

impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        use redis::ErrorKind;

        match err.kind() {
            ErrorKind::IoError => Self::ConnectionError(err.to_string()),
            ErrorKind::TypeError => Self::SerializationError(err.to_string()),
            ErrorKind::ResponseError if err.to_string().contains("timeout") => {
                Self::Timeout(err.to_string())
            }
            _ => Self::RedisError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error() {
        let err = CacheError::ConnectionError("Failed to connect to Redis".to_string());
        assert_eq!(err.to_string(), "Cache connection error: Failed to connect to Redis");
    }

    #[test]
    fn test_serialization_error() {
        let err = CacheError::SerializationError("Invalid JSON".to_string());
        assert_eq!(err.to_string(), "Cache serialization error: Invalid JSON");
    }

    #[test]
    fn test_key_not_found() {
        let err = CacheError::KeyNotFound("user:123".to_string());
        assert_eq!(err.to_string(), "Cache key not found: user:123");
    }
}
