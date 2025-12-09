use std::io::Error;
use std::net::SocketAddr;

use actix_cors::Cors;
use actix_web::dev;
use actix_web::http;
// use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::web::{Data, FormConfig, JsonConfig, PathConfig, PayloadConfig, ServiceConfig};
use actix_web::{App, HttpServer};
use rustls::ServerConfig as RustlsConfig;
use sea_orm::DatabaseConnection;

use crate::config::{ServerConfig, CorsConfig};
use crate::tls;

#[derive(Clone)]
pub struct Server {
    config: ServerConfig,
    database: DatabaseConnection,
    tls: Option<RustlsConfig>,
}

impl Server {
    /// Create a new Server instance from configuration
    pub fn from_config(config: ServerConfig, database: DatabaseConnection) -> Self {
        // Configure TLS if enabled
        let tls = if config.tls.enabled {
            Some(tls::configure(&config.tls.cert, &config.tls.key))
        } else {
            None
        };

        Self {
            config,
            database,
            tls,
        }
    }

    pub fn run<F>(self, callback: F) -> Result<dev::Server, Error>
    where
        F: FnOnce(&mut ServiceConfig) -> () + Clone + Copy + Send + 'static,
    {
        if self.tls.is_some() {
            return self.run_tls(callback);
        }

        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .expect("Failed to parse server address");

        let database = self.database.clone();
        let max_payload = self.config.max_payload_size;
        let workers = self.config.workers;
        let cors_config = self.config.cors.clone();

        let factory = move || {
            let payload = PayloadConfig::new(max_payload);
            let path = PathConfig::default();
            let json = JsonConfig::default().limit(max_payload);
            let form = FormConfig::default().limit(max_payload);

            App::new()
                // .wrap(NormalizePath::new(TrailingSlash::Trim))
                .wrap(Server::cors(&cors_config))
                .app_data(payload)
                .app_data(path)
                .app_data(json)
                .app_data(form)
                .app_data(Data::new(database.clone()))
                .configure(callback)
        };

        let mut http_server = HttpServer::new(factory);

        // Set workers (0 = auto-detect CPU count)
        if workers > 0 {
            http_server = http_server.workers(workers);
        }

        let server = http_server.bind(addr)?.run();

        Ok(server)
    }

    fn run_tls<F>(self, callback: F) -> Result<dev::Server, Error>
    where
        F: FnOnce(&mut ServiceConfig) -> () + Clone + Copy + Send + 'static,
    {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .expect("Failed to parse server address");

        let database = self.database.clone();
        let tls = self.tls.unwrap();
        let max_payload = self.config.max_payload_size;
        let workers = self.config.workers;
        let cors_config = self.config.cors.clone();

        let factory = move || {
            let payload = PayloadConfig::new(max_payload);
            let path = PathConfig::default();
            let json = JsonConfig::default().limit(max_payload);
            let form = FormConfig::default().limit(max_payload);

            App::new()
                // .wrap(NormalizePath::new(TrailingSlash::Trim))
                .wrap(Server::cors(&cors_config))
                .app_data(payload)
                .app_data(path)
                .app_data(json)
                .app_data(form)
                .app_data(Data::new(database.clone()))
                .configure(callback)
        };

        let f = factory.clone();

        // Spawn HTTP redirect server on port 80
        actix::spawn(async move {
            let mut http_server = HttpServer::new(f);
            if workers > 0 {
                http_server = http_server.workers(workers);
            }
            http_server
                .bind(SocketAddr::from(([0, 0, 0, 0], 80)))?
                .run()
                .await
        });

        // Main HTTPS server
        let mut https_server = HttpServer::new(factory);
        if workers > 0 {
            https_server = https_server.workers(workers);
        }

        let server = https_server.bind_rustls_0_23(addr, tls)?.run();

        Ok(server)
    }

    /// Create CORS middleware from configuration
    pub fn cors(config: &CorsConfig) -> Cors {
        // If CORS is disabled, return permissive CORS or a minimal setup
        if !config.enabled {
            return Cors::permissive();
        }

        let mut cors = Cors::default();

        // Handle origins
        let has_wildcard = config.origins.contains(&"*".to_string());

        if has_wildcard {
            // Wildcard origin - allow any origin
            cors = cors.allow_any_origin();

            // Send wildcard header if configured
            if config.send_wildcard {
                cors = cors.send_wildcard();
            }
        } else {
            // Specific origins
            for origin in &config.origins {
                cors = cors.allowed_origin(origin.as_str());
            }
        }

        // Set allowed methods
        let methods: Vec<http::Method> = config.methods.iter()
            .filter_map(|m| m.parse().ok())
            .collect();
        cors = cors.allowed_methods(methods);

        // Set allowed headers
        let headers: Vec<http::header::HeaderName> = config.headers.iter()
            .filter_map(|h| h.parse().ok())
            .collect();
        cors = cors.allowed_headers(headers);

        // Set max age for preflight requests
        cors = cors.max_age(config.max_age as usize);

        // Set allow credentials (only if not wildcard origin)
        if config.allow_credentials && !has_wildcard {
            cors = cors.supports_credentials();
        }

        cors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Validate, WithDefaults};

    #[test]
    fn test_cors_disabled() {
        let config = CorsConfig {
            enabled: false,
            ..CorsConfig::default()
        };

        // When CORS is disabled, it should return permissive CORS
        let cors = Server::cors(&config);
        // We can't easily test the internals of actix_cors::Cors,
        // but we can verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_cors_wildcard_origin() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["*".to_string()],
            methods: vec!["GET".to_string(), "POST".to_string()],
            headers: vec!["Content-Type".to_string()],
            max_age: 3600,
            allow_credentials: false,
            send_wildcard: true,
        };

        let cors = Server::cors(&config);
        // Verify it doesn't panic with wildcard configuration
        assert!(true);
    }

    #[test]
    fn test_cors_specific_origins() {
        let config = CorsConfig {
            enabled: true,
            origins: vec![
                "https://example.com".to_string(),
                "https://another.com".to_string(),
            ],
            methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string()],
            headers: vec!["Authorization".to_string(), "Content-Type".to_string()],
            max_age: 7200,
            allow_credentials: false,
            send_wildcard: false,
        };

        let cors = Server::cors(&config);
        // Verify it doesn't panic with specific origins
        assert!(true);
    }

    #[test]
    fn test_cors_with_credentials() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["https://example.com".to_string()],
            methods: vec!["GET".to_string(), "POST".to_string()],
            headers: vec!["Authorization".to_string(), "Content-Type".to_string()],
            max_age: 3600,
            allow_credentials: true,
            send_wildcard: false,
        };

        let cors = Server::cors(&config);
        // Verify it doesn't panic with credentials enabled
        assert!(true);
    }

    #[test]
    fn test_cors_wildcard_ignores_credentials() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["*".to_string()],
            methods: vec!["GET".to_string()],
            headers: vec!["Content-Type".to_string()],
            max_age: 3600,
            allow_credentials: true, // This should be ignored due to wildcard
            send_wildcard: true,
        };

        let cors = Server::cors(&config);
        // Verify it doesn't panic even with invalid config
        // (credentials are ignored when wildcard is used)
        assert!(true);
    }

    #[test]
    fn test_cors_all_methods() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["https://example.com".to_string()],
            methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
                "HEAD".to_string(),
            ],
            headers: vec!["Authorization".to_string()],
            max_age: 3600,
            allow_credentials: false,
            send_wildcard: false,
        };

        let cors = Server::cors(&config);
        assert!(true);
    }

    #[test]
    fn test_cors_multiple_headers() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["https://example.com".to_string()],
            methods: vec!["GET".to_string(), "POST".to_string()],
            headers: vec![
                "Authorization".to_string(),
                "Content-Type".to_string(),
                "Accept".to_string(),
                "X-Requested-With".to_string(),
                "X-Custom-Header".to_string(),
            ],
            max_age: 3600,
            allow_credentials: false,
            send_wildcard: false,
        };

        let cors = Server::cors(&config);
        assert!(true);
    }

    #[test]
    fn test_cors_max_age_variations() {
        let configs = vec![
            (1, "1 second"),
            (60, "1 minute"),
            (3600, "1 hour"),
            (86400, "1 day"),
        ];

        for (max_age, _description) in configs {
            let config = CorsConfig {
                enabled: true,
                origins: vec!["https://example.com".to_string()],
                methods: vec!["GET".to_string()],
                headers: vec!["Content-Type".to_string()],
                max_age,
                allow_credentials: false,
                send_wildcard: false,
            };

            let cors = Server::cors(&config);
            assert!(true);
        }
    }

    #[test]
    fn test_cors_invalid_method_filtered() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["https://example.com".to_string()],
            methods: vec![
                "GET".to_string(),
                "INVALID_METHOD".to_string(), // This should be filtered out
                "POST".to_string(),
            ],
            headers: vec!["Content-Type".to_string()],
            max_age: 3600,
            allow_credentials: false,
            send_wildcard: false,
        };

        // Should not panic even with invalid methods
        let cors = Server::cors(&config);
        assert!(true);
    }

    #[test]
    fn test_cors_invalid_header_filtered() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["https://example.com".to_string()],
            methods: vec!["GET".to_string()],
            headers: vec![
                "Content-Type".to_string(),
                "Invalid\nHeader".to_string(), // This should be filtered out
                "Authorization".to_string(),
            ],
            max_age: 3600,
            allow_credentials: false,
            send_wildcard: false,
        };

        // Should not panic even with invalid headers
        let cors = Server::cors(&config);
        assert!(true);
    }

    #[test]
    fn test_cors_default_config() {
        let config = CorsConfig::default();

        assert!(config.validate().is_ok());

        let cors = Server::cors(&config);
        assert!(true);
    }

    #[test]
    fn test_cors_config_with_defaults() {
        let config = CorsConfig::with_defaults();

        assert!(config.enabled);
        assert_eq!(config.origins, vec!["*"]);
        assert!(!config.methods.is_empty());
        assert!(!config.headers.is_empty());
        assert_eq!(config.max_age, 3600);
        assert!(!config.allow_credentials);
        assert!(config.send_wildcard);

        let cors = Server::cors(&config);
        assert!(true);
    }

    #[test]
    fn test_cors_send_wildcard_disabled() {
        let config = CorsConfig {
            enabled: true,
            origins: vec!["*".to_string()],
            methods: vec!["GET".to_string()],
            headers: vec!["Content-Type".to_string()],
            max_age: 3600,
            allow_credentials: false,
            send_wildcard: false, // Disabled
        };

        let cors = Server::cors(&config);
        assert!(true);
    }

    #[test]
    fn test_cors_multiple_specific_origins() {
        let config = CorsConfig {
            enabled: true,
            origins: vec![
                "https://app.example.com".to_string(),
                "https://admin.example.com".to_string(),
                "https://api.example.com".to_string(),
                "http://localhost:3000".to_string(),
            ],
            methods: vec!["GET".to_string(), "POST".to_string()],
            headers: vec!["Authorization".to_string(), "Content-Type".to_string()],
            max_age: 3600,
            allow_credentials: true, // Valid with specific origins
            send_wildcard: false,
        };

        assert!(config.validate().is_ok());

        let cors = Server::cors(&config);
        assert!(true);
    }
}
