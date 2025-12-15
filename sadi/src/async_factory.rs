//! Async service factory wrapper for SaDi.
//!
//! This module defines [`AsyncFactory<T>`], which wraps an async provider closure and manages
//! singleton or transient service lifetimes in async contexts. It uses tokio primitives for
//! thread-safe async operations and ensures singletons are cached and reused.
//!
//! This module is only available when the `async` feature is enabled.

use crate::Shared;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Type alias for an async service provider (async factory function).
///
/// The provider receives a shared Arc reference to the AsyncContainer and returns a Future
/// that resolves to a shared instance.
pub type AsyncProvider<T> = Box<
    dyn Fn(Arc<crate::async_container::AsyncContainer>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Shared<T>> + Send>>
        + Send
        + Sync
        + 'static,
>;

/// Wraps an async provider closure and manages singleton or transient lifetimes.
///
/// - If `singleton` is true, the first call to `provide` caches the instance and all
///   subsequent calls return the same shared pointer.
/// - If `singleton` is false, each call to `provide` returns a new instance.
///
/// Thread safety is ensured via `tokio::sync::Mutex`.
pub struct AsyncFactory<T: ?Sized + 'static> {
    provider: AsyncProvider<T>,
    singleton: bool,
    instance: Arc<Mutex<Option<Shared<T>>>>,
}

impl<T: ?Sized + 'static> AsyncFactory<T> {
    /// Create a new async factory.
    ///
    /// - `provider`: async closure that produces a `Shared<T>`
    /// - `singleton`: if true, cache and reuse the instance
    pub fn new(provider: AsyncProvider<T>, singleton: bool) -> Self {
        Self {
            provider,
            singleton,
            instance: Arc::new(Mutex::new(None)),
        }
    }

    /// Provide an instance of the service asynchronously.
    ///
    /// - If singleton, returns the cached instance or creates and caches it.
    /// - If transient, always calls the provider.
    pub async fn provide(&self, container: Arc<crate::async_container::AsyncContainer>) -> Shared<T> {
        if self.singleton {
            let mut guard = self.instance.lock().await;
            if let Some(inst) = guard.as_ref() {
                return inst.clone();
            }
            let inst = (self.provider)(container).await;
            *guard = Some(inst.clone());
            inst
        } else {
            (self.provider)(container).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_container::AsyncContainer;

    #[allow(dead_code)]
    struct AsyncCounter(u32);

    #[tokio::test]
    async fn test_async_transient_factory() {
        let provider: AsyncProvider<AsyncCounter> = Box::new(|_c| {
            Box::pin(async { Arc::new(AsyncCounter(42)) })
        });

        let f = AsyncFactory::new(provider, false); // transient
        let c = Arc::new(AsyncContainer::new());

        let a = f.provide(c.clone()).await;
        let b = f.provide(c.clone()).await;

        // Different instances for transient
        assert!(!Arc::ptr_eq(&a, &b));
    }

    #[tokio::test]
    async fn test_async_singleton_factory() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = counter.clone();

        let provider: AsyncProvider<AsyncCounter> = Box::new(move |_c| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let val = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Arc::new(AsyncCounter(val))
            })
        });

        let f = AsyncFactory::new(provider, true); // singleton
        let c = Arc::new(AsyncContainer::new());

        let a = f.provide(c.clone()).await;
        let b = f.provide(c.clone()).await;

        // Same instance for singleton
        assert!(Arc::ptr_eq(&a, &b));
        // Factory should have been called only once
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
