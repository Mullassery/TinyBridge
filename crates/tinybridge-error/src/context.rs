use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub details: Vec<(String, String)>,
}

impl ErrorContext {
    pub fn new() -> Self {
        ErrorContext {
            details: Vec::new(),
        }
    }

    pub fn with_detail(mut self, key: String, value: String) -> Self {
        self.details.push((key, value));
        self
    }

    pub fn memory_allocation(requested_mb: u32) -> Self {
        Self::new().with_detail(
            "Requested Memory".to_string(),
            format!("{}MB", requested_mb),
        )
    }

    pub fn disk_space(available_gb: u32, required_gb: u32) -> Self {
        Self::new()
            .with_detail("Available Space".to_string(), format!("{}GB", available_gb))
            .with_detail("Required Space".to_string(), format!("{}GB", required_gb))
    }

    pub fn network_error(host: &str, port: u16) -> Self {
        Self::new()
            .with_detail("Host".to_string(), host.to_string())
            .with_detail("Port".to_string(), port.to_string())
    }

    pub fn vm_state(state: &str) -> Self {
        Self::new().with_detail("VM State".to_string(), state.to_string())
    }

    pub fn permission_denied(resource: &str) -> Self {
        Self::new().with_detail("Resource".to_string(), resource.to_string())
    }

    pub fn configuration_issue(key: &str, value: &str) -> Self {
        Self::new()
            .with_detail("Config Key".to_string(), key.to_string())
            .with_detail("Value".to_string(), value.to_string())
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in &self.details {
            writeln!(f, "{}: {}", key, value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ErrorContext::new();
        assert_eq!(ctx.details.len(), 0);
    }

    #[test]
    fn test_context_with_detail() {
        let ctx = ErrorContext::new()
            .with_detail("key1".to_string(), "value1".to_string())
            .with_detail("key2".to_string(), "value2".to_string());
        assert_eq!(ctx.details.len(), 2);
    }

    #[test]
    fn test_memory_allocation_context() {
        let ctx = ErrorContext::memory_allocation(2048);
        assert_eq!(ctx.details.len(), 1);
        assert_eq!(ctx.details[0].0, "Requested Memory");
        assert_eq!(ctx.details[0].1, "2048MB");
    }

    #[test]
    fn test_disk_space_context() {
        let ctx = ErrorContext::disk_space(10, 20);
        assert_eq!(ctx.details.len(), 2);
    }

    #[test]
    fn test_context_display() {
        let ctx = ErrorContext::new().with_detail("Key".to_string(), "Value".to_string());
        let display = ctx.to_string();
        assert!(display.contains("Key: Value"));
    }
}
