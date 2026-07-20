use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DDS networking configuration and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsConfig {
    /// Unique identifier for this DDS configuration
    pub id: Uuid,
    /// Associated environment ID
    pub env_id: Uuid,
    /// Master enable/disable for all DDS networking
    pub enabled: bool,
    /// When this configuration was created
    pub created_at: DateTime<Utc>,
    /// When this configuration was last modified
    pub modified_at: DateTime<Utc>,
    /// Who enabled/disabled DDS (audit trail)
    pub modified_by: Option<String>,
    /// Detailed DDS feature controls
    pub features: DdsFeatures,
    /// Security configuration for DDS
    pub security: DdsSecurityConfig,
    /// Network-level controls
    pub networking: DdsNetworkingConfig,
}

/// Individual DDS feature toggles (principle of least privilege)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DdsFeatures {
    /// Enable/disable DDS participant discovery
    pub discovery_enabled: bool,
    /// Enable/disable DDS multicast discovery (UDP multicast)
    pub multicast_discovery_enabled: bool,
    /// Enable/disable DDS unicast discovery (point-to-point)
    pub unicast_discovery_enabled: bool,
    /// Enable/disable DDS routers (forward data between networks)
    pub router_enabled: bool,
    /// Enable/disable DDS relays (similar to routers, different implementation)
    pub relay_enabled: bool,
    /// Enable/disable DDS bridges (connect different DDS domains)
    pub bridge_enabled: bool,
    /// Enable/disable DDS monitoring and introspection
    pub monitoring_enabled: bool,
    /// Enable/disable DDS topic inspection (read active topics)
    pub topic_inspection_enabled: bool,
    /// Enable/disable DDS packet capture for debugging
    pub packet_capture_enabled: bool,
    /// Enable/disable DDS telemetry collection
    pub telemetry_enabled: bool,
    /// Enable/disable DDS security plugins
    pub security_enabled: bool,
    /// Enable/disable cross-host DDS communication
    pub cross_host_communication_enabled: bool,
    /// Enable/disable WAN (wide-area network) communication
    pub wan_communication_enabled: bool,
    /// Enable/disable VPN integration (route DDS over VPN)
    pub vpn_integration_enabled: bool,
}

/// Security configuration for DDS communications
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DdsSecurityConfig {
    /// Enable encrypted DDS communications
    pub encryption_enabled: bool,
    /// Enable authentication of DDS participants
    pub authentication_enabled: bool,
    /// Enable access control policies
    pub access_control_enabled: bool,
    /// List of allowed DDS domains (empty = allow all enabled domains)
    pub allowed_domains: Vec<u32>,
    /// List of blocked DDS domains
    pub blocked_domains: Vec<u32>,
    /// Require allowlist for participants to join (if true, denylists ignored)
    pub use_participant_allowlist: bool,
    /// Allowed DDS participant names (if allowlist enabled)
    pub allowed_participants: Vec<String>,
    /// Blocked DDS participant names
    pub blocked_participants: Vec<String>,
    /// Network isolation: each domain in separate network namespace
    pub network_isolation_by_domain: bool,
}

/// Network-level controls for DDS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsNetworkingConfig {
    /// Primary DDS domain ID (typically 0)
    pub primary_domain_id: u32,
    /// Secondary DDS domains (for multi-domain setups)
    pub secondary_domains: Vec<u32>,
    /// UDP ports for multicast discovery (default: 7400-7409 range)
    pub discovery_ports: Vec<u16>,
    /// UDP ports for data communication
    pub data_ports: Vec<u16>,
    /// Multicast group addresses (e.g., "239.255.0.1")
    pub multicast_addresses: Vec<String>,
    /// Network interface to use for DDS (empty = auto-select)
    pub network_interface: Option<String>,
    /// Disable firewall rules (firewall must be manually opened)
    pub firewall_rules_enabled: bool,
    /// Allow DDS traffic to VPN interface
    pub vpn_traffic_allowed: bool,
    /// Maximum latency budget for DDS (ms)
    pub max_latency_budget_ms: u32,
    /// Maximum message size (bytes)
    pub max_message_size_bytes: u32,
}

impl Default for DdsNetworkingConfig {
    fn default() -> Self {
        Self {
            primary_domain_id: 0,
            secondary_domains: vec![],
            discovery_ports: vec![7400, 7401, 7402],
            data_ports: vec![7410, 7411, 7412],
            multicast_addresses: vec!["239.255.0.1".to_string()],
            network_interface: None,
            firewall_rules_enabled: true,
            vpn_traffic_allowed: false,
            max_latency_budget_ms: 100,
            max_message_size_bytes: 65_536,
        }
    }
}

/// DDS configuration profile for common use cases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DdsProfile {
    /// DDS completely disabled (default)
    Disabled,
    /// Minimal ROS 2 setup (discovery + local multicast only)
    Ros2Minimal,
    /// Full ROS 2 with monitoring and diagnostics
    Ros2Full,
    /// Multi-robot with routers and cross-host communication
    MultiRobot,
    /// Industrial automation with security and WAN
    Industrial,
    /// Custom configuration
    Custom,
}

/// Audit event for DDS configuration changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsAuditEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Environment ID
    pub env_id: Uuid,
    /// Type of change (enabled, disabled, feature_changed, security_changed)
    pub event_type: DdsEventType,
    /// When this change occurred
    pub timestamp: DateTime<Utc>,
    /// Who made this change
    pub changed_by: Option<String>,
    /// What was changed (feature name or config section)
    pub change_details: String,
    /// Old value (if applicable)
    pub old_value: Option<String>,
    /// New value (if applicable)
    pub new_value: Option<String>,
    /// Why was this change made (reason/justification)
    pub change_reason: Option<String>,
    /// Requires admin approval to undo
    pub requires_admin_to_undo: bool,
}

/// Types of DDS audit events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DdsEventType {
    /// DDS networking enabled for environment
    DdsEnabled,
    /// DDS networking disabled for environment
    DdsDisabled,
    /// Individual feature toggled
    FeatureChanged,
    /// Security configuration changed
    SecurityChanged,
    /// Network configuration changed
    NetworkingChanged,
    /// Profile applied
    ProfileApplied,
}

impl DdsConfig {
    /// Create a new DDS configuration (disabled by default)
    pub fn new(env_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            env_id,
            enabled: false,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            modified_by: None,
            features: DdsFeatures::default(),
            security: DdsSecurityConfig::default(),
            networking: DdsNetworkingConfig::default(),
        }
    }

    /// Check if DDS is fully enabled (master control AND at least one feature)
    pub fn is_operational(&self) -> bool {
        if !self.enabled {
            return false;
        }
        // At least one feature must be enabled for DDS to be operational
        self.features.discovery_enabled
            || self.features.router_enabled
            || self.features.relay_enabled
            || self.features.bridge_enabled
    }

    /// Get list of enabled features
    pub fn enabled_features(&self) -> Vec<&'static str> {
        let mut features = vec![];
        if self.features.discovery_enabled {
            features.push("discovery");
        }
        if self.features.multicast_discovery_enabled {
            features.push("multicast_discovery");
        }
        if self.features.unicast_discovery_enabled {
            features.push("unicast_discovery");
        }
        if self.features.router_enabled {
            features.push("router");
        }
        if self.features.relay_enabled {
            features.push("relay");
        }
        if self.features.bridge_enabled {
            features.push("bridge");
        }
        if self.features.monitoring_enabled {
            features.push("monitoring");
        }
        if self.features.topic_inspection_enabled {
            features.push("topic_inspection");
        }
        if self.features.packet_capture_enabled {
            features.push("packet_capture");
        }
        if self.features.telemetry_enabled {
            features.push("telemetry");
        }
        if self.features.security_enabled {
            features.push("security");
        }
        if self.features.cross_host_communication_enabled {
            features.push("cross_host_communication");
        }
        if self.features.wan_communication_enabled {
            features.push("wan_communication");
        }
        if self.features.vpn_integration_enabled {
            features.push("vpn_integration");
        }
        features
    }

    /// Apply a DDS profile (convenience configuration)
    pub fn apply_profile(&mut self, profile: DdsProfile) {
        match profile {
            DdsProfile::Disabled => {
                self.enabled = false;
                self.features = DdsFeatures::default();
            }
            DdsProfile::Ros2Minimal => {
                self.enabled = true;
                self.features.discovery_enabled = true;
                self.features.multicast_discovery_enabled = true;
                self.features.unicast_discovery_enabled = true;
                // Routers, relays, bridges stay disabled
            }
            DdsProfile::Ros2Full => {
                self.enabled = true;
                self.features.discovery_enabled = true;
                self.features.multicast_discovery_enabled = true;
                self.features.unicast_discovery_enabled = true;
                self.features.monitoring_enabled = true;
                self.features.topic_inspection_enabled = true;
                self.features.telemetry_enabled = true;
            }
            DdsProfile::MultiRobot => {
                self.enabled = true;
                self.features.discovery_enabled = true;
                self.features.multicast_discovery_enabled = true;
                self.features.unicast_discovery_enabled = true;
                self.features.router_enabled = true;
                self.features.relay_enabled = true;
                self.features.bridge_enabled = true;
                self.features.cross_host_communication_enabled = true;
                self.features.monitoring_enabled = true;
                self.security.encryption_enabled = true;
                self.security.authentication_enabled = true;
            }
            DdsProfile::Industrial => {
                self.enabled = true;
                self.features.discovery_enabled = true;
                self.features.multicast_discovery_enabled = true;
                self.features.unicast_discovery_enabled = true;
                self.features.router_enabled = true;
                self.features.relay_enabled = true;
                self.features.wan_communication_enabled = true;
                self.features.vpn_integration_enabled = true;
                self.features.monitoring_enabled = true;
                self.features.telemetry_enabled = true;
                self.security.encryption_enabled = true;
                self.security.authentication_enabled = true;
                self.security.access_control_enabled = true;
                self.networking.vpn_traffic_allowed = true;
            }
            DdsProfile::Custom => {
                // Do nothing; caller manages features individually
            }
        }
        self.modified_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dds_config_default_disabled() {
        let config = DdsConfig::new(Uuid::new_v4());
        assert!(!config.enabled);
        assert!(!config.is_operational());
    }

    #[test]
    fn test_dds_enabled_but_no_features() {
        let mut config = DdsConfig::new(Uuid::new_v4());
        config.enabled = true;
        assert!(!config.is_operational());
    }

    #[test]
    fn test_dds_enabled_with_feature() {
        let mut config = DdsConfig::new(Uuid::new_v4());
        config.enabled = true;
        config.features.discovery_enabled = true;
        assert!(config.is_operational());
    }

    #[test]
    fn test_dds_enabled_features_list() {
        let mut config = DdsConfig::new(Uuid::new_v4());
        config.features.discovery_enabled = true;
        config.features.monitoring_enabled = true;
        let enabled = config.enabled_features();
        assert!(enabled.contains(&"discovery"));
        assert!(enabled.contains(&"monitoring"));
        assert_eq!(enabled.len(), 2);
    }

    #[test]
    fn test_apply_ros2_minimal_profile() {
        let mut config = DdsConfig::new(Uuid::new_v4());
        config.apply_profile(DdsProfile::Ros2Minimal);
        assert!(config.enabled);
        assert!(config.features.discovery_enabled);
        assert!(config.features.multicast_discovery_enabled);
        assert!(!config.features.router_enabled);
    }

    #[test]
    fn test_apply_multi_robot_profile() {
        let mut config = DdsConfig::new(Uuid::new_v4());
        config.apply_profile(DdsProfile::MultiRobot);
        assert!(config.enabled);
        assert!(config.features.router_enabled);
        assert!(config.security.encryption_enabled);
    }

    #[test]
    fn test_dds_security_defaults() {
        let security = DdsSecurityConfig::default();
        assert!(!security.encryption_enabled);
        assert!(!security.authentication_enabled);
        assert!(security.allowed_domains.is_empty());
    }
}
