use std::collections::HashMap;
use thiserror::Error;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;

use super::{AppError, AuthError, CacheError, DatabaseError, ValidationError};

/// HTTP error enum for API responses
#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Bad request: {message}")]
    BadRequest {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Unauthorized: {message}")]
    Unauthorized {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Forbidden: {message}")]
    Forbidden {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Not found: {message}")]
    NotFound {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Unprocessable entity")]
    UnprocessableEntity {
        errors: HashMap<String, Vec<String>>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Internal server error: {message}")]
    InternalServerError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Service unavailable: {message}")]
    ServiceUnavailable {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Too many requests: {message}")]
    TooManyRequests {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl HttpError {
    /// Create a BadRequest error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
            source: None,
        }
    }

    /// Create an Unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
            source: None,
        }
    }

    /// Create a Forbidden error
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
            message: message.into(),
            source: None,
        }
    }

    /// Create a NotFound error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
            source: None,
        }
    }

    /// Create an UnprocessableEntity error
    pub fn unprocessable_entity(errors: HashMap<String, Vec<String>>) -> Self {
        Self::UnprocessableEntity {
            errors,
            source: None,
        }
    }

    /// Create an InternalServerError error
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::InternalServerError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a ServiceUnavailable error
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::ServiceUnavailable {
            message: message.into(),
            source: None,
        }
    }

    /// Create a TooManyRequests error
    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::TooManyRequests {
            message: message.into(),
            source: None,
        }
    }

    /// Get the HTTP status code for this error
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    /// Get the error code string for this error
    fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest { .. } => "bad_request",
            Self::Unauthorized { .. } => "unauthorized",
            Self::Forbidden { .. } => "forbidden",
            Self::NotFound { .. } => "not_found",
            Self::UnprocessableEntity { .. } => "unprocessable_entity",
            Self::InternalServerError { .. } => "internal_server_error",
            Self::ServiceUnavailable { .. } => "service_unavailable",
            Self::TooManyRequests { .. } => "too_many_requests",
        }
    }
}

impl ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        self.status_code()
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            Self::UnprocessableEntity { errors, .. } => {
                HttpResponse::build(status).json(json!({
                    "errors": errors
                }))
            }
            Self::BadRequest { message, .. }
            | Self::Unauthorized { message, .. }
            | Self::Forbidden { message, .. }
            | Self::NotFound { message, .. }
            | Self::InternalServerError { message, .. }
            | Self::ServiceUnavailable { message, .. }
            | Self::TooManyRequests { message, .. } => {
                HttpResponse::build(status).json(json!({
                    "error": self.error_code(),
                    "message": message
                }))
            }
        }
    }
}

// From conversions
impl From<DatabaseError> for HttpError {
    fn from(err: DatabaseError) -> Self {
        match &err {
            DatabaseError::NotFound(msg) => Self::NotFound {
                message: msg.clone(),
                source: Some(Box::new(err)),
            },
            DatabaseError::UniqueViolation(msg) => {
                let mut errors = HashMap::new();
                errors.insert("constraint".to_string(), vec![msg.clone()]);
                Self::UnprocessableEntity {
                    errors,
                    source: Some(Box::new(err)),
                }
            }
            DatabaseError::ForeignKeyViolation(msg) => {
                let mut errors = HashMap::new();
                errors.insert("foreign_key".to_string(), vec![msg.clone()]);
                Self::UnprocessableEntity {
                    errors,
                    source: Some(Box::new(err)),
                }
            }
            DatabaseError::ConnectionError(msg)
            | DatabaseError::QueryError(msg)
            | DatabaseError::TransactionError(msg)
            | DatabaseError::MigrationError(msg)
            | DatabaseError::PoolError(msg)
            | DatabaseError::Other(msg) => {
                tracing::error!("Database error: {}", msg);
                Self::InternalServerError {
                    message: "An internal error occurred".to_string(),
                    source: Some(Box::new(err)),
                }
            }
        }
    }
}

impl From<AuthError> for HttpError {
    fn from(err: AuthError) -> Self {
        match &err {
            AuthError::InvalidCredentials
            | AuthError::TokenExpired
            | AuthError::InvalidToken(_)
            | AuthError::MissingToken
            | AuthError::SessionNotFound
            | AuthError::SessionExpired
            | AuthError::PasswordVerificationFailed
            | AuthError::Unauthorized(_) => Self::Unauthorized {
                message: err.to_string(),
                source: Some(Box::new(err)),
            },
            AuthError::InsufficientPermissions(msg)
            | AuthError::AccountLocked(msg)
            | AuthError::AccountDisabled(msg) => Self::Forbidden {
                message: msg.clone(),
                source: Some(Box::new(err)),
            },
            AuthError::MaxSessionsExceeded => Self::Forbidden {
                message: err.to_string(),
                source: Some(Box::new(err)),
            },
            AuthError::RateLimitExceeded(msg) => Self::TooManyRequests {
                message: msg.clone(),
                source: Some(Box::new(err)),
            },
            AuthError::PasswordHashError(msg) => {
                tracing::error!("Password hash error: {}", msg);
                Self::InternalServerError {
                    message: "An internal error occurred".to_string(),
                    source: Some(Box::new(err)),
                }
            }
        }
    }
}

impl From<CacheError> for HttpError {
    fn from(err: CacheError) -> Self {
        tracing::error!("Cache error: {}", err);
        Self::InternalServerError {
            message: "An internal error occurred".to_string(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<ValidationError> for HttpError {
    fn from(err: ValidationError) -> Self {
        Self::UnprocessableEntity {
            errors: err.errors.clone(),
            source: Some(Box::new(err)),
        }
    }
}

impl From<AppError> for HttpError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Database(e) => e.into(),
            AppError::Cache(e) => e.into(),
            AppError::Auth(e) => e.into(),
            AppError::Validation(e) => e.into(),
            AppError::Config(msg) => {
                tracing::error!("Configuration error: {}", msg);
                Self::InternalServerError {
                    message: "Service configuration error".to_string(),
                    source: None,
                }
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                Self::InternalServerError {
                    message: "An internal error occurred".to_string(),
                    source: None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_request() {
        let err = HttpError::bad_request("Invalid input");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.error_code(), "bad_request");
        assert!(err.to_string().contains("Invalid input"));
    }

    #[test]
    fn test_unauthorized() {
        let err = HttpError::unauthorized("Invalid token");
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(err.error_code(), "unauthorized");
        assert!(err.to_string().contains("Invalid token"));
    }

    #[test]
    fn test_forbidden() {
        let err = HttpError::forbidden("Access denied");
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(err.error_code(), "forbidden");
        assert!(err.to_string().contains("Access denied"));
    }

    #[test]
    fn test_not_found() {
        let err = HttpError::not_found("Resource not found");
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(err.error_code(), "not_found");
        assert!(err.to_string().contains("Resource not found"));
    }

    #[test]
    fn test_unprocessable_entity() {
        let mut errors = HashMap::new();
        errors.insert("email".to_string(), vec!["Invalid email".to_string()]);
        errors.insert("password".to_string(), vec!["Too short".to_string()]);

        let err = HttpError::unprocessable_entity(errors.clone());
        assert_eq!(err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(err.error_code(), "unprocessable_entity");

        if let HttpError::UnprocessableEntity { errors: e, .. } = err {
            assert_eq!(e.len(), 2);
            assert_eq!(e.get("email").unwrap(), &vec!["Invalid email".to_string()]);
        } else {
            panic!("Expected UnprocessableEntity variant");
        }
    }

    #[test]
    fn test_internal_server_error() {
        let err = HttpError::internal_server_error("Database connection failed");
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), "internal_server_error");
        assert!(err.to_string().contains("Database connection failed"));
    }

    #[test]
    fn test_service_unavailable() {
        let err = HttpError::service_unavailable("Service is down");
        assert_eq!(err.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(err.error_code(), "service_unavailable");
        assert!(err.to_string().contains("Service is down"));
    }

    #[test]
    fn test_too_many_requests() {
        let err = HttpError::too_many_requests("Rate limit exceeded");
        assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.error_code(), "too_many_requests");
        assert!(err.to_string().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_from_database_not_found() {
        let db_err = DatabaseError::NotFound("User not found".to_string());
        let http_err = HttpError::from(db_err);

        assert_eq!(http_err.status_code(), StatusCode::NOT_FOUND);
        assert!(http_err.to_string().contains("User not found"));
    }

    #[test]
    fn test_from_database_unique_violation() {
        let db_err = DatabaseError::UniqueViolation("Email already exists".to_string());
        let http_err = HttpError::from(db_err);

        assert_eq!(http_err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        if let HttpError::UnprocessableEntity { errors, .. } = http_err {
            assert!(errors.contains_key("constraint"));
        } else {
            panic!("Expected UnprocessableEntity variant");
        }
    }

    #[test]
    fn test_from_database_connection_error() {
        let db_err = DatabaseError::ConnectionError("Connection failed".to_string());
        let http_err = HttpError::from(db_err);

        assert_eq!(http_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        if let HttpError::InternalServerError { message, .. } = http_err {
            assert_eq!(message, "An internal error occurred");
        } else {
            panic!("Expected InternalServerError variant");
        }
    }

    #[test]
    fn test_from_auth_invalid_credentials() {
        let auth_err = AuthError::InvalidCredentials;
        let http_err = HttpError::from(auth_err);

        assert_eq!(http_err.status_code(), StatusCode::UNAUTHORIZED);
        assert!(http_err.to_string().contains("Invalid credentials"));
    }

    #[test]
    fn test_from_auth_insufficient_permissions() {
        let auth_err = AuthError::InsufficientPermissions("user.delete".to_string());
        let http_err = HttpError::from(auth_err);

        assert_eq!(http_err.status_code(), StatusCode::FORBIDDEN);
        assert!(http_err.to_string().contains("user.delete"));
    }

    #[test]
    fn test_from_auth_rate_limit_exceeded() {
        let auth_err = AuthError::RateLimitExceeded("Too many attempts".to_string());
        let http_err = HttpError::from(auth_err);

        assert_eq!(http_err.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert!(http_err.to_string().contains("Too many attempts"));
    }

    #[test]
    fn test_from_cache_error() {
        let cache_err = super::CacheError::ConnectionError("Redis down".to_string());
        let http_err = HttpError::from(cache_err);

        assert_eq!(http_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        if let HttpError::InternalServerError { message, .. } = http_err {
            assert_eq!(message, "An internal error occurred");
        } else {
            panic!("Expected InternalServerError variant");
        }
    }

    #[test]
    fn test_from_validation_error() {
        let mut validation = ValidationError::new();
        validation.add("email", "Invalid email format");
        validation.add("password", "Password is too short");

        let http_err = HttpError::from(validation);

        assert_eq!(http_err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        if let HttpError::UnprocessableEntity { errors, .. } = http_err {
            assert_eq!(errors.len(), 2);
            assert!(errors.contains_key("email"));
            assert!(errors.contains_key("password"));
        } else {
            panic!("Expected UnprocessableEntity variant");
        }
    }

    #[test]
    fn test_from_app_error_database() {
        let app_err = AppError::Database(DatabaseError::NotFound("Not found".to_string()));
        let http_err = HttpError::from(app_err);

        assert_eq!(http_err.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_from_app_error_auth() {
        let app_err = AppError::Auth(AuthError::InvalidCredentials);
        let http_err = HttpError::from(app_err);

        assert_eq!(http_err.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_from_app_error_config() {
        let app_err = AppError::Config("Invalid config".to_string());
        let http_err = HttpError::from(app_err);

        assert_eq!(http_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        if let HttpError::InternalServerError { message, .. } = http_err {
            assert_eq!(message, "Service configuration error");
        } else {
            panic!("Expected InternalServerError variant");
        }
    }

    #[test]
    fn test_error_response_format() {
        let err = HttpError::bad_request("Invalid input");
        let response = err.error_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_unprocessable_entity_response_format() {
        let mut errors = HashMap::new();
        errors.insert("email".to_string(), vec!["Invalid".to_string()]);

        let err = HttpError::unprocessable_entity(errors);
        let response = err.error_response();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
