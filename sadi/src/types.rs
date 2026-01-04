//! Internal type aliases and helpers for SaDi's container implementation.
//!
//! This module provides thread-safe and non-thread-safe variants of core types
//! (such as provider functions, instance cells, and factory maps) depending on
//! the enabled features:
//! - `thread-safe`: makes shared state `Send + Sync` and uses `Arc`, `Mutex`, and `RwLock`.
//! - `async`: enables asynchronous providers that return `BoxFuture` and implies thread-safety.
//!
//! These types are used internally by the DI container to manage service
//! registration and resolution.
use crate::Shared;
#[cfg(all(feature = "async"))]
use std::pin::Pin;
use std::{any::TypeId, collections::HashMap};

use crate::Container;


#[cfg(all(feature = "thread-safe", not(feature = "async")))]
pub type InnerProvider<T> = dyn Fn(&Container) -> Shared<T> + Send + Sync + 'static;
#[cfg(all(feature = "async"))]
pub type InnerProvider<T> = dyn for<'a> Fn(&'a Container)
            -> Pin<Box<dyn Future<Output = Shared<T>> + Send + 'a>>
        + Send
        + Sync
        + 'static;
#[cfg(all(not(feature = "thread-safe"), not(feature = "async")))]
pub type InnerProvider<T> = dyn Fn(&Container) -> Shared<T> + 'static;

/// Type alias for a service provider (factory function).
///
/// - In thread-safe mode, the provider closure must be `Send + Sync`.
/// - In async mode, the provider returns a `BoxFuture<'static, Shared<T>>` and is `Send + Sync`.
/// - In single-threaded mode, only `'static` is required and the provider returns `Shared<T>` directly.
///
/// The provider receives a reference to the container and returns a shared instance.
///
/// # Examples
///
/// Sync provider (default single-threaded):
/// ```ignore
/// use sadi::{Container, Shared, Provider};
///
/// let provider: Provider<u32> = Box::new(|_c: &Container| -> Shared<u32> {
///     Shared::new(41 + 1)
/// });
/// ```
///
/// Async provider (with `async` feature):
/// ```ignore
/// use sadi::{Container, Shared, Provider};
/// use futures::future;
///
/// let provider: Provider<u32> = Box::new(|_c: &Container| {
///     Box::pin(async move { Shared::new(42) })
/// });
/// ```
pub type Provider<T> = Box<InnerProvider<T>>;

/// Type alias for a cell holding a singleton/shared instance.
///
/// - In thread-safe/async modes, uses `Mutex` for safe concurrent access.
/// - In single-threaded mode, uses `RefCell` for fast interior mutability.
#[cfg(any(feature = "thread-safe", feature = "async"))]
pub type InstanceCell<T> = std::sync::Mutex<Option<Shared<T>>>;
#[cfg(all(not(feature = "thread-safe"), not(feature = "async")))]
pub type InstanceCell<T> = std::cell::RefCell<Option<Shared<T>>>;

/// Type alias for the map storing all registered service factories.
///
/// - In thread-safe/async modes, uses `RwLock` for concurrent reads/writes and
///   requires all stored factories to be `Send + Sync`.
/// - In single-threaded mode, uses `RefCell` for fast interior mutability.
#[cfg(any(feature = "thread-safe", feature = "async"))]
pub type FactoriesMap = std::sync::RwLock<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>;
#[cfg(all(not(feature = "thread-safe"), not(feature = "async")))]
pub type FactoriesMap = std::cell::RefCell<HashMap<TypeId, Box<dyn std::any::Any>>>;
