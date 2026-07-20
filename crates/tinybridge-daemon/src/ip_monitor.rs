use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use uuid::Uuid;

/// IP address tracking for environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddressRecord {
    /// Environment ID
    pub env_id: Uuid,

    /// Environment name
    pub env_name: String,

    /// Last known IP address
    pub ip_address: Option<String>,

    /// SSH config alias
    pub ssh_alias: String,
}

impl IpAddressRecord {
    /// Create a new IP address record
    pub fn new(env_id: Uuid, env_name: String, ssh_alias: String) -> Self {
        Self {
            env_id,
            env_name,
            ip_address: None,
            ssh_alias,
        }
    }

    /// Update IP address and return true if changed
    pub fn update_ip(&mut self, new_ip: Option<String>) -> bool {
        if self.ip_address != new_ip {
            self.ip_address = new_ip;
            return true;
        }
        false
    }

    /// Check if IP is valid
    pub fn is_valid_ip(ip: &str) -> bool {
        IpAddr::from_str(ip).is_ok()
    }
}

/// Monitor for tracking IP address changes
pub struct IpChangeMonitor {
    records: HashMap<Uuid, IpAddressRecord>,
}

impl IpChangeMonitor {
    /// Create a new IP change monitor
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Register an environment for monitoring
    pub fn register(&mut self, env_id: Uuid, env_name: String, ssh_alias: String) {
        self.records.insert(
            env_id,
            IpAddressRecord::new(env_id, env_name, ssh_alias),
        );
    }

    /// Unregister an environment
    pub fn unregister(&mut self, env_id: Uuid) -> Option<IpAddressRecord> {
        self.records.remove(&env_id)
    }

    /// Update IP for an environment and check if changed
    pub fn update_and_check_change(&mut self, env_id: Uuid, new_ip: Option<String>) -> bool {
        if let Some(record) = self.records.get_mut(&env_id) {
            // Validate IP if provided
            if let Some(ref ip) = new_ip {
                if !IpAddressRecord::is_valid_ip(ip) {
                    return false;
                }
            }
            record.update_ip(new_ip)
        } else {
            false
        }
    }

    /// Get current IP for an environment
    pub fn get_ip(&self, env_id: Uuid) -> Option<String> {
        self.records.get(&env_id).and_then(|r| r.ip_address.clone())
    }

    /// Get SSH alias for an environment
    pub fn get_ssh_alias(&self, env_id: Uuid) -> Option<String> {
        self.records.get(&env_id).map(|r| r.ssh_alias.clone())
    }

    /// Get all records
    pub fn list_all(&self) -> Vec<IpAddressRecord> {
        self.records.values().cloned().collect()
    }

    /// Count monitored environments
    pub fn count(&self) -> usize {
        self.records.len()
    }
}

impl Default for IpChangeMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_address_record() {
        let mut record = IpAddressRecord::new(Uuid::new_v4(), "test".to_string(), "myenv".to_string());

        assert_eq!(record.ip_address, None);

        let changed = record.update_ip(Some("192.168.1.100".to_string()));
        assert!(changed);
        assert_eq!(record.ip_address, Some("192.168.1.100".to_string()));

        let changed = record.update_ip(Some("192.168.1.100".to_string()));
        assert!(!changed);
    }

    #[test]
    fn test_ip_validation() {
        assert!(IpAddressRecord::is_valid_ip("192.168.1.100"));
        assert!(IpAddressRecord::is_valid_ip("::1"));
        assert!(!IpAddressRecord::is_valid_ip("invalid"));
        assert!(!IpAddressRecord::is_valid_ip("999.999.999.999"));
    }

    #[test]
    fn test_ip_change_monitor() {
        let mut monitor = IpChangeMonitor::new();
        let env_id = Uuid::new_v4();

        monitor.register(env_id, "myenv".to_string(), "myenv-alias".to_string());
        assert_eq!(monitor.count(), 1);

        let changed = monitor.update_and_check_change(env_id, Some("192.168.1.100".to_string()));
        assert!(changed);

        let ip = monitor.get_ip(env_id).unwrap();
        assert_eq!(ip, "192.168.1.100");

        let changed = monitor.update_and_check_change(env_id, Some("192.168.1.100".to_string()));
        assert!(!changed);

        let changed = monitor.update_and_check_change(env_id, Some("192.168.1.101".to_string()));
        assert!(changed);
    }

    #[test]
    fn test_unregister() {
        let mut monitor = IpChangeMonitor::new();
        let env_id = Uuid::new_v4();

        monitor.register(env_id, "myenv".to_string(), "myenv-alias".to_string());
        assert_eq!(monitor.count(), 1);

        let record = monitor.unregister(env_id).unwrap();
        assert_eq!(record.env_name, "myenv");
        assert_eq!(monitor.count(), 0);
    }

    #[test]
    fn test_invalid_ip_ignored() {
        let mut monitor = IpChangeMonitor::new();
        let env_id = Uuid::new_v4();

        monitor.register(env_id, "myenv".to_string(), "myenv-alias".to_string());
        let changed = monitor.update_and_check_change(env_id, Some("invalid-ip".to_string()));

        assert!(!changed);
        assert_eq!(monitor.get_ip(env_id), None);
    }
}
