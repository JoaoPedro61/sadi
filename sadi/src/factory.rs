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
//! let f = Factory::new(provider, false); // transient
//! let c = Container::new();
//!
//! let a = f.provide(&c).unwrap();
//! let b = f.provide(&c).unwrap();
//!
//! assert!(!Shared::ptr_eq(&a, &b)); // different instances
//! ```
//!
//! For singleton, use `true` for the second argument and verify the same instance is reused.

use crate::{Container, Error, InstanceCell, Limits, Policy, Provider, ResourceLimiter, Shared};

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
    limiter: Option<ResourceLimiter>,
    policy: Option<Policy>,
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
            limiter: None,
            policy: None,
        }
    }

    /// Create a new factory with resource limits.
    ///
    /// - `provider`: closure that produces a `Shared<T>`
    /// - `singleton`: if true, cache and reuse the instance
    /// - `limits`: concurrency limits for service creation
    pub fn with_limits(provider: Provider<T>, singleton: bool, limits: Limits) -> Result<Self, Error> {
        let limiter = ResourceLimiter::new(
            limits.max_concurrent_creations.unwrap_or(1),
        );
        Ok(Self {
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
            limiter: Some(limiter),
            policy: Some(limits.policy),
        })
    }

    /// Provide an instance of the service.
    ///
    /// - If limits are enabled, attempts to acquire a slot before creating/returning the instance.
    /// - If singleton, returns the cached instance or creates and caches it.
    /// - If transient, always calls the provider.
    pub fn provide(&self, container: &Container) -> Result<Shared<T>, Error> {
        // Try to acquire a slot if limits are configured
        let _guard = if let Some(limiter) = &self.limiter {
            match self.policy {
                Some(Policy::Deny) => {
                    Some(limiter.try_acquire()
                        .ok_or_else(|| Error::resource_limit_exceeded(
                            std::any::type_name::<T>(),
                            "limit exceeded (deny policy)",
                        ))?)
                }
                Some(Policy::Block) => {
                    Some(limiter.acquire_blocking())
                }
                None => {
                    return Err(Error::resource_limit_exceeded(
                        std::any::type_name::<T>(),
                        "invalid policy",
                    ));
                }
            }
        } else {
            None
        };

        if self.singleton {
            // thread-safe branch
            #[cfg(feature = "thread-safe")]
            {
                let mut guard = self.instance.lock().unwrap();
                if let Some(inst) = guard.as_ref() {
                    return Ok(inst.clone());
                }
                let inst = (self.provider)(container);
                *guard = Some(inst.clone());
                Ok(inst)
            }

            // non-thread-safe branch
            #[cfg(not(feature = "thread-safe"))]
            {
                let mut borrow = self.instance.borrow_mut();
                if let Some(inst) = borrow.as_ref() {
                    return Ok(inst.clone());
                }
                let inst = (self.provider)(container);
                *borrow = Some(inst.clone());
                Ok(inst)
            }
        } else {
            Ok((self.provider)(container))
        }
    }

    /// Estimate the memory usage of the factory.
    ///
    /// This includes the size of the instance cell.
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of_val(&self.instance)
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
        let a = f.provide(&c).unwrap();
        let b = f.provide(&c).unwrap();
        // Ensure these are the same instance (pointer equality)
        assert!(Shared::ptr_eq(&a, &b));
    }
}
