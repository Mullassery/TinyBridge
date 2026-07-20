use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Result, DnsError};
use crate::registry::DnsEntry;

/// mDNS responder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponderConfig {
    /// Enable mDNS announcements
    pub enabled: bool,

    /// Interface to announce on (e.g., "en0")
    pub interface: Option<String>,

    /// TTL for mDNS records (seconds)
    pub ttl: u32,

    /// Announcement interval (seconds)
    pub announcement_interval: u32,
}

impl Default for ResponderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interface: None,
            ttl: 4500, // 75 minutes
            announcement_interval: 300, // 5 minutes
        }
    }
}

/// mDNS responder status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponderStatus {
    /// Responder is running
    Running,
    /// Responder is stopped
    Stopped,
    /// Responder failed
    Failed,
}

/// mDNS responder for announcing .local domains
pub struct MdnsResponder {
    config: ResponderConfig,
    status: ResponderStatus,
    started_at: Option<DateTime<Utc>>,
    announced_entries: std::collections::HashSet<Uuid>,
}

impl MdnsResponder {
    /// Create a new mDNS responder
    pub fn new(config: ResponderConfig) -> Self {
        Self {
            config,
            status: ResponderStatus::Stopped,
            started_at: None,
            announced_entries: std::collections::HashSet::new(),
        }
    }

    /// Start the responder
    pub fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Err(DnsError::ResponderError("mDNS is disabled".to_string()));
        }

        if self.status == ResponderStatus::Running {
            return Err(DnsError::ResponderError("Responder already running".to_string()));
        }

        self.status = ResponderStatus::Running;
        self.started_at = Some(Utc::now());

        Ok(())
    }

    /// Stop the responder
    pub fn stop(&mut self) -> Result<()> {
        if self.status == ResponderStatus::Stopped {
            return Err(DnsError::ResponderError("Responder not running".to_string()));
        }

        self.status = ResponderStatus::Stopped;
        self.announced_entries.clear();

        Ok(())
    }

    /// Announce a DNS entry (record that it should be broadcast)
    pub fn announce(&mut self, entry: &DnsEntry) -> Result<()> {
        if self.status != ResponderStatus::Running {
            return Err(DnsError::ResponderError("Responder not running".to_string()));
        }

        if entry.ipv4.is_none() && entry.ipv6.is_none() {
            return Err(DnsError::ResponderError(
                "Entry must have at least one IP address".to_string(),
            ));
        }

        self.announced_entries.insert(entry.env_id);
        Ok(())
    }

    /// Withdraw an announcement
    pub fn withdraw(&mut self, env_id: Uuid) -> Result<()> {
        if self.status != ResponderStatus::Running {
            return Err(DnsError::ResponderError("Responder not running".to_string()));
        }

        self.announced_entries.remove(&env_id);
        Ok(())
    }

    /// Get current status
    pub fn get_status(&self) -> ResponderStatus {
        self.status
    }

    /// Get number of announced entries
    pub fn announced_count(&self) -> usize {
        self.announced_entries.len()
    }

    /// Check if an entry is announced
    pub fn is_announced(&self, env_id: Uuid) -> bool {
        self.announced_entries.contains(&env_id)
    }

    /// Get uptime (if running)
    pub fn uptime_secs(&self) -> Option<i64> {
        self.started_at.map(|start| (Utc::now() - start).num_seconds())
    }
}

impl Default for MdnsResponder {
    fn default() -> Self {
        Self::new(ResponderConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_responder_lifecycle() {
        let mut responder = MdnsResponder::new(ResponderConfig::default());

        assert_eq!(responder.get_status(), ResponderStatus::Stopped);

        responder.start().unwrap();
        assert_eq!(responder.get_status(), ResponderStatus::Running);

        responder.stop().unwrap();
        assert_eq!(responder.get_status(), ResponderStatus::Stopped);
    }

    #[test]
    fn test_double_start_fails() {
        let mut responder = MdnsResponder::new(ResponderConfig::default());

        responder.start().unwrap();
        let result = responder.start();

        assert!(result.is_err());
    }

    #[test]
    fn test_announce_entry() {
        let mut responder = MdnsResponder::new(ResponderConfig::default());
        responder.start().unwrap();

        let env_id = Uuid::new_v4();
        let mut entry = DnsEntry::new(env_id, "myenv".to_string(), "myenv.local".to_string());
        entry.ipv4 = Some("192.168.1.100".to_string());

        responder.announce(&entry).unwrap();
        assert!(responder.is_announced(env_id));
        assert_eq!(responder.announced_count(), 1);
    }

    #[test]
    fn test_announce_requires_ip() {
        let mut responder = MdnsResponder::new(ResponderConfig::default());
        responder.start().unwrap();

        let env_id = Uuid::new_v4();
        let entry = DnsEntry::new(env_id, "myenv".to_string(), "myenv.local".to_string());

        let result = responder.announce(&entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_withdraw_entry() {
        let mut responder = MdnsResponder::new(ResponderConfig::default());
        responder.start().unwrap();

        let env_id = Uuid::new_v4();
        let mut entry = DnsEntry::new(env_id, "myenv".to_string(), "myenv.local".to_string());
        entry.ipv4 = Some("192.168.1.100".to_string());

        responder.announce(&entry).unwrap();
        assert_eq!(responder.announced_count(), 1);

        responder.withdraw(env_id).unwrap();
        assert_eq!(responder.announced_count(), 0);
    }

    #[test]
    fn test_disabled_responder() {
        let config = ResponderConfig {
            enabled: false,
            ..Default::default()
        };
        let mut responder = MdnsResponder::new(config);

        let result = responder.start();
        assert!(result.is_err());
    }
}
