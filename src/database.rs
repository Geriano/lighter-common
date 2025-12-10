use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use crate::config::DatabaseConfig;

/// Create a database connection from configuration
pub async fn from_config(config: &DatabaseConfig) -> Result<DatabaseConnection, DbErr> {
    let mut options = ConnectOptions::new(&config.url);

    options
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(Duration::from_secs(config.connect_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .max_lifetime(Duration::from_secs(config.max_lifetime))
        .sqlx_logging(config.log_queries);

    Database::connect(options).await
}

/// Create an in-memory SQLite database for testing
pub async fn memory() -> Result<DatabaseConnection, DbErr> {
    let options = ConnectOptions::new("sqlite::memory:");
    Database::connect(options).await
}
