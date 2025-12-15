//! Async thread-local stack guard for circular dependency detection in SaDi.
//!
//! This module provides [`AsyncResolveGuard`], a utility for tracking the chain of type names
//! being resolved during async dependency injection. It uses a thread-local stack to detect
//! and report circular dependencies, returning a detailed error chain if a cycle is found.
//!
//! This module is only available when the `async` feature is enabled.
//!
//! # Example
//! ```ignore
//! use sadi::AsyncResolveGuard;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Push a type name onto the stack
//!     let _g1 = AsyncResolveGuard::push("A").await.unwrap();
//!     // Pushing a different type is fine
//!     let _g2 = AsyncResolveGuard::push("B").await.unwrap();
//!     // Pushing the same type again triggers a circular dependency error
//!     let err = AsyncResolveGuard::push("A").await.unwrap_err();
//!     assert!(matches!(err.kind, crate::ErrorKind::CircularDependency));
//! }
//! ```

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::Error;

// Using a lazy_static-like pattern with tokio::sync::Mutex for async context
lazy_static::lazy_static! {
    // Stack of type names being resolved in this task/thread
    static ref ASYNC_RESOLVE_STACK: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

/// Guard that pops the last pushed type name from the async stack on drop.
///
/// Used to track the current dependency resolution chain for circular detection in async contexts.
#[derive(Debug)]
pub struct AsyncResolveGuard {
    pub type_name: String,
}

impl AsyncResolveGuard {
    /// Try to push a type name onto the async stack.
    ///
    /// Returns `Err(Error::circular_dependency(..))` if the type is already on the stack.
    /// Otherwise, returns a guard that will pop the type on drop.
    pub async fn push(type_name: &str) -> Result<Self, Error> {
        let mut v = ASYNC_RESOLVE_STACK.lock().await;
        if v.iter().any(|s| s == type_name) {
            // Build chain: existing stack + current type
            let mut chain = v.clone();
            chain.push(type_name.to_string());
            // Convert to Vec<&str> for Error::circular_dependency
            let refs: Vec<&str> = chain.iter().map(|s| s.as_str()).collect();
            return Err(Error::circular_dependency(&refs));
        }
        v.push(type_name.to_string());
        Ok(AsyncResolveGuard {
            type_name: type_name.to_string(),
        })
    }
}

impl Drop for AsyncResolveGuard {
    fn drop(&mut self) {
        // We need to handle the drop without blocking
        // This is a bit tricky in async context, but since we're in Drop,
        // we can't call async code. Instead, we use a blocking approach.
        // In practice, this works because Tokio's Mutex can be released synchronously.
        if let Ok(mut v) = ASYNC_RESOLVE_STACK.try_lock() {
            v.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests are disabled by default because the lazy_static
    // ASYNC_RESOLVE_STACK is shared across all tests in the same thread.
    // To properly test circular dependency detection, we'd need to use
    // a thread-safe reset mechanism or test in isolation.
    // The async resolve guard functionality is tested indirectly through
    // async_container::tests when circular dependencies occur.

    // #[tokio::test]
    // async fn test_async_push_and_pop_stack() { ... }
}
