# lighter-common

A production-ready Rust library providing shared utilities and infrastructure for building RESTful web services in the lighter ecosystem. This library serves as the foundation for all lighter microservices, offering consistent patterns for HTTP servers, database connectivity, API responses, and common utilities.

## Project Purpose

lighter-common is designed to:
- Reduce boilerplate across lighter microservices (auth, etc.)
- Enforce consistent API patterns and error handling
- Provide battle-tested infrastructure components
- Enable rapid service development with production-ready defaults
- Maintain type safety and compile-time guarantees across services

## Architecture & Design Patterns

### 1. Prelude Pattern
The library exports a comprehensive prelude module for ergonomic imports:

```rust
use lighter_common::prelude::*;
```

This gives you access to:
- actix-web types (HttpRequest, HttpResponse, Json, Query, etc.)
- Database types (DatabaseConnection, sea_orm utilities)
- Response types (Error, Validation, pagination derives)
- Time utilities (now, unix)
- Common traits and re-exports

**File**: `src/prelude.rs`

### 2. Builder Pattern for Server
The Server struct uses a builder pattern for configuration:

```rust
let server = Server::env().await;  // From environment
// or
let mut server = Server::new(port, db);
server.tls(tls_config);
server.run(router::route)?.await
```

**File**: `src/server.rs`

### 3. Single Error Enum
All HTTP errors use a unified Error enum with automatic conversions:

```rust
pub enum Error {
    BadRequest { message: String },
    Unauthorized { message: String },
    Forbidden { message: String },
    NotFound { message: String },
    UnprocessableEntity { errors: HashMap<String, Vec<String>> },
    InternalServerError { message: String },
}
```

Implements From<DbErr>, From<serde_json::Error>, Responder, and ResponseError.

**File**: `src/responses/error.rs`

### 4. Pagination Derive Macros
Automatic pagination struct generation:

```rust
#[derive(PaginationRequest, PaginationResponse)]
pub struct User {
    #[order]  // Makes field orderable
    pub name: String,
    #[order(default)]  // Default order field
    pub created_at: NaiveDateTime,
}
```

This generates:
- `UserPaginationRequest` with page, limit, search, sort, order
- `UserPaginationResponse` with total, page, pages, data
- `UserPaginationSort` enum (Asc, Desc)
- `UserPaginationOrder` enum from orderable fields

**File**: `derives/src/pagination.rs`

### 5. OpenAPI/Swagger Integration
Built-in OpenAPI schema support with utoipa:

```rust
use lighter_common::api::{Builtin, Authentication};

#[derive(OpenApi)]
#[openapi(
    modifiers(&Builtin, &Authentication),
    // ... your paths and schemas
)]
pub struct ApiDoc;
```

**File**: `src/api.rs`

## Technology Stack

### Core Dependencies
- **actix-web 4.12.1** - HTTP server with HTTP/2 and rustls TLS support
- **SeaORM 1.1.19** - Async ORM with tokio-rustls runtime
- **utoipa 5.4.0** - OpenAPI schema generation
- **utoipa-swagger-ui 9.0.2** - Embedded Swagger UI
- **rustls 0.23.35** - Modern TLS implementation
- **serde 1.0.228** - Serialization framework
- **tracing 0.1.43** - Structured logging
- **tracing-subscriber 0.3.22** - Log collection with env-filter, JSON, chrono

### Cryptography & Encoding
- **sha2 0.10.9** - SHA-256 hashing
- **bs58 0.5.1** - Base58 encoding
- **hex 0.4.3** - Hexadecimal encoding

### Time & IDs
- **chrono 0.4.42** - Date/time handling
- **uuid 1.19.0** - UUID v4 generation

### Database Features
- **postgres** (default) - PostgreSQL via sqlx-postgres
- **sqlite** - SQLite via sqlx-sqlite

## Development Workflow

### Common Tasks

#### 1. Starting a New Service with lighter-common

```rust
// Cargo.toml
[dependencies]
lighter-common = { path = "../common" }
# or from git:
# lighter-common = { git = "https://github.com/Geriano/lighter-common" }

// main.rs
use lighter_common::prelude::*;

#[actix::main]
async fn main() -> Result<(), std::io::Error> {
    tracing::init();

    let server = Server::env().await;
    server.run(router::route)?.await
}
```

#### 2. Creating API Endpoints

```rust
use lighter_common::prelude::*;

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "User",
    responses(UserPaginationResponse, Error)
)]
pub async fn paginate(
    db: Data<DatabaseConnection>,
    query: Query<UserPaginationRequest>,
) -> Result<Json<UserPaginationResponse>, Error> {
    // Use pagination helpers
    let page = query.page();
    let limit = query.limit();
    let offset = query.offset();
    let sort = query.sort();  // sea_orm::Order

    // Database query with SeaORM
    let users = entity::User::find()
        .offset(offset)
        .limit(limit)
        .order_by(entity::user::Column::CreatedAt, sort)
        .all(db.as_ref())
        .await?;

    let total = entity::User::find().count(db.as_ref()).await?;

    Ok(Json(UserPaginationResponse {
        total,
        page: query.page(),
        pages: (total + limit - 1) / limit,
        data: users.into_iter().map(|u| u.into()).collect(),
    }))
}
```

#### 3. Error Handling Patterns

```rust
use lighter_common::prelude::*;

pub async fn update(id: Uuid, db: Data<DatabaseConnection>) -> Result<User, Error> {
    // Automatic conversion from DbErr to Error::InternalServerError
    let user = entity::User::find_by_id(id)
        .one(db.as_ref())
        .await?
        .ok_or(Error::NotFound {
            message: "User not found".to_string()
        })?;

    // Validation errors
    let mut validation = Validation::new();
    if user.email.is_empty() {
        validation.add("email", "Email is required");
    }
    if !validation.is_empty() {
        return Err(validation.into());
    }

    Ok(user.into())
}
```

#### 4. Using Hash Utilities

```rust
use lighter_common::prelude::*;
use lighter_common::hash::Hash;

// Create hash with salt
let salt = Uuid::new_v4();
let hash = Hash::make(salt, "password123");

// Verify hash
if hash.verify(salt, "password123") {
    // Password correct
}

// Display as hex string
println!("{}", hash);  // Outputs hex-encoded hash
```

#### 5. Time Utilities

```rust
use lighter_common::time;

// Current timestamp
let current = time::now();  // NaiveDateTime
let unix_ms = time::unix();  // u64 milliseconds

// Parse from string
let parsed = time::from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")?;

// Format to string
let formatted = time::format(current);  // "YYYY-MM-DD HH:MM:SS"

// From unix timestamp
let from_unix = time::from_unix(1609459200000);
```

#### 6. Database Connection

```rust
use lighter_common::database;

// From environment variables (DATABASE_URL)
let db = database::env().await?;

// Direct connection
let db = database::connect("postgresql://user:pass@localhost/db").await?;

// In-memory SQLite for testing
let db = database::memory().await?;
```

#### 7. TLS Configuration

```rust
use lighter_common::tls;
use lighter_common::prelude::*;

let tls_config = tls::configure("cert.pem", "key.pem");

let mut server = Server::env().await;
server.tls(tls_config);
server.run(router)?.await
```

#### 8. Base58 Encoding

```rust
use lighter_common::base58;

// Encode to Vec<u8>
let encoded = base58::encode(b"hello");

// Encode to String
let encoded_str = base58::to_string(b"hello");

// Decode
let decoded = base58::decode(encoded)?;
```

### Building and Testing

```bash
# Build with default features (PostgreSQL)
cargo build

# Build with SQLite
cargo build --no-default-features --features sqlite

# Run tests
cargo test

# Check all workspace members
cargo check --workspace

# Build optimized for size
cargo build --release
```

### Feature Flags

```toml
[features]
default = ["postgres"]
postgres = ["sea-orm/sqlx-postgres"]
sqlite = ["sea-orm/sqlx-sqlite"]
```

Services using lighter-common should propagate these features:

```toml
[features]
postgres = ["lighter-common/postgres", "sea-orm/sqlx-postgres"]
sqlite = ["lighter-common/sqlite", "sea-orm/sqlx-sqlite"]
```

## Production Readiness Checklist

### Currently Implemented
- [x] Structured logging with tracing + tracing-subscriber
- [x] Comprehensive error handling with unified Error enum
- [x] TLS support with rustls
- [x] Database connection pooling via SeaORM
- [x] OpenAPI/Swagger documentation support
- [x] Feature flags for database selection
- [x] Pagination utilities with derive macros
- [x] HTTP/2 support via actix-web
- [x] Thread IDs in logs for debugging

### Missing/Needs Implementation

#### CRITICAL - Metrics Implementation
**Status**: NOT IMPLEMENTED

Per global CLAUDE.md requirements, metrics MUST be implemented:

```rust
// Required implementation:
use std::sync::atomic::{AtomicU64, Ordering};

pub mod metrics {
    pub struct Metrics {
        pub http_requests_total: AtomicU64,
        pub http_request_duration_ms: AtomicU64,
        pub db_queries_total: AtomicU64,
        pub db_query_duration_ms: AtomicU64,
        pub errors_total: AtomicU64,
    }

    impl Metrics {
        pub fn export_prometheus(&self) -> String {
            // Format metrics in Prometheus format
        }
    }
}

// Metrics must be exposed via:
// - HTTP /metrics endpoint
// - SSE /metrics/stream
// - WebSocket /metrics/ws (optional)
```

**Action Required**: Add metrics crate or use atomic counters for simple metrics.

**Files to Create**:
- `src/metrics.rs`
- `src/metrics/middleware.rs`
- `src/metrics/export.rs`

#### CRITICAL - Error Handling Enhancement
**Status**: PARTIALLY IMPLEMENTED

Current implementation uses custom Error enum, but global CLAUDE.md requires:
- `anyhow` for application errors
- `thiserror` for library errors

**Action Required**:
```rust
// Add to Cargo.toml
anyhow = "1.0"
thiserror = "1.0"

// Refactor Error enum to use thiserror:
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Bad request: {message}")]
    BadRequest { message: String },

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    // etc.
}
```

**File**: `src/responses/error.rs`

#### CRITICAL - Tracing Enhancement
**Status**: BASIC IMPLEMENTATION

Current tracing is basic. Need to add:
- Tracing instrument on key functions
- Structured fields in logs
- Correlation IDs

```rust
use tracing::instrument;

#[instrument(skip(db), fields(user_id = %user_id))]
pub async fn get_user(user_id: Uuid, db: &DatabaseConnection) -> Result<User, Error> {
    tracing::info!("Fetching user");
    // ...
}
```

**File**: `src/tracing.rs`

#### HIGH - CORS Configuration
**Status**: TOO PERMISSIVE

Current implementation uses `Cors::permissive()` which allows all origins.

```rust
// File: src/server.rs:133
pub fn cors() -> Cors {
    Cors::permissive()  // SECURITY RISK!
}
```

**Action Required**:
```rust
pub fn cors() -> Cors {
    Cors::default()
        .allowed_origin_fn(|origin, _req_head| {
            // Check against allowed origins from env
            true
        })
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
        .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(3600)
}
```

**File**: `src/server.rs`

#### HIGH - Integration Tests
**Status**: NOT IMPLEMENTED

Need integration tests for:
- Server startup and routing
- Database connections (Postgres & SQLite)
- Error response formatting
- Pagination helpers
- TLS configuration

**Directory to Create**: `tests/`

#### MEDIUM - Documentation
**Status**: MISSING

Need:
- README.md with quick start guide
- API documentation (cargo doc)
- Usage examples

**Files to Create**:
- `README.md`
- `examples/simple_server.rs`
- `examples/pagination.rs`
- `examples/error_handling.rs`

#### MEDIUM - Database Connection Timeouts
**Status**: COMMENTED OUT

Connection timeouts are currently disabled:

```rust
// File: src/database.rs:7-10
// .idle_timeout(Duration::from_millis(1000))
// .connect_timeout(Duration::from_millis(1000))
// .acquire_timeout(Duration::from_millis(1000))
```

**Action Required**: Enable timeouts with sensible defaults from environment variables.

#### LOW - Macros Workspace
**Status**: PLACEHOLDER

The macros workspace member exists but only has placeholder code:

```rust
// File: macros/src/lib.rs
pub fn add(left: usize, right: usize) -> usize {
    left + right
}
```

**Decision Required**: Either implement useful macros or remove this workspace member.

## Integration with lighter Ecosystem

### Consuming Services

lighter-common is used by:
- **lighter-auth** - Authentication and authorization service

### Usage Pattern in Services

```rust
// Service Cargo.toml
[dependencies]
lighter-common = { path = "../common" }

// Propagate database features
[features]
postgres = ["lighter-common/postgres"]
sqlite = ["lighter-common/sqlite"]

// Service main.rs
use lighter_common::prelude::*;

#[actix::main]
async fn main() -> Result<(), std::io::Error> {
    tracing::init();
    let server = Server::env().await;
    server.run(router::route)?.await
}

// Service API documentation
use lighter_common::api::{Builtin, Authentication};

#[derive(OpenApi)]
#[openapi(
    modifiers(&Builtin, &Authentication),
    // ... service-specific paths
)]
pub struct ApiDoc;
```

### Example: lighter-auth Integration

The lighter-auth service demonstrates full integration:

1. **Prelude usage** in all modules
2. **Pagination derives** on User, Permission, Role responses
3. **Error handling** with unified Error type
4. **OpenAPI** with Builtin and Authentication modifiers
5. **Database** via Server::env()
6. **Base58** encoding for tokens

## Key Implementation Files

### Core Infrastructure
- `src/lib.rs` - Module exports
- `src/prelude.rs` - Prelude pattern
- `src/server.rs` - HTTP server builder

### Database
- `src/database.rs` - Connection utilities

### Responses
- `src/responses/error.rs` - Unified error type
- `src/responses/validation.rs` - Validation helper
- `src/responses/message.rs` - Success response
- `src/responses/schema.rs` - HTTP status schemas

### Pagination
- `derives/src/pagination.rs` - Derive macro logic

### Utilities
- `src/hash.rs` - SHA-256 hashing
- `src/base58.rs` - Base58 encoding
- `src/time.rs` - Time utilities
- `src/tls.rs` - TLS configuration
- `src/tracing.rs` - Logging setup

### OpenAPI
- `src/api.rs` - OpenAPI modifiers

## Environment Variables

```bash
# Required
DATABASE_URL=postgresql://user:pass@localhost:5432/dbname
# or
DATABASE_URL=sqlite://path/to/db.sqlite
# or
DATABASE_URL=sqlite::memory:

# Optional
PORT=3000  # Default: 3000

# Future (when timeouts enabled)
DATABASE_IDLE_TIMEOUT=1000
DATABASE_CONNECT_TIMEOUT=1000
DATABASE_ACQUIRE_TIMEOUT=1000

# Tracing (via env-filter)
RUST_LOG=info,lighter_common=debug
```

## Contributing Guidelines

### Code Style
- Follow Rust 2024 edition conventions
- Use `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Add doc comments to public APIs

### Testing Requirements
- Unit tests for utilities (hash, base58, time)
- Integration tests for server and database
- Example code must compile and run

### Breaking Changes
Since this is a shared library, breaking changes require:
1. Coordination with all consuming services
2. Migration guide in commit message
3. Version bump following SemVer

## Performance Considerations

### Server Configuration
- Workers: 4 (hardcoded in server.rs)
- Payload size: usize::MAX (no limit - consider adding configurable limit)
- JSON/Form size: usize::MAX (no limit - consider adding configurable limit)

### Database
- SeaORM uses connection pooling by default
- Consider enabling connection timeouts for production

### Logging
- JSON logging available via tracing-subscriber features
- Thread IDs enabled for debugging concurrent requests

## Deployment Notes

### Docker
Services using lighter-common should use multi-stage builds:

```dockerfile
FROM rust:1.90 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/app /usr/local/bin/
CMD ["app"]
```

### TLS
When using TLS, the server automatically:
- Binds port 80 for HTTP (redirects)
- Binds configured port for HTTPS
- Requires cert.pem and key.pem files

### Database Migrations
Use SeaORM migration system in consuming services:

```rust
use sea_orm_migration::prelude::*;

// In service migration workspace
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![/* migrations */]
    }
}
```

## Troubleshooting

### Common Issues

**Error: "PORT environment variable must be a number"**
- Ensure PORT is set to valid u16 (0-65535)

**Error: "DATABASE_URL environment variable must be set"**
- Set DATABASE_URL in .env or environment

**Build fails with feature errors**
- Ensure either postgres or sqlite feature is enabled
- Check that consuming service propagates features correctly

**CORS errors in browser**
- Current implementation is permissive (Cors::permissive())
- If restricting CORS, update Server::cors() method

**TLS certificate errors**
- Ensure cert.pem and key.pem are valid PKCS8 format
- Check file permissions

## Future Roadmap

### Short Term (Next Sprint)
1. Implement metrics with atomic counters
2. Add Prometheus export endpoint
3. Fix CORS configuration
4. Enable database connection timeouts
5. Add integration tests

### Medium Term
1. Implement anyhow/thiserror for error handling
2. Add tracing instrumentation
3. Create README and examples
4. Add rate limiting middleware
5. Implement request correlation IDs

### Long Term
1. WebSocket support for real-time features
2. gRPC support alongside HTTP
3. Distributed tracing (OpenTelemetry)
4. Advanced connection pooling strategies
5. Circuit breaker pattern for resilience

---

## Quick Reference

```rust
// Import everything you need
use lighter_common::prelude::*;

// Create server from environment
let server = Server::env().await;

// Run with routing function
server.run(router::route)?.await

// Database connection
let db = database::env().await?;

// Pagination on response struct
#[derive(PaginationRequest, PaginationResponse)]
pub struct User {
    #[order(default)]
    pub created_at: NaiveDateTime,
}

// Hash with salt
let hash = Hash::make(salt, message);
hash.verify(salt, message);

// Time utilities
let now = time::now();
let unix = time::unix();

// Base58 encoding
let encoded = base58::to_string(bytes);
let decoded = base58::decode(encoded)?;

// Error handling
return Err(Error::NotFound {
    message: "Not found".to_string()
});

// Validation
let mut v = Validation::new();
v.add("field", "Error message");
if !v.is_empty() {
    return Err(v.into());
}
```
