use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySuggestion {
    pub steps: Vec<String>,
    pub docs_link: Option<String>,
}

impl RecoverySuggestion {
    pub fn new(steps: Vec<String>) -> Self {
        RecoverySuggestion {
            steps,
            docs_link: None,
        }
    }

    pub fn with_docs_link(mut self, link: String) -> Self {
        self.docs_link = Some(link);
        self
    }

    pub fn restart_daemon() -> Self {
        Self::new(vec![
            "Stop the TinyBridge daemon: tinybridge daemon stop".to_string(),
            "Wait a few seconds".to_string(),
            "Restart the daemon: tinybridge daemon start".to_string(),
        ])
        .with_docs_link("https://github.com/Mullassery/tinybridge/docs/daemon".to_string())
    }

    pub fn increase_memory(required_mb: u32) -> Self {
        Self::new(vec![
            format!("Increase allocated memory to at least {}MB", required_mb),
            "Use: tinybridge config set memory {}".to_string(),
            "Restart the VM: tinybridge restart".to_string(),
        ])
        .with_docs_link("https://github.com/Mullassery/tinybridge/docs/memory".to_string())
    }

    pub fn check_disk_space(required_gb: u32) -> Self {
        Self::new(vec![
            format!("Free up at least {}GB of disk space", required_gb),
            "Run: tinybridge doctor to analyze disk usage".to_string(),
            "Consider moving or removing unused VMs".to_string(),
        ])
    }

    pub fn check_network() -> Self {
        Self::new(vec![
            "Check your internet connection".to_string(),
            "Verify firewall settings".to_string(),
            "Test connectivity: ping 8.8.8.8".to_string(),
            "Run: tinybridge doctor --network".to_string(),
        ])
    }

    pub fn check_permissions(resource: &str) -> Self {
        Self::new(vec![
            format!("Check permissions for: {}", resource),
            "Run: ls -la {}".to_string(),
            "Grant permissions: chmod 755 {}".to_string(),
            "Or use sudo if necessary".to_string(),
        ])
    }

    pub fn invalid_configuration(key: &str) -> Self {
        Self::new(vec![
            format!("Check the configuration key: {}", key),
            "Run: tinybridge config show".to_string(),
            "Reset to defaults: tinybridge config reset".to_string(),
            "See docs: tinybridge config --help".to_string(),
        ])
    }

    pub fn update_software() -> Self {
        Self::new(vec![
            "Update TinyBridge: brew upgrade tinybridge".to_string(),
            "Or: cargo install --upgrade tinybridge".to_string(),
            "Restart the daemon after updating".to_string(),
        ])
    }

    pub fn contact_support() -> Self {
        Self::new(vec![
            "This error requires manual intervention".to_string(),
            "Check the logs: tinybridge logs --follow".to_string(),
            "Report on GitHub: https://github.com/Mullassery/tinybridge/issues".to_string(),
        ])
    }
}

impl fmt::Display for RecoverySuggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, step) in self.steps.iter().enumerate() {
            writeln!(f, "{}. {}", i + 1, step)?;
        }

        if let Some(link) = &self.docs_link {
            write!(f, "\n📖 Learn more: {}", link)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_creation() {
        let sugg = RecoverySuggestion::new(vec!["Step 1".to_string(), "Step 2".to_string()]);
        assert_eq!(sugg.steps.len(), 2);
    }

    #[test]
    fn test_restart_daemon_suggestion() {
        let sugg = RecoverySuggestion::restart_daemon();
        assert_eq!(sugg.steps.len(), 3);
        assert!(sugg.docs_link.is_some());
    }

    #[test]
    fn test_increase_memory_suggestion() {
        let sugg = RecoverySuggestion::increase_memory(4096);
        assert_eq!(sugg.steps.len(), 3);
    }

    #[test]
    fn test_check_network_suggestion() {
        let sugg = RecoverySuggestion::check_network();
        assert!(!sugg.steps.is_empty());
    }

    #[test]
    fn test_suggestion_display() {
        let sugg = RecoverySuggestion::new(vec!["Step 1".to_string()]);
        let display = sugg.to_string();
        assert!(display.contains("1. Step 1"));
    }
}
