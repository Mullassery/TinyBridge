use crate::error::{Result, SshError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Type of SSH audit event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    KeyCreated,
    KeyRotated,
    KeyDeleted,
    PublicKeyInjected,
    SshConfigCreated,
    SshConfigUpdated,
    SshConfigDeleted,
    ConnectionAttempt,
    AuthenticationFailed,
    IpAddressChanged,
    EnvironmentDeleted,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::KeyCreated => write!(f, "KeyCreated"),
            AuditEventType::KeyRotated => write!(f, "KeyRotated"),
            AuditEventType::KeyDeleted => write!(f, "KeyDeleted"),
            AuditEventType::PublicKeyInjected => write!(f, "PublicKeyInjected"),
            AuditEventType::SshConfigCreated => write!(f, "SshConfigCreated"),
            AuditEventType::SshConfigUpdated => write!(f, "SshConfigUpdated"),
            AuditEventType::SshConfigDeleted => write!(f, "SshConfigDeleted"),
            AuditEventType::ConnectionAttempt => write!(f, "ConnectionAttempt"),
            AuditEventType::AuthenticationFailed => write!(f, "AuthenticationFailed"),
            AuditEventType::IpAddressChanged => write!(f, "IpAddressChanged"),
            AuditEventType::EnvironmentDeleted => write!(f, "EnvironmentDeleted"),
        }
    }
}

/// SSH audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub env_id: Option<Uuid>,
    pub env_name: Option<String>,
    pub user: String,
    pub success: bool,
    pub details: String,
}

impl AuditEvent {
    /// Format event as JSON line
    pub fn to_json_line(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| SshError::SerializationError(e))
    }

    /// Parse from JSON line
    pub fn from_json_line(line: &str) -> Result<Self> {
        serde_json::from_str(line).map_err(|e| SshError::SerializationError(e))
    }

    /// Format event as human-readable log line
    pub fn to_log_line(&self) -> String {
        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S");
        let status = if self.success { "✓" } else { "✗" };
        let env_info = if let Some(ref name) = self.env_name {
            format!(" [{}]", name)
        } else {
            String::new()
        };

        format!(
            "[{}] {} {} {}{}: {}",
            timestamp, status, self.event_type, self.user, env_info, self.details
        )
    }
}

/// Manages SSH audit logging
pub struct SshAuditLogger {
    audit_log_path: PathBuf,
}

impl SshAuditLogger {
    /// Create a new audit logger
    pub fn new(audit_log_path: impl AsRef<Path>) -> Self {
        Self {
            audit_log_path: audit_log_path.as_ref().to_path_buf(),
        }
    }

    /// Create logger with default path (~/.tinybridge/ssh/audit.log)
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| SshError::InvalidEnvironment("HOME directory not found".to_string()))?;
        let audit_log_path = home.join(".tinybridge/ssh/audit.log");
        Ok(Self::new(audit_log_path))
    }

    /// Log an event
    pub fn log_event(&self, event: &AuditEvent) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.audit_log_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| SshError::AuditError(e.to_string()))?;
        }

        // Append to log file (JSON lines format)
        let json_line = event.to_json_line()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_log_path)
            .map_err(|e| SshError::AuditError(e.to_string()))?;

        writeln!(file, "{}", json_line).map_err(|e| SshError::AuditError(e.to_string()))?;

        Ok(())
    }

    /// Read all audit events
    pub fn read_events(&self) -> Result<Vec<AuditEvent>> {
        if !self.audit_log_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.audit_log_path)
            .map_err(|e| SshError::AuditError(e.to_string()))?;

        let mut events = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match AuditEvent::from_json_line(line) {
                Ok(event) => events.push(event),
                Err(_) => {
                    // Skip malformed lines
                    continue;
                }
            }
        }

        Ok(events)
    }

    /// Query events by environment
    pub fn events_for_env(&self, env_id: Uuid) -> Result<Vec<AuditEvent>> {
        let events = self.read_events()?;
        Ok(events
            .into_iter()
            .filter(|e| e.env_id == Some(env_id))
            .collect())
    }

    /// Query events by type
    pub fn events_by_type(&self, event_type: AuditEventType) -> Result<Vec<AuditEvent>> {
        let events = self.read_events()?;
        Ok(events
            .into_iter()
            .filter(|e| e.event_type == event_type)
            .collect())
    }

    /// Get events in a time range
    pub fn events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditEvent>> {
        let events = self.read_events()?;
        Ok(events
            .into_iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect())
    }

    /// Get recent events
    pub fn recent_events(&self, limit: usize) -> Result<Vec<AuditEvent>> {
        let mut events = self.read_events()?;
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(events.into_iter().take(limit).collect())
    }

    /// Clear audit log (archive existing and start fresh)
    pub fn clear(&self) -> Result<()> {
        if self.audit_log_path.exists() {
            let archive_path = self.audit_log_path.with_extension("log.old");
            std::fs::rename(&self.audit_log_path, &archive_path)
                .map_err(|e| SshError::AuditError(e.to_string()))?;
        }

        Ok(())
    }

    /// Export events as CSV
    pub fn export_csv(&self) -> Result<String> {
        let events = self.read_events()?;

        let mut csv = String::from("timestamp,event_type,env_id,env_name,user,success,details\n");

        for event in events {
            let env_id = event.env_id.map(|id| id.to_string()).unwrap_or_default();
            let env_name = event.env_name.unwrap_or_default();
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                event.timestamp.to_rfc3339(),
                event.event_type,
                env_id,
                env_name,
                event.user,
                if event.success { "true" } else { "false" },
                event.details
            ));
        }

        Ok(csv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_event_formatting() {
        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: AuditEventType::KeyCreated,
            env_id: Some(Uuid::new_v4()),
            env_name: Some("myvm".to_string()),
            user: "testuser".to_string(),
            success: true,
            details: "Ed25519 key generated".to_string(),
        };

        let log_line = event.to_log_line();
        assert!(log_line.contains("KeyCreated"));
        assert!(log_line.contains("testuser"));
        assert!(log_line.contains("✓"));
    }

    #[test]
    fn test_audit_logging() {
        let temp_dir = TempDir::new().unwrap();
        let audit_path = temp_dir.path().join("audit.log");
        let logger = SshAuditLogger::new(&audit_path);

        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: AuditEventType::KeyCreated,
            env_id: Some(Uuid::new_v4()),
            env_name: Some("myvm".to_string()),
            user: "testuser".to_string(),
            success: true,
            details: "Test event".to_string(),
        };

        assert!(logger.log_event(&event).is_ok());
        assert!(audit_path.exists());

        let events = logger.read_events().unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_csv_export() {
        let temp_dir = TempDir::new().unwrap();
        let audit_path = temp_dir.path().join("audit.log");
        let logger = SshAuditLogger::new(&audit_path);

        let event = AuditEvent {
            timestamp: Utc::now(),
            event_type: AuditEventType::KeyCreated,
            env_id: Some(Uuid::new_v4()),
            env_name: Some("myvm".to_string()),
            user: "testuser".to_string(),
            success: true,
            details: "Test event".to_string(),
        };

        logger.log_event(&event).unwrap();
        let csv = logger.export_csv().unwrap();
        assert!(csv.contains("KeyCreated"));
        assert!(csv.contains("testuser"));
    }
}
