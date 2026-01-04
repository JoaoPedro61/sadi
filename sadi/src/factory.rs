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
                #[cfg(any(feature = "thread-safe", feature = "async"))]
                {
                    std::sync::Mutex::new(None)
                }
                #[cfg(all(not(feature = "thread-safe"), not(feature = "async")))]
                {
                    std::cell::RefCell::new(None)
                }
            },
        }
    }

    /// Provide an instance from the factory.
    ///
    /// - If singleton, returns the cached instance or creates and caches it.
    /// - If transient, creates a new instance each time.
    #[cfg(not(feature = "async"))]
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

    #[cfg(feature = "async")]
    pub async fn provide(&self, container: &Container) -> Shared<T> {
        if self.singleton {
            // thread-safe branch
            #[cfg(feature = "thread-safe")]
            {
                let mut guard = self.instance.lock().unwrap();
                if let Some(inst) = guard.as_ref() {
                    return inst.clone();
                }
                let inst = (self.provider)(container).await;
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
                let inst = (self.provider)(container).await;
                *borrow = Some(inst.clone());
                inst
            }
        } else {
            (self.provider)(container).await
        }
    }
}


#[cfg(test)]
mod tests {

    struct Counter;
    impl Counter {}

    #[cfg(not(feature = "async"))]
    #[cfg(test)]
    mod sync_tests {
        use super::super::*;
        use super::*;

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

    #[cfg(feature = "async")]
    #[cfg(test)]
    mod async_tests {
        use super::super::*;
        use super::*;
        use futures::executor::block_on;

        #[test]
        fn singleton_factory_gives_same_instance() {
            let provider: Provider<Counter> = Box::new(|_container| {
                Box::pin(async move {
                    // async logic here
                    Shared::new(Counter)
                })
            });
            let f = Factory::new(provider, true);
            let c = Container::new();
            let a = block_on(f.provide(&c));
            let b = block_on(f.provide(&c));
            // Ensure these are the same instance (pointer equality)
            assert!(Shared::ptr_eq(&a, &b));
        }
    }
}
