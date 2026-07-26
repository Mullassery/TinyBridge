pub mod context;
pub mod error;
pub mod severity;
pub mod suggestion;

pub use context::ErrorContext;
pub use error::{BridgeError, ErrorKind};
pub use severity::ErrorSeverity;
pub use suggestion::RecoverySuggestion;

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
