use thiserror::Error;
use sea_orm::{DbErr, TransactionError};

/// Database-related errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("Foreign key constraint violation: {0}")]
    ForeignKeyViolation(String),

    #[error("Database query error: {0}")]
    QueryError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Connection pool error: {0}")]
    PoolError(String),

    #[error("Database error: {0}")]
    Other(String),
}

impl From<TransactionError<DbErr>> for DatabaseError {
    fn from(err: TransactionError<DbErr>) -> Self {
        match err {
            TransactionError::Connection(e) => Self::ConnectionError(e.to_string()),
            TransactionError::Transaction(e) => Self::TransactionError(e.to_string()),
        }
    }
}

impl From<DbErr> for DatabaseError {
    fn from(err: DbErr) -> Self {
        let err_str = err.to_string();

        match err {
            DbErr::RecordNotFound(msg) => Self::NotFound(msg),
            DbErr::Query(_) if err_str.contains("UNIQUE constraint") || err_str.contains("duplicate key") => {
                Self::UniqueViolation(err_str)
            }
            DbErr::Query(_) if err_str.contains("FOREIGN KEY constraint") || err_str.contains("foreign key") => {
                Self::ForeignKeyViolation(err_str)
            }
            DbErr::Conn(_) => Self::ConnectionError(err_str),
            DbErr::Exec(_) => Self::QueryError(err_str),
            DbErr::Query(_) => Self::QueryError(err_str),
            _ => Self::Other(err_str),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let err = DatabaseError::NotFound("User not found".to_string());
        assert_eq!(err.to_string(), "Record not found: User not found");
    }

    #[test]
    fn test_unique_violation() {
        let err = DatabaseError::UniqueViolation("Email already exists".to_string());
        assert_eq!(err.to_string(), "Unique constraint violation: Email already exists");
    }

    #[test]
    fn test_from_db_err() {
        let db_err = DbErr::RecordNotFound("test".to_string());
        let err = DatabaseError::from(db_err);
        assert!(matches!(err, DatabaseError::NotFound(_)));
    }
}
