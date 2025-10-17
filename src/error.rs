use std::fmt;

/// Error kinds for SaDi dependency injection operations
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Service factory not registered
    ServiceNotRegistered,
    /// Factory returned wrong type
    TypeMismatch,
    /// Cached instance has wrong type
    CachedTypeMismatch,
    /// Factory already registered
    FactoryAlreadyRegistered,
}

/// Error structure for SaDi operations
#[derive(Debug, Clone)]
pub struct SaDiError {
    /// The kind of error that occurred
    pub kind: ErrorKind,
    /// Human-readable error message
    pub message: String,
}

impl SaDiError {
    /// Create a new SaDiError
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Create a service not registered error
    pub fn service_not_registered(type_name: &str, service_type: &str) -> Self {
        Self::new(
            ErrorKind::ServiceNotRegistered,
            format!(
                "No {} factory registered for type: {}",
                service_type, type_name
            ),
        )
    }

    /// Create a type mismatch error
    pub fn type_mismatch(type_name: &str) -> Self {
        Self::new(
            ErrorKind::TypeMismatch,
            format!("Factory returned wrong type for: {}", type_name),
        )
    }

    /// Create a cached type mismatch error
    pub fn cached_type_mismatch(type_name: &str) -> Self {
        Self::new(
            ErrorKind::CachedTypeMismatch,
            format!("Cached instance has wrong type for: {}", type_name),
        )
    }

    /// Create a factory already registered error
    pub fn factory_already_registered(type_name: &str, service_type: &str) -> Self {
        Self::new(
            ErrorKind::FactoryAlreadyRegistered,
            format!(
                "{} factory already registered for type: {}",
                service_type, type_name
            ),
        )
    }
}

impl fmt::Display for SaDiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SaDiError {}
