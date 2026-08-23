/// DDS networking configuration for ROS 2 environments
///
/// Configures DDS middleware for ROS 2 native networking.
/// Supports multicast, UDP, and TCP transport.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DDS transport type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DDSTransport {
    /// Multicast (default for LAN)
    #[default]
    Multicast,
    /// Unicast UDP
    UnicodeUdp,
    /// TCP for WAN/firewalled networks
    Tcp,
    /// Shared memory (local inter-process)
    SharedMemory,
}

/// DDS domain ID (0-232 valid range)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DDSDomainId(u8);

impl DDSDomainId {
    pub fn new(id: u8) -> Result<Self, String> {
        if id <= 232 {
            Ok(DDSDomainId(id))
        } else {
            Err(format!("Domain ID must be 0-232, got {}", id))
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

/// DDS participant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDSParticipantConfig {
    /// Domain ID for DDS
    pub domain_id: u8,
    /// Transport type
    pub transport: DDSTransport,
    /// Allow multicast
    pub enable_multicast: bool,
    /// Multicast addresses to listen on
    pub multicast_addresses: Vec<String>,
    /// UDP/TCP port range
    pub port_base: u16,
    /// Maximum participant connections
    pub max_participants: u32,
    /// Heartbeat period (ms)
    pub heartbeat_period_ms: u32,
    /// Discovery enabled
    pub enable_discovery: bool,
    /// QoS settings
    pub qos_settings: HashMap<String, String>,
}

impl Default for DDSParticipantConfig {
    fn default() -> Self {
        DDSParticipantConfig {
            domain_id: 0,
            transport: DDSTransport::Multicast,
            enable_multicast: true,
            multicast_addresses: vec!["239.255.0.1".to_string()],
            port_base: 7400,
            max_participants: 100,
            heartbeat_period_ms: 100,
            enable_discovery: true,
            qos_settings: HashMap::new(),
        }
    }
}

impl DDSParticipantConfig {
    pub fn for_environment(env_name: &str) -> Self {
        let mut config = DDSParticipantConfig::default();
        config
            .qos_settings
            .insert("participant".to_string(), format!("env-{}", env_name));
        config
    }

    /// Create Fast-DDS XML configuration
    pub fn to_fast_dds_xml(&self) -> String {
        let transport_str = match self.transport {
            DDSTransport::Multicast => "DEFAULT",
            DDSTransport::UnicodeUdp => "UDPv4",
            DDSTransport::Tcp => "TCPv4",
            DDSTransport::SharedMemory => "SHM",
        };

        format!(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<dds>
    <profiles xmlns="http://www.eprosima.com/XMLSchemas/fastRTPS_Profiles">
        <participant profile_name="default_participant" is_default_profile="true">
            <rtps>
                <name>ROS2Participant</name>
                <domainId>{}</domainId>
                <participantID>1</participantID>
                <defaultUnicastLocatorList>
                    <locator>
                        <udpv4>
                            <port>{}</port>
                        </udpv4>
                    </locator>
                </defaultUnicastLocatorList>
                <defaultMulticastLocatorList>
                    <locator>
                        <udpv4>
                            <address>{}</address>
                            <port>{}</port>
                        </udpv4>
                    </locator>
                </defaultMulticastLocatorList>
                <useBuiltinTransports>{}</useBuiltinTransports>
                <discovery>
                    <discovery_config discovery_protocol="SIMPLE">
                        <discoveryServersList/>
                    </discovery_config>
                </discovery>
                <builtin>
                    <discovery_config discovery_protocol="SIMPLE"/>
                    <metadata_policy_qos>
                        <history>
                            <kind>KEEP_LAST</kind>
                            <depth>10</depth>
                        </history>
                    </metadata_policy_qos>
                </builtin>
            </rtps>
        </participant>
    </profiles>
</dds>"#,
            self.domain_id,
            self.port_base,
            self.multicast_addresses
                .first()
                .unwrap_or(&"239.255.0.1".to_string()),
            self.port_base + 1,
            if self.enable_multicast {
                "true"
            } else {
                "false"
            }
        )
    }

    /// Create environment variables for ROS 2
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert("ROS_DOMAIN_ID".to_string(), self.domain_id.to_string());

        env.insert(
            "RMW_IMPLEMENTATION".to_string(),
            "rmw_fastrtps_cpp".to_string(),
        );

        match self.transport {
            DDSTransport::Multicast => {
                env.insert(
                    "FASTRTPS_DEFAULT_PROFILES_FILE".to_string(),
                    "/etc/ros/fastdds_profiles.xml".to_string(),
                );
            }
            DDSTransport::Tcp => {
                env.insert("ROS_DOMAIN_ID".to_string(), self.domain_id.to_string());
                env.insert(
                    "FASTRTPS_DEFAULT_PROFILES_FILE".to_string(),
                    "/etc/ros/fastdds_tcp_profiles.xml".to_string(),
                );
            }
            _ => {}
        }

        env
    }
}

/// DDS network status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDSNetworkStatus {
    /// Environment name
    pub environment: String,
    /// Domain ID
    pub domain_id: u8,
    /// Transport type
    pub transport: String,
    /// Is operational
    pub operational: bool,
    /// Connected participants
    pub connected_participants: usize,
    /// ROS 2 nodes discovered
    pub ros_nodes: Vec<String>,
    /// ROS 2 topics discovered
    pub ros_topics: Vec<String>,
    /// Last heartbeat
    pub last_heartbeat: String,
    /// Latency (ms)
    pub latency_ms: f64,
}

impl DDSNetworkStatus {
    pub fn new(environment: String, domain_id: u8) -> Self {
        DDSNetworkStatus {
            environment,
            domain_id,
            transport: "multicast".to_string(),
            operational: false,
            connected_participants: 0,
            ros_nodes: vec![],
            ros_topics: vec![],
            last_heartbeat: chrono::Utc::now().to_rfc3339(),
            latency_ms: 0.0,
        }
    }

    pub fn mark_operational(&mut self) {
        self.operational = true;
        self.last_heartbeat = chrono::Utc::now().to_rfc3339();
    }

    pub fn add_node(&mut self, node_name: String) {
        if !self.ros_nodes.contains(&node_name) {
            self.ros_nodes.push(node_name);
        }
    }

    pub fn add_topic(&mut self, topic_name: String) {
        if !self.ros_topics.contains(&topic_name) {
            self.ros_topics.push(topic_name);
        }
    }
}

/// DDS network manager
pub struct DDSNetworkManager {
    environments: std::collections::HashMap<String, DDSNetworkStatus>,
}

impl DDSNetworkManager {
    pub fn new() -> Self {
        DDSNetworkManager {
            environments: std::collections::HashMap::new(),
        }
    }

    /// Register environment with DDS
    pub fn register_environment(&mut self, env_name: String, domain_id: u8) -> DDSNetworkStatus {
        let status = DDSNetworkStatus::new(env_name.clone(), domain_id);
        self.environments.insert(env_name, status.clone());
        status
    }

    /// Get network status for environment
    pub fn get_status(&self, env_name: &str) -> Option<&DDSNetworkStatus> {
        self.environments.get(env_name)
    }

    /// Update network status
    pub fn update_status(&mut self, env_name: &str, status: DDSNetworkStatus) -> bool {
        if self.environments.contains_key(env_name) {
            self.environments.insert(env_name.to_string(), status);
            true
        } else {
            false
        }
    }

    /// Get all statuses
    pub fn get_all_statuses(&self) -> Vec<DDSNetworkStatus> {
        self.environments.values().cloned().collect()
    }

    /// Check if environment is operational
    pub fn is_operational(&self, env_name: &str) -> bool {
        self.environments
            .get(env_name)
            .map(|s| s.operational)
            .unwrap_or(false)
    }

    /// Get operational environments
    pub fn get_operational_environments(&self) -> Vec<String> {
        self.environments
            .values()
            .filter(|s| s.operational)
            .map(|s| s.environment.clone())
            .collect()
    }
}

impl Default for DDSNetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_id_creation() {
        assert!(DDSDomainId::new(0).is_ok());
        assert!(DDSDomainId::new(232).is_ok());
        assert!(DDSDomainId::new(233).is_err());
    }

    #[test]
    fn test_participant_config_default() {
        let config = DDSParticipantConfig::default();

        assert_eq!(config.domain_id, 0);
        assert_eq!(config.transport, DDSTransport::Multicast);
        assert!(config.enable_discovery);
    }

    #[test]
    fn test_participant_config_for_environment() {
        let config = DDSParticipantConfig::for_environment("ros-env");

        assert_eq!(config.domain_id, 0);
        assert!(config.qos_settings.contains_key("participant"));
    }

    #[test]
    fn test_fast_dds_xml_generation() {
        let config = DDSParticipantConfig::default();
        let xml = config.to_fast_dds_xml();

        assert!(xml.contains("<?xml"));
        assert!(xml.contains("domainId"));
        assert!(xml.contains("ROS2Participant"));
    }

    #[test]
    fn test_env_vars_generation() {
        let config = DDSParticipantConfig::default();
        let env_vars = config.to_env_vars();

        assert!(env_vars.contains_key("ROS_DOMAIN_ID"));
        assert!(env_vars.contains_key("RMW_IMPLEMENTATION"));
    }

    #[test]
    fn test_dds_network_status_creation() {
        let status = DDSNetworkStatus::new("test-env".to_string(), 0);

        assert_eq!(status.environment, "test-env");
        assert_eq!(status.domain_id, 0);
        assert!(!status.operational);
    }

    #[test]
    fn test_dds_network_status_operations() {
        let mut status = DDSNetworkStatus::new("test-env".to_string(), 0);

        status.mark_operational();
        assert!(status.operational);

        status.add_node("/node1".to_string());
        status.add_topic("/topic1".to_string());

        assert_eq!(status.ros_nodes.len(), 1);
        assert_eq!(status.ros_topics.len(), 1);

        // Adding same should not duplicate
        status.add_node("/node1".to_string());
        assert_eq!(status.ros_nodes.len(), 1);
    }

    #[test]
    fn test_dds_network_manager() {
        let mut manager = DDSNetworkManager::new();

        let status = manager.register_environment("env1".to_string(), 0);
        assert!(!status.operational);

        assert!(manager.get_status("env1").is_some());
        assert!(!manager.is_operational("env1"));

        let mut updated_status = status;
        updated_status.mark_operational();
        manager.update_status("env1", updated_status);

        assert!(manager.is_operational("env1"));
    }

    #[test]
    fn test_dds_network_manager_operational_list() {
        let mut manager = DDSNetworkManager::new();

        manager.register_environment("env1".to_string(), 0);
        let mut status2 = manager.register_environment("env2".to_string(), 1);
        status2.mark_operational();

        manager.update_status("env2", status2);

        let operational = manager.get_operational_environments();
        assert_eq!(operational.len(), 1);
        assert!(operational.contains(&"env2".to_string()));
    }
}
