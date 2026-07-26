use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl ErrorSeverity {
    pub fn symbol(&self) -> &'static str {
        match self {
            ErrorSeverity::Info => "ℹ️",
            ErrorSeverity::Warning => "⚠️",
            ErrorSeverity::Error => "❌",
            ErrorSeverity::Critical => "🚨",
        }
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(self, ErrorSeverity::Info | ErrorSeverity::Warning)
    }

    pub fn should_exit(&self) -> bool {
        matches!(self, ErrorSeverity::Critical)
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_symbols() {
        assert_eq!(ErrorSeverity::Info.symbol(), "ℹ️");
        assert_eq!(ErrorSeverity::Warning.symbol(), "⚠️");
        assert_eq!(ErrorSeverity::Error.symbol(), "❌");
        assert_eq!(ErrorSeverity::Critical.symbol(), "🚨");
    }

    #[test]
    fn test_severity_recoverable() {
        assert!(ErrorSeverity::Info.is_recoverable());
        assert!(ErrorSeverity::Warning.is_recoverable());
        assert!(!ErrorSeverity::Error.is_recoverable());
        assert!(!ErrorSeverity::Critical.is_recoverable());
    }

    #[test]
    fn test_severity_should_exit() {
        assert!(!ErrorSeverity::Info.should_exit());
        assert!(!ErrorSeverity::Warning.should_exit());
        assert!(!ErrorSeverity::Error.should_exit());
        assert!(ErrorSeverity::Critical.should_exit());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(ErrorSeverity::Info.to_string(), "INFO");
        assert_eq!(ErrorSeverity::Critical.to_string(), "CRITICAL");
    }
}
