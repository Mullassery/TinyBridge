use crate::{ErrorContext, ErrorSeverity, RecoverySuggestion};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    Bootstrap,
    Vm,
    Cli,
    Network,
    Storage,
    Permission,
    Configuration,
    Unknown,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Bootstrap => write!(f, "Bootstrap Error"),
            ErrorKind::Vm => write!(f, "VM Error"),
            ErrorKind::Cli => write!(f, "CLI Error"),
            ErrorKind::Network => write!(f, "Network Error"),
            ErrorKind::Storage => write!(f, "Storage Error"),
            ErrorKind::Permission => write!(f, "Permission Error"),
            ErrorKind::Configuration => write!(f, "Configuration Error"),
            ErrorKind::Unknown => write!(f, "Unknown Error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeError {
    pub kind: ErrorKind,
    pub message: String,
    pub severity: ErrorSeverity,
    pub context: Option<ErrorContext>,
    pub suggestion: Option<RecoverySuggestion>,
}

impl BridgeError {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        BridgeError {
            kind,
            message,
            severity: ErrorSeverity::Error,
            context: None,
            suggestion: None,
        }
    }

    pub fn bootstrap(message: String) -> Self {
        Self::new(ErrorKind::Bootstrap, message)
    }

    pub fn vm(message: String) -> Self {
        Self::new(ErrorKind::Vm, message)
    }

    pub fn cli(message: String) -> Self {
        Self::new(ErrorKind::Cli, message)
    }

    pub fn network(message: String) -> Self {
        Self::new(ErrorKind::Network, message)
    }

    pub fn storage(message: String) -> Self {
        Self::new(ErrorKind::Storage, message)
    }

    pub fn permission(message: String) -> Self {
        Self::new(ErrorKind::Permission, message)
    }

    pub fn configuration(message: String) -> Self {
        Self::new(ErrorKind::Configuration, message)
    }

    pub fn unknown(message: String) -> Self {
        Self::new(ErrorKind::Unknown, message)
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_suggestion(mut self, suggestion: RecoverySuggestion) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn user_message(&self) -> String {
        let mut msg = format!("{}: {}", self.kind, self.message);

        if let Some(ctx) = &self.context {
            msg.push_str(&format!("\nContext: {}", ctx));
        }

        if let Some(sugg) = &self.suggestion {
            msg.push_str(&format!("\n\n💡 Suggestion: {}", sugg));
        }

        msg
    }
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for BridgeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_error() {
        let err = BridgeError::bootstrap("Failed to initialize".to_string());
        assert_eq!(err.kind, ErrorKind::Bootstrap);
        assert_eq!(err.severity, ErrorSeverity::Error);
    }

    #[test]
    fn test_error_kind_display() {
        assert_eq!(ErrorKind::Bootstrap.to_string(), "Bootstrap Error");
        assert_eq!(ErrorKind::Vm.to_string(), "VM Error");
        assert_eq!(ErrorKind::Network.to_string(), "Network Error");
    }

    #[test]
    fn test_user_message() {
        let err = BridgeError::vm("Memory limit exceeded".to_string())
            .with_suggestion(RecoverySuggestion::increase_memory(4096));
        let msg = err.user_message();
        assert!(msg.contains("Memory limit exceeded"));
        assert!(msg.contains("Suggestion"));
    }

    #[test]
    fn test_error_chain() {
        let err = BridgeError::vm("Initialization failed".to_string())
            .with_severity(ErrorSeverity::Critical)
            .with_context(ErrorContext::vm_state("running"))
            .with_suggestion(RecoverySuggestion::restart_daemon());
        assert_eq!(err.severity, ErrorSeverity::Critical);
        assert!(err.context.is_some());
        assert!(err.suggestion.is_some());
    }
}
