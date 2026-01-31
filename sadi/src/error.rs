//! Error types for the SADI dependency injection container.
//!
//! This module defines a lightweight error model used across the container to
//! describe failures that can occur during service registration, resolution,
//! scope handling, and module initialization.
//!
//! # Design
//!
//! - `ErrorKind` captures the error category.
//! - `Error` stores the category and a human-readable message.
//!
//! The helpers in `Error` are provided to keep call sites concise and to
//! maintain consistent error messages.
//!
//! # Feature Flags
//!
//! - `tracing`: logs errors when they are created.
//! - `debug`: enables extra diagnostic formatting in `Display`.
//!
//! # Examples
//!
//! ```
//! use sadi::error::Error;
//!
//! let err = Error::service_not_provided("MyService");
//! assert!(err.message.contains("MyService"));
//! ```

use core::fmt;

#[cfg(feature = "tracing")]
use tracing::error;


/// Error categories for the container.
///
/// These variants are intentionally coarse-grained to keep error handling
/// straightforward while still expressive enough for diagnostics.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ErrorKind {
    /// Service provider not found for the requested type.
    ServiceNotProvided,
    /// Type mismatch during downcast or resolution.
    TypeMismatch,
    /// Factory closure already registered for this type.
    ProviderAlreadyRegistered,
    /// Circular dependency detected in resolution chain.
    CircularDependency,
    /// Factory execution failed (panic or invalid state).
    FactoryExecutionFailed,
    /// Invalid scope transition or configuration.
    InvalidScope,
    /// Module initialization or loading failed.
    ModuleLoadFailed,
}

/// Container error structure.
///
/// `kind` enables programmatic handling, while `message` is human-readable.
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    /// Creates a new error with the given kind and message.
    ///
    /// If the `tracing` feature is enabled, the error is automatically logged.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        let error = Self {
            kind: kind.clone(),
            message: message.into(),
        };

        #[cfg(feature = "tracing")]
        error!("{}", error);

        error
    }

    /// Service provider not found for the requested type.
    pub fn service_not_provided(type_name: &str) -> Self {
        Self::new(
            ErrorKind::ServiceNotProvided,
            format!("No provider registered for type: {}", type_name),
        )
    }

    /// Type mismatch during downcast or factory execution.
    ///
    /// This covers both immediate type mismatches and cached instance type mismatches.
    pub fn type_mismatch(type_name: &str) -> Self {
        Self::new(
            ErrorKind::TypeMismatch,
            format!("Type mismatch when resolving: {}", type_name),
        )
    }

    /// Provider already registered for this type.
    ///
    /// Attempting to register a provider for a type that already has one.
    pub fn provider_already_registered(type_name: &str, scope: &str) -> Self {
        Self::new(
            ErrorKind::ProviderAlreadyRegistered,
            format!(
                "Provider ({} scope) already registered for type: {}",
                scope, type_name
            ),
        )
    }

    /// Circular dependency detected in resolution chain.
    pub fn circular_dependency(dependency_chain: &[&str]) -> Self {
        Self::new(
            ErrorKind::CircularDependency,
            format!(
                "Circular dependency detected: {}",
                dependency_chain.join(" -> ")
            ),
        )
    }

    /// Factory closure execution failed.
    pub fn factory_execution_failed(type_name: &str, reason: &str) -> Self {
        Self::new(
            ErrorKind::FactoryExecutionFailed,
            format!("Factory execution failed for {}: {}", type_name, reason),
        )
    }

    /// Invalid scope or scope transition.
    pub fn invalid_scope(reason: &str) -> Self {
        Self::new(
            ErrorKind::InvalidScope,
            format!("Invalid scope: {}", reason),
        )
    }

    /// Module initialization or loading failed.
    pub fn module_load_failed(module_name: &str, reason: &str) -> Self {
        Self::new(
            ErrorKind::ModuleLoadFailed,
            format!("Module '{}' failed to load: {}", module_name, reason),
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "debug")]
        {
            write!(f, "({:?}) - {}", self.kind, self.message)
        }
        #[cfg(not(feature = "debug"))]
        {
            write!(f, "{}", self.message)
        }
    }
}

#[cfg(feature = "debug")]
impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_not_provided_error() {
        let err = Error::service_not_provided("MyType");
        assert_eq!(err.kind == ErrorKind::ServiceNotProvided, true);
        assert!(err.message.contains("MyType"));
        assert!(err.message.contains("provider"));
    }

    #[test]
    fn type_mismatch_error() {
        let err = Error::type_mismatch("OtherType");
        assert_eq!(err.kind == ErrorKind::TypeMismatch, true);
        assert!(err.message.contains("OtherType"));
    }

    #[test]
    fn provider_already_registered_error() {
        let err = Error::provider_already_registered("Foo", "transient");
        assert_eq!(err.kind == ErrorKind::ProviderAlreadyRegistered, true);
        assert!(err.message.contains("Foo"));
        assert!(err.message.contains("transient"));
    }

    #[test]
    fn circular_dependency_error() {
        let chain = ["A", "B", "A"];
        let err = Error::circular_dependency(&chain);
        assert_eq!(err.kind == ErrorKind::CircularDependency, true);
        assert!(err.message.contains("A -> B -> A"));
    }

    #[test]
    fn factory_execution_failed_error() {
        let err = Error::factory_execution_failed("ServiceX", "out of memory");
        assert_eq!(err.kind == ErrorKind::FactoryExecutionFailed, true);
        assert!(err.message.contains("ServiceX"));
        assert!(err.message.contains("out of memory"));
    }

    #[test]
    fn invalid_scope_error() {
        let err = Error::invalid_scope("Unknown scope type");
        assert_eq!(err.kind == ErrorKind::InvalidScope, true);
        assert!(err.message.contains("Unknown scope type"));
    }

    #[test]
    fn module_load_failed_error() {
        let err = Error::module_load_failed("AuthModule", "missing config");
        assert_eq!(err.kind == ErrorKind::ModuleLoadFailed, true);
        assert!(err.message.contains("AuthModule"));
        assert!(err.message.contains("missing config"));
    }

    #[test]
    fn display_trait() {
        let err = Error::service_not_provided("X");
        let s = format!("{}", err);
        #[cfg(feature = "debug")]
        assert!(s.contains("ServiceNotProvided"));
        assert!(s.contains("X"));
    }

    #[test]
    fn error_kind_equality() {
        let err1 = Error::type_mismatch("A");
        let err2 = Error::type_mismatch("B");
        assert_eq!(err1.kind == err2.kind, true);
        assert_ne!(err1.message, err2.message);
    }
}
