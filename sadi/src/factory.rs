//! Service factory wrapper for SaDi.
//!
//! This module defines [`Factory<T>`], which wraps a provider closure and manages
//! singleton or transient service lifetimes. It uses the appropriate cell/mutex for
//! thread safety, and ensures singletons are cached and reused.
//!
//! # Example
//! ```
//! use sadi::Factory;
//! use sadi::{Shared, Container};
//! use std::cell::Cell;
//!
//! struct Counter(Cell<u32>);
//! impl Counter {
//!     fn inc(&self) -> u32 {
//!         let v = self.0.get();
//!         self.0.set(v+1);
//!         v+1
//!     }
//! }
//!
//! let provider = Box::new(|_c: &Container| Shared::new(Counter(Cell::new(0))));
//!
//! let f = Factory::new(provider, false); // singleton
//! let c = Container::new();
//!
//! let a = f.provide(&c);
//! let b = f.provide(&c);
//!
//! assert!(!Shared::ptr_eq(&a, &b)); // same instances
//! ```
//!
//! For singleton, use `true` for the second argument and verify the same instance is reused.

use crate::{Container, InstanceCell, Provider, Shared};

#[cfg(feature = "async")]
use std::sync::Arc;
#[cfg(feature = "async")]
use std::future::Future;
#[cfg(feature = "async")]
use std::pin::Pin;

/// Async provider function type.
///
/// When the `async` feature is enabled, this type represents an async factory function
/// that produces a `Shared<T>` from an `Arc<Container>`.
#[cfg(feature = "async")]
pub type AsyncProvider<T> = Arc<
    dyn Fn(Arc<Container>) -> Pin<Box<dyn Future<Output = Shared<T>> + Send + 'static>>
        + Send
        + Sync,
>;

/// Wraps a provider closure and manages singleton or transient lifetimes.
///
/// - If `singleton` is true, the first call to `provide` caches the instance and all
///   subsequent calls return the same shared pointer.
/// - If `singleton` is false, each call to `provide` returns a new instance.
///
/// Thread safety is handled by the `InstanceCell` type alias.
pub struct Factory<T: ?Sized + 'static> {
    provider: Provider<T>,
    singleton: bool,
    instance: InstanceCell<T>,
    #[cfg(feature = "async")]
    async_provider: Option<AsyncProvider<T>>,
    #[cfg(feature = "async")]
    async_instance: Option<Arc<tokio::sync::Mutex<Option<Shared<T>>>>>,
}

impl<T: ?Sized + 'static> Factory<T> {
    /// Create a new factory.
    ///
    /// - `provider`: closure that produces a `Shared<T>`
    /// - `singleton`: if true, cache and reuse the instance
    pub fn new(provider: Provider<T>, singleton: bool) -> Self {
        Self {
            provider,
            singleton,
            instance: {
                #[cfg(feature = "thread-safe")]
                {
                    std::sync::Mutex::new(None)
                }
                #[cfg(not(feature = "thread-safe"))]
                {
                    std::cell::RefCell::new(None)
                }
            },
            #[cfg(feature = "async")]
            async_provider: None,
            #[cfg(feature = "async")]
            async_instance: None,
        }
    }

    /// Create a new async factory (only available with the `async` feature).
    ///
    /// - `provider`: async closure that produces a `Shared<T>`
    /// - `singleton`: if true, cache and reuse the instance
    #[cfg(feature = "async")]
    pub fn new_async(provider: AsyncProvider<T>, singleton: bool) -> Self {
        Self {
            provider: Box::new(|_| unreachable!("sync provider should not be called for async factory")),
            singleton,
            instance: {
                #[cfg(feature = "thread-safe")]
                {
                    std::sync::Mutex::new(None)
                }
                #[cfg(not(feature = "thread-safe"))]
                {
                    std::cell::RefCell::new(None)
                }
            },
            async_provider: Some(provider),
            async_instance: if singleton {
                Some(Arc::new(tokio::sync::Mutex::new(None)))
            } else {
                None
            },
        }
    }

    /// Provide an instance of the service.
    ///
    /// - If singleton, returns the cached instance or creates and caches it.
    /// - If transient, always calls the provider.
    pub fn provide(&self, container: &Container) -> Shared<T> {
        if self.singleton {
            // thread-safe branch
            #[cfg(feature = "thread-safe")]
            {
                let mut guard = self.instance.lock().unwrap();
                if let Some(inst) = guard.as_ref() {
                    return inst.clone();
                }
                let inst = (self.provider)(container);
                *guard = Some(inst.clone());
                inst
            }

            // non-thread-safe branch
            #[cfg(not(feature = "thread-safe"))]
            {
                let mut borrow = self.instance.borrow_mut();
                if let Some(inst) = borrow.as_ref() {
                    return inst.clone();
                }
                let inst = (self.provider)(container);
                *borrow = Some(inst.clone());
                inst
            }
        } else {
            (self.provider)(container)
        }
    }

    /// Provide an instance asynchronously (only available with the `async` feature).
    ///
    /// - If singleton, returns the cached instance or creates and caches it.
    /// - If transient, always calls the async provider.
    #[cfg(feature = "async")]
    pub async fn provide_async(&self, container: Arc<Container>) -> Shared<T> {
        let provider = self
            .async_provider
            .as_ref()
            .expect("provide_async called on non-async factory");

        if self.singleton {
            let cache = self
                .async_instance
                .as_ref()
                .expect("async singleton must have async_instance");
            
            let mut guard = cache.lock().await;
            if let Some(inst) = guard.as_ref() {
                return inst.clone();
            }
            let inst = provider(container).await;
            *guard = Some(inst.clone());
            inst
        } else {
            provider(container).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Counter;
    impl Counter {}

    #[test]
    fn singleton_factory_gives_same_instance() {
        let provider = Box::new(|_c: &Container| Shared::new(Counter));
        let f = Factory::new(provider, true);
        let c = Container::new();
        let a = f.provide(&c);
        let b = f.provide(&c);
        // Ensure these are the same instance (pointer equality)
        assert!(Shared::ptr_eq(&a, &b));
    }
}
