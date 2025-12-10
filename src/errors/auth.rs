use thiserror::Error;

/// Authentication and authorization related errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Missing token")]
    MissingToken,

    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    #[error("Account locked: {0}")]
    AccountLocked(String),

    #[error("Account disabled: {0}")]
    AccountDisabled(String),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Password hash error: {0}")]
    PasswordHashError(String),

    #[error("Password verification failed")]
    PasswordVerificationFailed,

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Maximum sessions exceeded")]
    MaxSessionsExceeded,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl AuthError {
    /// Create an insufficient permissions error with a custom message
    pub fn insufficient_permissions(permission: impl Into<String>) -> Self {
        Self::InsufficientPermissions(permission.into())
    }

    /// Create an unauthorized error with a custom message
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    /// Create an invalid token error with a custom message
    pub fn invalid_token(msg: impl Into<String>) -> Self {
        Self::InvalidToken(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_credentials() {
        let err = AuthError::InvalidCredentials;
        assert_eq!(err.to_string(), "Invalid credentials");
    }

    #[test]
    fn test_token_expired() {
        let err = AuthError::TokenExpired;
        assert_eq!(err.to_string(), "Token expired");
    }

    #[test]
    fn test_insufficient_permissions() {
        let err = AuthError::insufficient_permissions("user.delete");
        assert_eq!(err.to_string(), "Insufficient permissions: user.delete");
    }

    #[test]
    fn test_invalid_token() {
        let err = AuthError::invalid_token("Malformed token");
        assert_eq!(err.to_string(), "Invalid token: Malformed token");
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let err = AuthError::RateLimitExceeded("Too many login attempts".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: Too many login attempts");
    }
}
