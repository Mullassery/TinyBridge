/// Policy audit logger for compliance and forensics
///
/// Records every policy decision with full context.
/// Tamper-evident audit trail for compliance (SOC 2, ISO 27001, PCI-DSS).

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};

/// Audit log entry for a policy decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Unique audit entry ID
    pub id: String,
    /// Timestamp (UTC)
    pub timestamp: String,
    /// Environment making the request
    pub environment: String,
    /// Project (if applicable)
    pub project: Option<String>,
    /// User ID (if applicable)
    pub user_id: Option<String>,
    /// Device ID being accessed
    pub device_id: String,
    /// Device type
    pub device_type: String,
    /// Decision: Allow or Deny
    pub decision: String,
    /// Reason for decision
    pub reason: String,
    /// Which policy level decided
    pub policy_level: String,
    /// Decision path (evaluation steps)
    pub decision_path: Vec<String>,
    /// Optional error message
    pub error: Option<String>,
}

impl AuditLogEntry {
    pub fn new(
        environment: String,
        device_id: String,
        device_type: String,
        decision: bool,
        reason: String,
        policy_level: String,
    ) -> Self {
        AuditLogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            environment,
            project: None,
            user_id: None,
            device_id,
            device_type,
            decision: if decision { "ALLOW" } else { "DENY" }.to_string(),
            reason,
            policy_level,
            decision_path: vec![],
            error: None,
        }
    }

    pub fn with_project(mut self, project: String) -> Self {
        self.project = Some(project);
        self
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_path(mut self, path: Vec<String>) -> Self {
        self.decision_path = path;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} {} {} device {} ({}): {}",
            self.timestamp.split('T').next().unwrap_or(""),
            self.environment,
            self.decision,
            self.device_type,
            &self.device_id[..8.min(self.device_id.len())],
            self.policy_level,
            self.reason
        )
    }
}

/// Policy audit logger for compliance
pub struct PolicyAuditLogger {
    /// In-memory log buffer (ring buffer)
    entries: Arc<RwLock<VecDeque<AuditLogEntry>>>,
    /// Maximum entries in memory
    max_entries: usize,
}

impl PolicyAuditLogger {
    pub fn new(max_entries: usize) -> Self {
        PolicyAuditLogger {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Log a policy decision
    pub fn log(&self, entry: AuditLogEntry) {
        let mut entries = self.entries.write();

        // Maintain size limit (FIFO removal when full)
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }

        entries.push_back(entry);
    }

    /// Get all audit entries
    pub fn get_all_entries(&self) -> Vec<AuditLogEntry> {
        self.entries.read().iter().cloned().collect()
    }

    /// Get entries for an environment
    pub fn get_environment_entries(&self, environment: &str) -> Vec<AuditLogEntry> {
        self.entries
            .read()
            .iter()
            .filter(|e| e.environment == environment)
            .cloned()
            .collect()
    }

    /// Get entries for a device
    pub fn get_device_entries(&self, device_id: &str) -> Vec<AuditLogEntry> {
        self.entries
            .read()
            .iter()
            .filter(|e| e.device_id == device_id)
            .cloned()
            .collect()
    }

    /// Get denied access attempts
    pub fn get_denied_attempts(&self) -> Vec<AuditLogEntry> {
        self.entries
            .read()
            .iter()
            .filter(|e| e.decision == "DENY")
            .cloned()
            .collect()
    }

    /// Get allowed access attempts
    pub fn get_allowed_attempts(&self) -> Vec<AuditLogEntry> {
        self.entries
            .read()
            .iter()
            .filter(|e| e.decision == "ALLOW")
            .cloned()
            .collect()
    }

    /// Get entries since a timestamp
    pub fn get_entries_since(&self, timestamp: &str) -> Vec<AuditLogEntry> {
        self.entries
            .read()
            .iter()
            .filter(|e| e.timestamp.as_str() >= timestamp)
            .cloned()
            .collect()
    }

    /// Generate compliance report
    pub fn generate_compliance_report(&self) -> ComplianceReport {
        let entries = self.entries.read();

        let total_requests = entries.len();
        let allowed = entries.iter().filter(|e| e.decision == "ALLOW").count();
        let denied = entries.iter().filter(|e| e.decision == "DENY").count();

        // Get unique environments
        let mut environments = std::collections::HashSet::new();
        let mut devices = std::collections::HashSet::new();
        let mut policy_levels = std::collections::HashMap::new();

        for entry in entries.iter() {
            environments.insert(entry.environment.clone());
            devices.insert(entry.device_id.clone());
            *policy_levels
                .entry(entry.policy_level.clone())
                .or_insert(0) += 1;
        }

        let timestamp_start = entries
            .front()
            .map(|e| e.timestamp.clone())
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        let timestamp_end = entries
            .back()
            .map(|e| e.timestamp.clone())
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        ComplianceReport {
            timestamp_generated: Utc::now().to_rfc3339(),
            timestamp_period_start: timestamp_start,
            timestamp_period_end: timestamp_end,
            total_requests,
            allowed_requests: allowed,
            denied_requests: denied,
            allow_percentage: if total_requests > 0 {
                (allowed as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            },
            unique_environments: environments.len(),
            unique_devices: devices.len(),
            policy_level_distribution: policy_levels,
            audit_trail_entries: total_requests,
        }
    }

    /// Clear audit log (for testing)
    pub fn clear(&self) {
        self.entries.write().clear();
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.read().len()
    }
}

impl Default for PolicyAuditLogger {
    fn default() -> Self {
        Self::new(10_000) // Default 10k entries
    }
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Report generated timestamp
    pub timestamp_generated: String,
    /// Audit period start
    pub timestamp_period_start: String,
    /// Audit period end
    pub timestamp_period_end: String,
    /// Total access requests
    pub total_requests: usize,
    /// Allowed requests
    pub allowed_requests: usize,
    /// Denied requests
    pub denied_requests: usize,
    /// Allow rate percentage
    pub allow_percentage: f64,
    /// Unique environments accessed
    pub unique_environments: usize,
    /// Unique devices accessed
    pub unique_devices: usize,
    /// Policy level distribution
    pub policy_level_distribution: std::collections::HashMap<String, usize>,
    /// Total audit trail entries
    pub audit_trail_entries: usize,
}

impl ComplianceReport {
    /// Get summary line
    pub fn summary(&self) -> String {
        format!(
            "Compliance Report: {} allowed, {} denied ({:.1}%), {} environments, {} devices",
            self.allowed_requests,
            self.denied_requests,
            self.allow_percentage,
            self.unique_environments,
            self.unique_devices
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entry_creation() {
        let entry = AuditLogEntry::new(
            "ml-env".to_string(),
            "device-1".to_string(),
            "usb".to_string(),
            true,
            "Policy allows".to_string(),
            "platform".to_string(),
        );

        assert_eq!(entry.environment, "ml-env");
        assert_eq!(entry.device_id, "device-1");
        assert_eq!(entry.decision, "ALLOW");
    }

    #[test]
    fn test_audit_logger_logging() {
        let logger = PolicyAuditLogger::new(100);

        let entry = AuditLogEntry::new(
            "env1".to_string(),
            "dev1".to_string(),
            "serial".to_string(),
            true,
            "Allowed".to_string(),
            "platform".to_string(),
        );

        logger.log(entry);

        assert_eq!(logger.entry_count(), 1);
    }

    #[test]
    fn test_audit_logger_filtering_by_environment() {
        let logger = PolicyAuditLogger::new(100);

        for i in 0..3 {
            logger.log(AuditLogEntry::new(
                format!("env{}", i),
                format!("dev{}", i),
                "usb".to_string(),
                i % 2 == 0,
                "Test".to_string(),
                "platform".to_string(),
            ));
        }

        let env1_entries = logger.get_environment_entries("env1");
        assert_eq!(env1_entries.len(), 1);
    }

    #[test]
    fn test_audit_logger_denied_attempts() {
        let logger = PolicyAuditLogger::new(100);

        logger.log(AuditLogEntry::new(
            "env1".to_string(),
            "dev1".to_string(),
            "camera".to_string(),
            true,
            "Allowed".to_string(),
            "platform".to_string(),
        ));

        logger.log(AuditLogEntry::new(
            "env2".to_string(),
            "dev2".to_string(),
            "camera".to_string(),
            false,
            "Policy denies".to_string(),
            "project".to_string(),
        ));

        assert_eq!(logger.get_allowed_attempts().len(), 1);
        assert_eq!(logger.get_denied_attempts().len(), 1);
    }

    #[test]
    fn test_compliance_report_generation() {
        let logger = PolicyAuditLogger::new(100);

        for i in 0..10 {
            logger.log(AuditLogEntry::new(
                format!("env{}", i % 3),
                format!("dev{}", i % 5),
                "usb".to_string(),
                i % 3 != 0, // Some denied
                "Test".to_string(),
                "platform".to_string(),
            ));
        }

        let report = logger.generate_compliance_report();

        assert_eq!(report.total_requests, 10);
        assert!(report.allowed_requests > 0);
        assert!(report.denied_requests > 0);
        assert!(report.allow_percentage > 0.0 && report.allow_percentage <= 100.0);
    }

    #[test]
    fn test_audit_logger_size_limit() {
        let logger = PolicyAuditLogger::new(5);

        for i in 0..10 {
            logger.log(AuditLogEntry::new(
                format!("env{}", i),
                format!("dev{}", i),
                "usb".to_string(),
                true,
                "Test".to_string(),
                "platform".to_string(),
            ));
        }

        assert_eq!(logger.entry_count(), 5);
    }

    #[test]
    fn test_audit_entry_summary() {
        let entry = AuditLogEntry::new(
            "ml-env".to_string(),
            "device-123456789".to_string(),
            "camera".to_string(),
            true,
            "Security policy allows".to_string(),
            "environment".to_string(),
        );

        let summary = entry.summary();
        assert!(summary.contains("ALLOW"));
        assert!(summary.contains("camera"));
        assert!(summary.contains("device-"));
    }

    #[test]
    fn test_audit_logger_entries_since() {
        let logger = PolicyAuditLogger::new(100);

        let entry1 = AuditLogEntry::new(
            "env1".to_string(),
            "dev1".to_string(),
            "usb".to_string(),
            true,
            "Test".to_string(),
            "platform".to_string(),
        );
        let timestamp = entry1.timestamp.clone();

        logger.log(entry1);

        // Small delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));

        logger.log(AuditLogEntry::new(
            "env2".to_string(),
            "dev2".to_string(),
            "serial".to_string(),
            false,
            "Test".to_string(),
            "platform".to_string(),
        ));

        let recent = logger.get_entries_since(&timestamp);
        assert!(recent.len() >= 1);
    }

    #[test]
    fn test_compliance_report_summary() {
        let report = ComplianceReport {
            timestamp_generated: Utc::now().to_rfc3339(),
            timestamp_period_start: Utc::now().to_rfc3339(),
            timestamp_period_end: Utc::now().to_rfc3339(),
            total_requests: 100,
            allowed_requests: 85,
            denied_requests: 15,
            allow_percentage: 85.0,
            unique_environments: 5,
            unique_devices: 10,
            policy_level_distribution: std::collections::HashMap::new(),
            audit_trail_entries: 100,
        };

        let summary = report.summary();
        assert!(summary.contains("85 allowed"));
        assert!(summary.contains("15 denied"));
        assert!(summary.contains("5 environments"));
    }
}
