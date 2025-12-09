use std::env;
use std::io::Error;
use std::net::SocketAddr;
use std::usize::MAX;

use actix_cors::Cors;
use actix_web::dev;
// use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::web::{Data, FormConfig, JsonConfig, PathConfig, PayloadConfig, ServiceConfig};
use actix_web::{App, HttpServer};
use rustls::ServerConfig;
use sea_orm::DatabaseConnection;

use crate::database;

#[derive(Clone)]
pub struct Server {
    port: u16,
    database: DatabaseConnection,
    tls: Option<ServerConfig>,
}

impl Server {
    pub fn new(port: u16, database: DatabaseConnection) -> Self {
        Self {
            port,
            database,
            tls: None,
        }
    }

    pub async fn env() -> Self {
        let port = env::var("PORT")
            .unwrap_or("3000".to_string())
            .parse::<u16>()
            .expect("PORT environment variable must be a number");

        let database = database::env().await;

        Self {
            port,
            database: database.unwrap(),
            tls: None,
        }
    }

    pub fn port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn database(&mut self, database: DatabaseConnection) {
        self.database = database;
    }

    pub fn tls(&mut self, tls: ServerConfig) {
        self.tls = Some(tls);
    }

    pub fn run<F>(self, callback: F) -> Result<dev::Server, Error>
    where
        F: FnOnce(&mut ServiceConfig) -> () + Clone + Copy + Send + 'static,
    {
        if self.tls.is_some() {
            return self.run_tls(callback);
        }

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let database = self.database.clone();
        let factory = move || {
            let payload = PayloadConfig::new(MAX);
            let path = PathConfig::default();
            let json = JsonConfig::default().limit(MAX);
            let form = FormConfig::default().limit(MAX);

            App::new()
                // .wrap(NormalizePath::new(TrailingSlash::Trim))
                .wrap(Server::cors())
                .app_data(payload)
                .app_data(path)
                .app_data(json)
                .app_data(form)
                .app_data(Data::new(database.clone()))
                .configure(callback)
        };

        let server = HttpServer::new(factory).workers(4).bind(addr)?.run();

        Ok(server)
    }

    fn run_tls<F>(self, callback: F) -> Result<dev::Server, Error>
    where
        F: FnOnce(&mut ServiceConfig) -> () + Clone + Copy + Send + 'static,
    {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let database = self.database.clone();
        let tls = self.tls.unwrap();
        let factory = move || {
            let payload = PayloadConfig::new(MAX);
            let path = PathConfig::default();
            let json = JsonConfig::default().limit(MAX);
            let form = FormConfig::default().limit(MAX);

            App::new()
                // .wrap(NormalizePath::new(TrailingSlash::Trim))
                .wrap(Server::cors())
                .app_data(payload)
                .app_data(path)
                .app_data(json)
                .app_data(form)
                .app_data(Data::new(database.clone()))
                .configure(callback)
        };

        let f = factory.clone();

        actix::spawn(async move {
            HttpServer::new(f)
                .workers(4)
                .bind(SocketAddr::from(([0, 0, 0, 0], 80)))?
                .run()
                .await
        });

        let server = HttpServer::new(factory)
            .workers(4)
            .bind_rustls_0_23(addr, tls)?
            .run();

        Ok(server)
    }

    pub fn cors() -> Cors {
        Cors::permissive()
    }
}
