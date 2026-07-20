use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use uuid::Uuid;

/// Network connectivity status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectivityStatus {
    /// VM is reachable and responsive
    Online,
    /// VM is not responding to network queries
    Offline,
    /// Connectivity status unknown (not yet checked)
    Unknown,
}

/// Network configuration for an environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Primary IP address (IPv4)
    pub ipv4: Option<String>,
    /// Secondary IP address (IPv6)
    pub ipv6: Option<String>,
    /// DNS servers (discovered)
    pub dns_servers: Vec<String>,
    /// Default gateway
    pub gateway: Option<String>,
    /// Subnet mask (CIDR notation)
    pub subnet: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            ipv4: None,
            ipv6: None,
            dns_servers: Vec::new(),
            gateway: None,
            subnet: None,
        }
    }
}

/// Security event for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    /// IP address changed unexpectedly
    UnexpectedIpChange { from: String, to: String },
    /// Multiple rapid IP changes (possible compromise)
    RapidIpChanges { count: u32 },
    /// VM came online from unexpected subnet
    UnexpectedSubnet { ip: String },
    /// Connection attempt from suspicious destination
    SuspiciousConnection { ip: String },
}

/// VPN/Firewall status detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkPath {
    /// Traffic routed through VPN
    Vpn,
    /// Traffic blocked by firewall
    Blocked,
    /// Traffic allowed directly
    Direct,
    /// Status unknown
    Unknown,
}

/// Comprehensive IP address tracking for environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddressRecord {
    /// Environment ID
    pub env_id: Uuid,
    /// Environment name
    pub env_name: String,
    /// SSH config alias
    pub ssh_alias: String,
    /// Network configuration
    pub network: NetworkConfig,
    /// Connectivity status
    pub status: ConnectivityStatus,
    /// VPN/Firewall path detection
    pub network_path: NetworkPath,
    /// Last IP change timestamp
    pub last_ip_change: Option<DateTime<Utc>>,
    /// Last connectivity check
    pub last_check: Option<DateTime<Utc>>,
    /// SSH port (for connectivity checks)
    pub ssh_port: u16,
    /// Security events log (recent anomalies)
    pub security_events: Vec<SecurityEvent>,
}

impl IpAddressRecord {
    /// Create a new IP address record
    pub fn new(env_id: Uuid, env_name: String, ssh_alias: String) -> Self {
        Self {
            env_id,
            env_name,
            ssh_alias,
            network: NetworkConfig::default(),
            status: ConnectivityStatus::Unknown,
            network_path: NetworkPath::Unknown,
            last_ip_change: None,
            last_check: None,
            ssh_port: 22,
            security_events: Vec::new(),
        }
    }

    /// Update primary IP address and return true if changed
    pub fn update_ipv4(&mut self, new_ip: Option<String>) -> bool {
        if let Some(ref ip) = new_ip {
            if !Self::is_valid_ip(ip) {
                return false;
            }
        }
        if self.network.ipv4 != new_ip {
            self.network.ipv4 = new_ip;
            self.last_ip_change = Some(Utc::now());
            return true;
        }
        false
    }

    /// Update secondary IPv6 address
    pub fn update_ipv6(&mut self, new_ip: Option<String>) -> bool {
        if let Some(ref ip) = new_ip {
            if !Self::is_valid_ip(ip) {
                return false;
            }
        }
        if self.network.ipv6 != new_ip {
            self.network.ipv6 = new_ip;
            return true;
        }
        false
    }

    /// Update network configuration
    pub fn update_network_config(&mut self, config: NetworkConfig) {
        self.network = config;
    }

    /// Update connectivity status
    pub fn set_connectivity_status(&mut self, status: ConnectivityStatus) {
        self.status = status;
        self.last_check = Some(Utc::now());
    }

    /// Update VPN/Firewall path detection
    pub fn set_network_path(&mut self, path: NetworkPath) {
        self.network_path = path;
    }

    /// Record a security event
    pub fn record_security_event(&mut self, event: SecurityEvent) {
        self.security_events.push(event);
        // Keep only last 10 events
        if self.security_events.len() > 10 {
            self.security_events.remove(0);
        }
    }

    /// Get primary IP (preferring IPv4)
    pub fn primary_ip(&self) -> Option<String> {
        self.network
            .ipv4
            .clone()
            .or_else(|| self.network.ipv6.clone())
    }

    /// Check if VM is currently reachable
    pub fn is_online(&self) -> bool {
        self.status == ConnectivityStatus::Online
    }

    /// Check if IP is valid
    pub fn is_valid_ip(ip: &str) -> bool {
        IpAddr::from_str(ip).is_ok()
    }

    /// Generate VM discovery metadata
    pub fn vm_discovery_metadata(&self) -> VmDiscoveryMetadata {
        VmDiscoveryMetadata {
            name: self.env_name.clone(),
            ip_address: self.primary_ip(),
            hostname: format!("{}.local", self.env_name),
            ssh_command: self.primary_ip().map(|ip| format!("ssh ubuntu@{}", ip)),
            http_url: self
                .network
                .ipv4
                .as_ref()
                .map(|ip| format!("http://{}:8080", ip)),
            status: self.status,
            network_path: self.network_path,
        }
    }
}

/// VM discovery information (for UI/CLI display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmDiscoveryMetadata {
    pub name: String,
    pub ip_address: Option<String>,
    pub hostname: String,
    pub ssh_command: Option<String>,
    pub http_url: Option<String>,
    pub status: ConnectivityStatus,
    pub network_path: NetworkPath,
}

/// Comprehensive IP monitoring system (OrbStack-style)
pub struct IpMonitor {
    records: HashMap<Uuid, IpAddressRecord>,
    ip_to_env: HashMap<String, Uuid>, // Reverse lookup: IP → Env ID
}

impl IpMonitor {
    /// Create a new IP monitor
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            ip_to_env: HashMap::new(),
        }
    }

    /// Register an environment for monitoring
    pub fn register(&mut self, env_id: Uuid, env_name: String, ssh_alias: String) {
        self.records
            .insert(env_id, IpAddressRecord::new(env_id, env_name, ssh_alias));
    }

    /// Unregister an environment
    pub fn unregister(&mut self, env_id: Uuid) -> Option<IpAddressRecord> {
        if let Some(record) = self.records.remove(&env_id) {
            // Clean up reverse lookup
            if let Some(ip) = &record.network.ipv4 {
                self.ip_to_env.remove(ip);
            }
            if let Some(ip) = &record.network.ipv6 {
                self.ip_to_env.remove(ip);
            }
            return Some(record);
        }
        None
    }

    /// Update IPv4 address and check for changes
    pub fn update_ipv4(&mut self, env_id: Uuid, new_ip: Option<String>) -> bool {
        if let Some(record) = self.records.get_mut(&env_id) {
            // Update reverse lookup
            if let Some(old_ip) = &record.network.ipv4 {
                self.ip_to_env.remove(old_ip);
            }
            if let Some(ref new_ip) = new_ip {
                self.ip_to_env.insert(new_ip.clone(), env_id);
            }
            record.update_ipv4(new_ip)
        } else {
            false
        }
    }

    /// Update IPv6 address
    pub fn update_ipv6(&mut self, env_id: Uuid, new_ip: Option<String>) -> bool {
        if let Some(record) = self.records.get_mut(&env_id) {
            if let Some(old_ip) = &record.network.ipv6 {
                self.ip_to_env.remove(old_ip);
            }
            if let Some(ref new_ip) = new_ip {
                self.ip_to_env.insert(new_ip.clone(), env_id);
            }
            record.update_ipv6(new_ip)
        } else {
            false
        }
    }

    /// Update network configuration
    pub fn update_network_config(&mut self, env_id: Uuid, config: NetworkConfig) -> bool {
        if let Some(record) = self.records.get_mut(&env_id) {
            record.update_network_config(config);
            true
        } else {
            false
        }
    }

    /// Update connectivity status
    pub fn set_connectivity_status(&mut self, env_id: Uuid, status: ConnectivityStatus) -> bool {
        if let Some(record) = self.records.get_mut(&env_id) {
            record.set_connectivity_status(status);
            true
        } else {
            false
        }
    }

    /// Detect and record network path (VPN/Firewall/Direct)
    pub fn detect_network_path(&mut self, env_id: Uuid, path: NetworkPath) -> bool {
        if let Some(record) = self.records.get_mut(&env_id) {
            record.set_network_path(path);
            true
        } else {
            false
        }
    }

    /// Record a security event (anomaly detection)
    pub fn record_security_event(&mut self, env_id: Uuid, event: SecurityEvent) -> bool {
        if let Some(record) = self.records.get_mut(&env_id) {
            record.record_security_event(event);
            true
        } else {
            false
        }
    }

    /// Get record by environment ID
    pub fn get_by_env_id(&self, env_id: Uuid) -> Option<IpAddressRecord> {
        self.records.get(&env_id).cloned()
    }

    /// Get environment ID by IP address
    pub fn get_env_by_ip(&self, ip: &str) -> Option<Uuid> {
        self.ip_to_env.get(ip).copied()
    }

    /// Get all records
    pub fn list_all(&self) -> Vec<IpAddressRecord> {
        self.records.values().cloned().collect()
    }

    /// Get all online VMs
    pub fn list_online(&self) -> Vec<IpAddressRecord> {
        self.records
            .values()
            .filter(|r| r.is_online())
            .cloned()
            .collect()
    }

    /// Generate VM discovery list (for UI/dashboard)
    pub fn vm_discovery(&self) -> Vec<VmDiscoveryMetadata> {
        self.records
            .values()
            .map(|r| r.vm_discovery_metadata())
            .collect()
    }

    /// Detect suspicious activity (security monitoring)
    pub fn has_security_alerts(&self, env_id: Uuid) -> bool {
        self.records
            .get(&env_id)
            .map(|r| !r.security_events.is_empty())
            .unwrap_or(false)
    }

    /// Get security events for an environment
    pub fn get_security_events(&self, env_id: Uuid) -> Vec<SecurityEvent> {
        self.records
            .get(&env_id)
            .map(|r| r.security_events.clone())
            .unwrap_or_default()
    }

    /// Count monitored environments
    pub fn count(&self) -> usize {
        self.records.len()
    }

    /// Count online environments
    pub fn count_online(&self) -> usize {
        self.records.values().filter(|r| r.is_online()).count()
    }
}

impl Default for IpMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_address_record() {
        let mut record =
            IpAddressRecord::new(Uuid::new_v4(), "test".to_string(), "alias".to_string());
        assert_eq!(record.status, ConnectivityStatus::Unknown);
        assert!(record.update_ipv4(Some("192.168.1.100".to_string())));
        assert_eq!(record.primary_ip(), Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_vm_discovery_metadata() {
        let mut record =
            IpAddressRecord::new(Uuid::new_v4(), "myvm".to_string(), "alias".to_string());
        record.update_ipv4(Some("192.168.1.100".to_string()));
        record.set_connectivity_status(ConnectivityStatus::Online);

        let metadata = record.vm_discovery_metadata();
        assert_eq!(metadata.name, "myvm");
        assert_eq!(metadata.hostname, "myvm.local");
        assert!(metadata.ssh_command.is_some());
    }

    #[test]
    fn test_ip_monitor_lifecycle() {
        let mut monitor = IpMonitor::new();
        let env_id = Uuid::new_v4();

        monitor.register(env_id, "myvm".to_string(), "alias".to_string());
        assert_eq!(monitor.count(), 1);

        assert!(monitor.update_ipv4(env_id, Some("192.168.1.100".to_string())));
        assert_eq!(monitor.get_env_by_ip("192.168.1.100"), Some(env_id));

        monitor.set_connectivity_status(env_id, ConnectivityStatus::Online);
        assert_eq!(monitor.count_online(), 1);

        monitor.unregister(env_id);
        assert_eq!(monitor.count(), 0);
    }

    #[test]
    fn test_security_event_tracking() {
        let mut monitor = IpMonitor::new();
        let env_id = Uuid::new_v4();

        monitor.register(env_id, "myvm".to_string(), "alias".to_string());
        assert!(!monitor.has_security_alerts(env_id));

        monitor.record_security_event(
            env_id,
            SecurityEvent::UnexpectedIpChange {
                from: "192.168.1.100".to_string(),
                to: "192.168.1.101".to_string(),
            },
        );

        assert!(monitor.has_security_alerts(env_id));
        assert_eq!(monitor.get_security_events(env_id).len(), 1);
    }

    #[test]
    fn test_ipv6_support() {
        let mut monitor = IpMonitor::new();
        let env_id = Uuid::new_v4();

        monitor.register(env_id, "myvm".to_string(), "alias".to_string());
        monitor.update_ipv4(env_id, Some("192.168.1.100".to_string()));
        monitor.update_ipv6(env_id, Some("fe80::1".to_string()));

        let record = monitor.get_by_env_id(env_id).unwrap();
        assert_eq!(record.network.ipv4, Some("192.168.1.100".to_string()));
        assert_eq!(record.network.ipv6, Some("fe80::1".to_string()));
    }

    #[test]
    fn test_network_path_detection() {
        let mut monitor = IpMonitor::new();
        let env_id = Uuid::new_v4();

        monitor.register(env_id, "myvm".to_string(), "alias".to_string());
        monitor.detect_network_path(env_id, NetworkPath::Vpn);

        let record = monitor.get_by_env_id(env_id).unwrap();
        assert_eq!(record.network_path, NetworkPath::Vpn);
    }
}
