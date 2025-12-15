//! Async Dependency Injection Container for SaDi
//!
//! This module provides `AsyncContainer`, an async-aware version of the DI container
//! that supports async factory functions and non-blocking service resolution.
//!
//! This module is only available when the `async` feature is enabled.
//!
//! # Key Differences from Sync Container
//!
//! - Uses `AsyncFactory<T>` instead of `Factory<T>`
//! - Resolution methods are async (`resolve_async`)
//! - Proper circular dependency detection in async contexts
//! - Thread-safe singleton caching using `tokio::sync::Mutex`
//!
//! # Example
//!
//! ```ignore
//! use sadi::AsyncContainer;
//!
//! #[derive(Clone)]
//! struct Database {
//!     connection_string: String,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let c = AsyncContainer::new();
//!
//!     // Register async factory
//!     c.bind_async_concrete_singleton::<Database, Database, _, _>(|_| async {
//!         Database {
//!             connection_string: "postgresql://localhost:5432/myapp".to_string(),
//!         }
//!     }).await.unwrap();
//!
//!     // Resolve asynchronously
//!     let db = c.resolve_async::<Database>().await.unwrap();
//!     println!("Connected to: {}", db.connection_string);
//! }
//! ```

use crate::{AsyncFactory, Error, IntoShared, Shared};
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The async-aware IoC/DI container.
///
/// Similar to the sync `Container`, but with support for async factories and
/// non-blocking resolution. Uses `Arc<RwLock<HashMap>>` for thread-safe access.
#[derive(Clone)]
pub struct AsyncContainer {
    factories: Arc<RwLock<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>>,
}

impl AsyncContainer {
    /// Creates a new async container.
    pub fn new() -> Self {
        Self {
            factories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers an **abstract async binding** for trait objects or unsized types.
    ///
    /// Instances are **not cached** (transient).
    pub async fn bind_async_abstract<T, R, F, Fut>(&mut self, provider: F) -> Result<(), Error>
    where
        T: ?Sized + Send + Sync + 'static,
        R: IntoShared<T> + 'static,
        F: Fn(Arc<AsyncContainer>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = R> + Send + 'static,
    {
        let provider = Arc::new(provider);
        let provider_fn: crate::async_factory::AsyncProvider<T> = Box::new(move |c| {
            let provider = provider.clone();
            Box::pin(async move {
                let result = (provider)(c).await;
                result.into_shared()
            })
        });

        self.bind_internal::<T>(provider_fn, false).await
    }

    /// Registers a **singleton** abstract async binding.
    pub async fn bind_async_abstract_singleton<T, R, F, Fut>(
        &mut self,
        provider: F,
    ) -> Result<(), Error>
    where
        T: ?Sized + Send + Sync + 'static,
        R: IntoShared<T> + 'static,
        F: Fn(Arc<AsyncContainer>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = R> + Send + 'static,
    {
        let provider = Arc::new(provider);
        let provider_fn: crate::async_factory::AsyncProvider<T> = Box::new(move |c| {
            let provider = provider.clone();
            Box::pin(async move {
                let result = (provider)(c).await;
                result.into_shared()
            })
        });

        self.bind_internal::<T>(provider_fn, true).await
    }

    /// Registers a concrete async implementation.
    pub async fn bind_async_concrete<T, U, F, Fut>(&mut self, provider: F) -> Result<(), Error>
    where
        T: Send + Sync + 'static,
        U: 'static,
        F: Fn(Arc<AsyncContainer>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = U> + Send + 'static,
        Arc<U>: Into<Arc<T>>,
    {
        let provider = Arc::new(provider);
        let provider_fn: crate::async_factory::AsyncProvider<T> = Box::new(move |c| {
            let provider = provider.clone();
            Box::pin(async move {
                let instance = (provider)(c).await;
                Arc::new(instance).into()
            })
        });

        self.bind_internal::<T>(provider_fn, false).await
    }

    /// Registers a concrete async singleton implementation.
    pub async fn bind_async_concrete_singleton<T, U, F, Fut>(
        &mut self,
        provider: F,
    ) -> Result<(), Error>
    where
        T: Send + Sync + 'static,
        U: 'static,
        F: Fn(Arc<AsyncContainer>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = U> + Send + 'static,
        Arc<U>: Into<Arc<T>>,
    {
        let provider = Arc::new(provider);
        let provider_fn: crate::async_factory::AsyncProvider<T> = Box::new(move |c| {
            let provider = provider.clone();
            Box::pin(async move {
                let instance = (provider)(c).await;
                Arc::new(instance).into()
            })
        });

        self.bind_internal::<T>(provider_fn, true).await
    }

    /// Registers an already created instance as an async singleton.
    pub async fn bind_async_instance<T, R>(&mut self, instance: R) -> Result<(), Error>
    where
        T: ?Sized + Send + Sync + 'static,
        R: IntoShared<T> + 'static,
    {
        let shared = instance.into_shared();

        let provider_fn: crate::async_factory::AsyncProvider<T> = Box::new(move |_c| {
            let shared = shared.clone();
            Box::pin(async move { shared })
        });

        self.bind_internal::<T>(provider_fn, true).await
    }

    /// Internal binding logic shared by all async binding methods.
    async fn bind_internal<T>(&mut self, provider: crate::async_factory::AsyncProvider<T>, singleton: bool) -> Result<(), Error>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let id = TypeId::of::<T>();
        let name = std::any::type_name::<T>();

        let mut map = self.factories.write().await;

        if map.contains_key(&id) {
            return Err(Error::factory_already_registered(name, "async factory"));
        }

        let factory = AsyncFactory::new(provider, singleton);

        map.insert(id, Box::new(factory));

        Ok(())
    }

    /// Resolves a previously registered async binding.
    ///
    /// Performs:
    /// - Type lookup
    /// - Circular dependency detection
    /// - Singleton caching
    /// - Provider invocation (asynchronously)
    pub async fn resolve_async<T>(self: Arc<Self>) -> Result<Shared<T>, Error>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let id = TypeId::of::<T>();
        let name = std::any::type_name::<T>();

        // Async version of circular dependency detection
        let _guard = crate::AsyncResolveGuard::push(name).await?;

        let factory = {
            let map = self.factories.read().await;
            let boxed = map
                .get(&id)
                .ok_or_else(|| Error::service_not_registered(name, "async factory"))?;

            let factory = boxed
                .downcast_ref::<AsyncFactory<T>>()
                .ok_or_else(|| Error::type_mismatch(name))?;

            // Clone the factory arc to avoid borrow issues
            let factory_ptr = factory as *const AsyncFactory<T>;
            unsafe { &*factory_ptr }
        };

        Ok(factory.provide(self.clone()).await)
    }

    /// Returns `true` if a type has been registered.
    pub async fn has_async<T>(&self) -> bool
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let id = TypeId::of::<T>();
        self.factories.read().await.contains_key(&id)
    }
}

impl Default for AsyncContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to clear the async resolve guard stack
    #[allow(dead_code)]
    async fn clear_resolve_stack() {
        // This is a workaround to clear the lazy_static stack
        // In a real scenario, you'd want to implement Drop or a reset method
        // For now, we'll just use unique names in tests
    }

    struct Service {
        value: i32,
    }

    #[tokio::test]
    async fn test_async_bind_and_resolve_concrete() {
        let mut c = AsyncContainer::new();
        // Use a unique type name for this test
        c.bind_async_concrete::<Service, Service, _, _>(|_c| async { Service { value: 42 } })
            .await
            .unwrap();

        let c = Arc::new(c);
        let s = c.clone().resolve_async::<Service>().await.unwrap();
        assert_eq!(s.value, 42);
    }

    #[tokio::test]
    async fn test_async_singleton_behavior() {
        #[allow(dead_code)]
        struct AnotherService {
            value: i32,
        }

        let mut c = AsyncContainer::new();
        c.bind_async_concrete_singleton::<AnotherService, AnotherService, _, _>(|_c| async { AnotherService { value: 99 } })
            .await
            .unwrap();

        let c = Arc::new(c);
        let a = c.clone().resolve_async::<AnotherService>().await.unwrap();
        let b = c.clone().resolve_async::<AnotherService>().await.unwrap();

        let pa = (&*a) as *const AnotherService;
        let pb = (&*b) as *const AnotherService;
        assert_eq!(pa, pb);
    }

    #[tokio::test]
    async fn test_has_async() {
        #[allow(dead_code)]
        struct YetAnotherService {
            value: i32,
        }

        let mut c = AsyncContainer::new();
        assert!(!c.has_async::<YetAnotherService>().await);

        c.bind_async_concrete::<YetAnotherService, YetAnotherService, _, _>(|_c| async { YetAnotherService { value: 42 } })
            .await
            .unwrap();

        assert!(c.has_async::<YetAnotherService>().await);
    }
}
