use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{Result, DnsError};

/// DNS entry for an environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsEntry {
    /// Environment ID
    pub env_id: Uuid,

    /// Environment name
    pub env_name: String,

    /// Full FQDN (e.g., "myenv.local")
    pub fqdn: String,

    /// IPv4 address (if available)
    pub ipv4: Option<String>,

    /// IPv6 address (if available)
    pub ipv6: Option<String>,

    /// Registration timestamp
    pub registered_at: DateTime<Utc>,

    /// Whether entry is active
    pub is_active: bool,
}

impl DnsEntry {
    /// Create a new DNS entry
    pub fn new(env_id: Uuid, env_name: String, fqdn: String) -> Self {
        Self {
            env_id,
            env_name,
            fqdn,
            ipv4: None,
            ipv6: None,
            registered_at: Utc::now(),
            is_active: false,
        }
    }

    /// Set IPv4 address
    pub fn with_ipv4(mut self, ipv4: String) -> Self {
        self.ipv4 = Some(ipv4);
        self
    }

    /// Set IPv6 address
    pub fn with_ipv6(mut self, ipv6: String) -> Self {
        self.ipv6 = Some(ipv6);
        self
    }

    /// Activate the entry
    pub fn activate(mut self) -> Self {
        self.is_active = true;
        self
    }
}

/// DNS registry for managing .local domain entries
pub struct DnsRegistry {
    entries: HashMap<Uuid, DnsEntry>,
    fqdn_to_id: HashMap<String, Uuid>,
}

impl DnsRegistry {
    /// Create a new DNS registry
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            fqdn_to_id: HashMap::new(),
        }
    }

    /// Register a new DNS entry
    pub fn register(&mut self, entry: DnsEntry) -> Result<()> {
        let env_id = entry.env_id;
        let fqdn = entry.fqdn.clone();

        // Check for FQDN conflicts
        if self.fqdn_to_id.contains_key(&fqdn) {
            return Err(DnsError::RegistrationError(
                format!("FQDN {} already registered", fqdn),
            ));
        }

        self.fqdn_to_id.insert(fqdn.clone(), env_id);
        self.entries.insert(env_id, entry);

        Ok(())
    }

    /// Get a DNS entry by environment ID
    pub fn get_by_env_id(&self, env_id: Uuid) -> Option<DnsEntry> {
        self.entries.get(&env_id).cloned()
    }

    /// Get a DNS entry by FQDN
    pub fn get_by_fqdn(&self, fqdn: &str) -> Option<DnsEntry> {
        self.fqdn_to_id
            .get(fqdn)
            .and_then(|id| self.entries.get(id).cloned())
    }

    /// Unregister a DNS entry
    pub fn unregister(&mut self, env_id: Uuid) -> Result<DnsEntry> {
        let entry = self
            .entries
            .remove(&env_id)
            .ok_or_else(|| DnsError::RegistrationError(format!("Entry not found for {}", env_id)))?;

        self.fqdn_to_id.remove(&entry.fqdn);
        Ok(entry)
    }

    /// Update IPv4 address for an entry
    pub fn update_ipv4(&mut self, env_id: Uuid, ipv4: String) -> Result<()> {
        self.entries
            .get_mut(&env_id)
            .ok_or_else(|| DnsError::RegistrationError(format!("Entry not found for {}", env_id)))?
            .ipv4 = Some(ipv4);
        Ok(())
    }

    /// Update IPv6 address for an entry
    pub fn update_ipv6(&mut self, env_id: Uuid, ipv6: String) -> Result<()> {
        self.entries
            .get_mut(&env_id)
            .ok_or_else(|| DnsError::RegistrationError(format!("Entry not found for {}", env_id)))?
            .ipv6 = Some(ipv6);
        Ok(())
    }

    /// Activate an entry
    pub fn activate(&mut self, env_id: Uuid) -> Result<()> {
        self.entries
            .get_mut(&env_id)
            .ok_or_else(|| DnsError::RegistrationError(format!("Entry not found for {}", env_id)))?
            .is_active = true;
        Ok(())
    }

    /// Deactivate an entry
    pub fn deactivate(&mut self, env_id: Uuid) -> Result<()> {
        self.entries
            .get_mut(&env_id)
            .ok_or_else(|| DnsError::RegistrationError(format!("Entry not found for {}", env_id)))?
            .is_active = false;
        Ok(())
    }

    /// List all active entries
    pub fn list_active(&self) -> Vec<DnsEntry> {
        self.entries
            .values()
            .filter(|e| e.is_active)
            .cloned()
            .collect()
    }

    /// List all entries
    pub fn list_all(&self) -> Vec<DnsEntry> {
        self.entries.values().cloned().collect()
    }
}

impl Default for DnsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_entry() {
        let mut registry = DnsRegistry::new();
        let env_id = Uuid::new_v4();
        let entry = DnsEntry::new(env_id, "myenv".to_string(), "myenv.local".to_string())
            .with_ipv4("192.168.1.100".to_string())
            .activate();

        registry.register(entry.clone()).unwrap();

        let retrieved = registry.get_by_env_id(env_id).unwrap();
        assert_eq!(retrieved.fqdn, "myenv.local");
        assert_eq!(retrieved.ipv4, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_get_by_fqdn() {
        let mut registry = DnsRegistry::new();
        let env_id = Uuid::new_v4();
        let entry = DnsEntry::new(env_id, "myenv".to_string(), "myenv.local".to_string());

        registry.register(entry).unwrap();

        let retrieved = registry.get_by_fqdn("myenv.local").unwrap();
        assert_eq!(retrieved.env_id, env_id);
    }

    #[test]
    fn test_fqdn_conflict() {
        let mut registry = DnsRegistry::new();
        let env_id1 = Uuid::new_v4();
        let env_id2 = Uuid::new_v4();

        let entry1 = DnsEntry::new(env_id1, "env1".to_string(), "myenv.local".to_string());
        let entry2 = DnsEntry::new(env_id2, "env2".to_string(), "myenv.local".to_string());

        registry.register(entry1).unwrap();
        let result = registry.register(entry2);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_ipv4() {
        let mut registry = DnsRegistry::new();
        let env_id = Uuid::new_v4();
        let entry = DnsEntry::new(env_id, "myenv".to_string(), "myenv.local".to_string());

        registry.register(entry).unwrap();
        registry.update_ipv4(env_id, "192.168.1.100".to_string()).unwrap();

        let retrieved = registry.get_by_env_id(env_id).unwrap();
        assert_eq!(retrieved.ipv4, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_activate_deactivate() {
        let mut registry = DnsRegistry::new();
        let env_id = Uuid::new_v4();
        let entry = DnsEntry::new(env_id, "myenv".to_string(), "myenv.local".to_string());

        registry.register(entry).unwrap();
        assert_eq!(registry.list_active().len(), 0);

        registry.activate(env_id).unwrap();
        assert_eq!(registry.list_active().len(), 1);

        registry.deactivate(env_id).unwrap();
        assert_eq!(registry.list_active().len(), 0);
    }

    #[test]
    fn test_unregister() {
        let mut registry = DnsRegistry::new();
        let env_id = Uuid::new_v4();
        let entry = DnsEntry::new(env_id, "myenv".to_string(), "myenv.local".to_string());

        registry.register(entry).unwrap();
        assert_eq!(registry.list_all().len(), 1);

        registry.unregister(env_id).unwrap();
        assert_eq!(registry.list_all().len(), 0);
        assert!(registry.get_by_fqdn("myenv.local").is_none());
    }
}
