pub mod error;
pub mod context;
pub mod suggestion;
pub mod severity;

pub use error::{BridgeError, ErrorKind};
pub use context::ErrorContext;
pub use suggestion::RecoverySuggestion;
pub use severity::ErrorSeverity;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = BridgeError::bootstrap("VM failed to start".to_string());
        assert_eq!(err.kind, ErrorKind::Bootstrap);
    }

    #[test]
    fn test_error_with_context() {
        let err = BridgeError::vm("Memory allocation failed".to_string())
            .with_context(ErrorContext::memory_allocation(1024));
        assert_eq!(err.kind, ErrorKind::Vm);
        assert!(err.context.is_some());
    }
}
