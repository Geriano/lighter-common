use std::env;
// use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

pub async fn connect<S: AsRef<str>>(url: S) -> Result<DatabaseConnection, DbErr> {
    let option = ConnectOptions::new(url.as_ref())
        // .idle_timeout(Duration::from_millis(1000))
        // .connect_timeout(Duration::from_millis(1000))
        // .acquire_timeout(Duration::from_millis(1000))
        .to_owned();

    Database::connect(option).await
}

pub async fn env() -> Result<DatabaseConnection, DbErr> {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
    // let idle_timeout = env::var("DATABASE_IDLE_TIMEOUT")
    //     .unwrap_or_else(|_| "1000".to_string())
    //     .parse::<u64>()
    //     .expect("DATABASE_IDLE_TIMEOUT environment variable must be a number");
    // let connect_timeout = env::var("DATABASE_CONNECT_TIMEOUT")
    //     .unwrap_or_else(|_| "1000".to_string())
    //     .parse::<u64>()
    //     .expect("DATABASE_CONNECT_TIMEOUT environment variable must be a number");
    // let acquire_timeout = env::var("DATABASE_ACQUIRE_TIMEOUT")
    //     .unwrap_or_else(|_| "1000".to_string())
    //     .parse::<u64>()
    //     .expect("DATABASE_ACQUIRE_TIMEOUT environment variable must be a number");

    let option = ConnectOptions::new(url)
        // .idle_timeout(Duration::from_millis(idle_timeout))
        // .connect_timeout(Duration::from_millis(connect_timeout))
        // .acquire_timeout(Duration::from_millis(acquire_timeout))
        .to_owned();

    Database::connect(option).await
}

pub async fn memory() -> Result<DatabaseConnection, DbErr> {
    let option = ConnectOptions::new("sqlite::memory:");

    Database::connect(option).await
}
