# SaDi - Semi-automatic Dependency Injector

[![Crates.io](https://img.shields.io/crates/v/sadi.svg)](https://crates.io/crates/sadi)
[![Documentation](https://docs.rs/sadi/badge.svg)](https://docs.rs/sadi)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/JoaoPedro61/sadi/actions/workflows/CI.yml/badge.svg)](https://github.com/JoaoPedro61/sadi/actions/workflows/CI.yml)

A lightweight, type-safe dependency injection container for Rust applications. SaDi provides ergonomic service registration (including trait-object bindings), transient and singleton lifetimes, semi-automatic dependency resolution, and circular dependency detection.

## ✨ Features

- 🔒 **Type-Safe**: Leverages Rust's type system for compile-time safety
- 🔄 **Transient Services**: Create new instances on each request
- 🔗 **Singleton Services**: Shared instances with reference counting
- 🔍 **Circular Detection**: Prevents infinite loops in dependency graphs
- ❌ **Error Handling**: Comprehensive error types with detailed messages
- 📊 **Optional Logging**: Tracing integration with feature gates
- 🚀 **Zero-Cost Abstractions**: Feature gates enable compile-time optimization

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sadi = "0.2.1"
```

## 🚀 Quick Start

```rust
use sadi::{container, bind, Container, Shared};
use std::rc::Rc;

// Define your services (non-thread-safe default uses `Rc` via `Shared`)
struct DatabaseService {
    connection_string: String,
}

impl DatabaseService {
    fn new() -> Self {
        Self {
            connection_string: "postgresql://localhost:5432/myapp".to_string(),
        }
    }

    fn query(&self, sql: &str) -> String {
        format!("Executing '{}' on {}", sql, self.connection_string)
    }
}

struct UserService {
    db: Shared<DatabaseService>,
}

impl UserService {
    fn new(db: Shared<DatabaseService>) -> Self {
        Self { db }
    }

    fn create_user(&self, name: &str) -> String {
        self.db.query(&format!("INSERT INTO users (name) VALUES ('{}')", name))
    }
}

fn main() {
    // Use the `container!` macro to register bindings ergonomically
    let container = container! {
        bind(singleton DatabaseService => |_| DatabaseService::new())
        bind(UserService => |c| UserService::new(c.resolve::<DatabaseService>().unwrap()))
    };

    // Resolve and use services
    let user_service = container.resolve::<UserService>().unwrap();
    println!("{}", user_service.create_user("Alice"));
}
```

## 📖 Usage Guide

### Service Registration

#### Transient Services
Create new instances on each request. The default `bind` registration is transient:

```rust
use sadi::{container, bind};
use uuid::Uuid;

struct LoggerService {
    session_id: String,
}

let c = container! {
    bind(LoggerService => |_| LoggerService { session_id: Uuid::new_v4().to_string() })
};

let logger1 = c.resolve::<LoggerService>().unwrap();
let logger2 = c.resolve::<LoggerService>().unwrap();
```

#### Singleton Services
Create once and share across all dependents. Use the `singleton` annotation in `bind`:

```rust
use sadi::{container, bind, Shared};

struct ConfigService {
    app_name: String,
    debug: bool,
}

let c = container! {
    bind(singleton ConfigService => |_| ConfigService { app_name: "MyApp".to_string(), debug: true })
};

let config1 = c.resolve::<ConfigService>().unwrap();
let config2 = c.resolve::<ConfigService>().unwrap();
assert!(Shared::ptr_eq(&config1, &config2));
```

### Error Handling

SaDi provides both panicking and non-panicking variants:

```rust
use sadi::{Container, Error};

let c = Container::new();
c.bind_concrete::<String, String, _>(|_| "Hello".to_string()).unwrap();

// Resolve (panicking)
let service = c.resolve::<String>().unwrap();

// Non-panicking
match c.resolve::<String>() {
    Ok(s) => println!("Got: {}", s),
    Err(e) => println!("Error: {}", e),
}

// Trying to resolve an unregistered type
match c.resolve::<u32>() {
    Ok(_) => unreachable!(),
    Err(e) => println!("Expected error: {}", e),
}
```

### Dependency Injection

Services can depend on other services. Use the `container!` macro to register bindings concisely:

```rust
use sadi::{container, bind, Shared};

struct DatabaseService { /* ... */ }
impl DatabaseService { fn new() -> Self { DatabaseService {} } }

struct CacheService { /* ... */ }
impl CacheService { fn new() -> Self { CacheService {} } }

struct UserRepository {
    db: Shared<DatabaseService>,
    cache: Shared<CacheService>,
}

impl UserRepository {
    fn new(db: Shared<DatabaseService>, cache: Shared<CacheService>) -> Self {
        Self { db, cache }
    }
}

let c = container! {
    bind(singleton DatabaseService => |_| DatabaseService::new())
    bind(singleton CacheService => |_| CacheService::new())
    bind(UserRepository => |c| UserRepository::new(c.resolve::<DatabaseService>().unwrap(), c.resolve::<CacheService>().unwrap()))
};

let repo = c.resolve::<UserRepository>().unwrap();
```

## 🔍 Advanced Features

### Circular Dependency Detection

SaDi automatically detects and prevents circular dependencies:

```rust
use sadi::Container;

// Example: registering circular dependencies will produce a descriptive error at runtime
let c = Container::new();
// c.bind_concrete::<ServiceA, ServiceA, _>(|c| { let _ = c.resolve::<ServiceB>(); ServiceA });
// c.bind_concrete::<ServiceB, ServiceB, _>(|c| { let _ = c.resolve::<ServiceA>(); ServiceB });

match c.resolve::<ServiceA>() {
    Ok(_) => println!("unexpected"),
    Err(e) => println!("Circular dependency detected: {}", e),
}
```

### Tracing Integration

Enable the `tracing` feature for automatic logging (the crate's `default` feature includes `tracing`):

```toml
[dependencies]
sadi = { version = "0.2.1", features = ["tracing"] }
```

```rust
use sadi::{container, bind};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let c = container! {
        bind(singleton DatabaseService => |_| DatabaseService::new())
    };

    // resolving singletons or other services will be trace-logged when tracing feature is enabled
    let _db = c.resolve::<DatabaseService>().unwrap();
}
```

### Async Support

Enable the `async` feature to use async-aware dependency injection with full support for async factories and non-blocking service resolution:

```toml
[dependencies]
sadi = { version = "0.2.1", features = ["async"] }
tokio = { version = "1.35", features = ["full"] }
```

```rust
use sadi::AsyncContainer;
use std::sync::Arc;

#[derive(Clone)]
struct DatabaseConnection {
    connection_string: String,
}

impl DatabaseConnection {
    async fn connect(url: &str) -> Self {
        // Simulate async connection setup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Self {
            connection_string: url.to_string(),
        }
    }
}

#[derive(Clone)]
struct UserRepository {
    db: Arc<DatabaseConnection>,
}

#[tokio::main]
async fn main() {
    let mut container = AsyncContainer::new();

    // Register async singleton
    container
        .bind_async_concrete_singleton::<DatabaseConnection, DatabaseConnection, _, _>(
            |_| async { DatabaseConnection::connect("postgresql://localhost:5432/myapp").await }
        )
        .await
        .unwrap();

    // Register with dependency
    container
        .bind_async_concrete::<UserRepository, UserRepository, _, _>(|c| {
            let c = c.clone();
            async move {
                let db = c
                    .clone()
                    .resolve_async::<DatabaseConnection>()
                    .await
                    .unwrap();
                UserRepository { db }
            }
        })
        .await
        .unwrap();

    // Resolve asynchronously
    let container = Arc::new(container);
    let repo = container.clone().resolve_async::<UserRepository>().await.unwrap();
    
    println!("Connected to: {}", repo.db.connection_string);
}
```

#### Key Async Features

- **Async Factories**: Define async factory functions using `bind_async_concrete` and similar methods
- **Non-blocking Resolution**: Use `resolve_async` for non-blocking service resolution
- **Async Singleton Caching**: Singletons are properly cached and reused in async contexts using `tokio::sync::Mutex`
- **Circular Dependency Detection**: Async context maintains proper circular dependency detection with `AsyncResolveGuard`
- **Container Cloning**: `AsyncContainer` implements `Clone` and uses `Arc` internally for safe sharing across async tasks

## 🧪 Testing

Run the test suite:

```bash
# Run all tests for the workspace
cargo test

# Run tests for the sadi crate only
cargo test -p sadi

# Run with tracing feature
cargo test --features tracing

# Run with async feature
cargo test -p sadi --features async

# Run documentation tests
cargo test --doc -p sadi

# Run basic example
cargo run -p basic

# Run async example (requires async feature)
cargo run -p async_example
```

## 📁 Project Structure

```
sadi/
├── sadi/                      # library crate with main DI container
│   ├── src/
│   │   ├── container.rs       # Sync container implementation
│   │   ├── async_container.rs # Async container (requires "async" feature)
│   │   ├── factory.rs         # Factory wrapper for sync services
│   │   ├── async_factory.rs   # Async factory wrapper (requires "async" feature)
│   │   ├── resolve_guard.rs   # Circular dependency detection (sync)
│   │   ├── async_resolve_guard.rs # Circular dependency detection (async)
│   │   ├── error.rs           # Error types
│   │   ├── shared.rs          # Shared pointer abstraction
│   │   ├── types.rs           # Type aliases and helpers
│   │   ├── macros.rs          # Container macro
│   │   └── lib.rs             # Library entry point
│   └── Cargo.toml
├── examples/
│   ├── basic/                 # Synchronous DI example
│   └── async_example/         # Asynchronous DI example (requires "async" feature)
├── tests/                     # Integration tests
├── Cargo.toml                 # Workspace configuration
└── README.md                  # This file

## 🔧 Configuration

### Feature Flags

SaDi exposes a small set of feature flags. See `sadi/Cargo.toml` for the authoritative list, but the crate currently defines:

- `thread-safe` (enabled by default) — switches internal shared pointer and synchronization primitives to `Arc` + `RwLock`/`Mutex` for thread-safe containers.
- `tracing` (enabled by default) — integrates with the `tracing` crate to emit logs during registration/resolution.
- `async` (disabled by default) — enables async-aware dependency injection with `AsyncContainer`, async factories, and non-blocking service resolution. Requires `tokio`.

The workspace default enables both `thread-safe` and `tracing`. To opt out of thread-safe behavior (use `Rc` instead of `Arc`), disable the `thread-safe` feature. To enable async support, enable the `async` feature.

### Environment Variables

When using the tracing feature, you can control logging levels:

```bash
# Set log level
RUST_LOG=debug cargo run --example basic

# Enable only SaDi logs
RUST_LOG=sadi=info cargo run --example basic
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Clone the repository:
```bash
git clone https://github.com/JoaoPedro61/sadi.git
cd sadi
```

2. Run tests:
```bash
cargo test --all-features
```

3. Check formatting:
```bash
cargo fmt --check
```

4. Run clippy:
```bash
cargo clippy -- -D warnings
```

## 📋 Roadmap & TODO

### 🔄 Async Support
- [x] **Async Factory Functions**: Support for `async fn` factories
- [x] **Async Service Resolution**: Non-blocking service creation
- [x] **Async Singleton Caching**: Thread-safe async singleton management
- [x] **Async Circular Detection**: Proper handling in async contexts

### 🧵 Thread Safety
- [x] **Arc-based Container**: Thread-safe version of SaDi using `Arc` instead of `Rc` (implemented behind the `thread-safe` feature)
- [x] **Send + Sync Services**: Support for `Send + Sync` services in thread-safe mode (enforced by API bounds)
- [x] **Concurrent Access**: Concurrent reads/writes supported via `RwLock`/`Mutex` in thread-safe mode
- [ ] **Lock-free Operations**: Minimize contention in high-concurrency scenarios

### 🔧 Advanced Features
- [ ] **Service Scoping**: Request-scoped, thread-scoped service lifetimes
- [x] **Lazy Initialization**: Singleton instances are created on first `provide` (implemented in `Factory`)
- [ ] **Service Decorators**: Middleware/decoration patterns for services
- [ ] **Conditional Registration**: Register services based on runtime conditions
- [ ] **Service Health Checks**: Verify singleton services are healthy before returning them
- [ ] **Service Metrics**: Performance and usage statistics

### 📦 Ecosystem Integration
- [ ] **Tokio Integration**: First-class support for Tokio runtime
- [ ] **Actix-web Plugin**: Direct integration with Actix-web framework
- [ ] **Axum Integration**: Support for Axum web framework
- [ ] **Tower Service**: Implement Tower service trait
- [ ] **Serde Support**: Serialize/deserialize container configuration

### 🛠️ Developer Experience
- [ ] **Derive Macros**: Auto-generate factory functions from service structs
- [ ] **Builder Validation**: Compile-time validation of dependency graphs
- [ ] **Error Suggestions**: Better error messages with fix suggestions
- [ ] **IDE Integration**: Language server support for dependency analysis
- [ ] **Container Visualization**: Graphical representation of service dependencies

### 🔒 Security & Reliability
- [ ] **Concurrency Limits**: Limit concurrent service creations per binding
- [ ] **Graceful Shutdown**: Proper cleanup on container disposal
- [ ] **Retry Policies**: Retry factory invocation on transient failures

### 📊 Observability
- [ ] **OpenTelemetry**: Built-in telemetry and distributed tracing
- [ ] **Prometheus Metrics**: Expose container metrics for monitoring
- [ ] **Service Discovery**: Integration with service discovery systems

### 🎯 Performance
- [ ] **Compile-time validation / Builder checks**: Improve compile-time validation and builder-time checks for dependency graphs
- [ ] **Service Pooling**: Object pooling for expensive-to-create services
- [ ] **Memory Optimization**: Reduced memory footprint for large containers

### 📚 Long-term Wishlist
- [ ] **Hot Reloading**: Dynamic service replacement without container restart (large, architecture-level feature; moved to long-term wishlist)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/JoaoPedro61/sadi/blob/main/LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by dependency injection patterns from other languages and frameworks
- Built with ❤️ using Rust's powerful type system
- Thanks to the Rust community for excellent crates and documentation

---

**Made with ❤️ by [João Pedro Martins](https://github.com/JoaoPedro61)**
