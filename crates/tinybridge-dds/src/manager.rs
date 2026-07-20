use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use tinybridge_core::{
    DdsAuditEvent, DdsConfig, DdsEventType, DdsProfile,
};

/// Error type for DDS operations
#[derive(Debug, Clone)]
pub enum DdsError {
    ConfigNotFound(String),
    AlreadyConfigured(String),
    InvalidConfiguration(String),
    UnauthorizedChange(String),
}

impl std::fmt::Display for DdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DdsError::ConfigNotFound(msg) => write!(f, "DDS config not found: {}", msg),
            DdsError::AlreadyConfigured(msg) => write!(f, "DDS already configured: {}", msg),
            DdsError::InvalidConfiguration(msg) => write!(f, "Invalid DDS configuration: {}", msg),
            DdsError::UnauthorizedChange(msg) => write!(f, "Unauthorized DDS change: {}", msg),
        }
    }
}

impl std::error::Error for DdsError {}

pub type Result<T> = std::result::Result<T, DdsError>;

/// Compliance report section for DDS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsComplianceReport {
    pub env_id: Uuid,
    pub dds_enabled: bool,
    pub enabled_features: Vec<String>,
    pub total_features_enabled: usize,
    pub security_enabled: bool,
    pub audit_events_count: usize,
    pub last_change: Option<chrono::DateTime<Utc>>,
    pub changed_by: Option<String>,
}

/// DDS manager handles configuration lifecycle and audit trails
pub struct DdsManager {
    configs: HashMap<Uuid, DdsConfig>,
    audit_log: Vec<DdsAuditEvent>,
}

impl DdsManager {
    /// Create a new DDS manager
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            audit_log: Vec::new(),
        }
    }

    /// Create a new DDS configuration for an environment (disabled by default)
    pub fn create_config(&mut self, env_id: Uuid) -> Result<DdsConfig> {
        if self.configs.contains_key(&env_id) {
            return Err(DdsError::AlreadyConfigured(env_id.to_string()));
        }

        let config = DdsConfig::new(env_id);
        self.configs.insert(env_id, config.clone());

        // Log creation event
        self.audit_log.push(DdsAuditEvent {
            id: Uuid::new_v4(),
            env_id,
            event_type: DdsEventType::DdsDisabled,
            timestamp: Utc::now(),
            changed_by: None,
            change_details: "DDS configuration created (disabled by default)".to_string(),
            old_value: None,
            new_value: Some("disabled".to_string()),
            change_reason: Some("Initial configuration".to_string()),
            requires_admin_to_undo: false,
        });

        Ok(config)
    }

    /// Get DDS configuration for an environment
    pub fn get_config(&self, env_id: Uuid) -> Result<DdsConfig> {
        self.configs
            .get(&env_id)
            .cloned()
            .ok_or_else(|| DdsError::ConfigNotFound(env_id.to_string()))
    }

    /// Enable DDS networking (requires explicit action)
    pub fn enable_dds(
        &mut self,
        env_id: Uuid,
        changed_by: Option<String>,
        reason: Option<String>,
    ) -> Result<()> {
        let config = self
            .configs
            .get_mut(&env_id)
            .ok_or_else(|| DdsError::ConfigNotFound(env_id.to_string()))?;

        let was_enabled = config.enabled;
        config.enabled = true;
        config.modified_at = Utc::now();
        config.modified_by = changed_by.clone();

        self.audit_log.push(DdsAuditEvent {
            id: Uuid::new_v4(),
            env_id,
            event_type: DdsEventType::DdsEnabled,
            timestamp: Utc::now(),
            changed_by,
            change_details: "DDS networking enabled".to_string(),
            old_value: Some(was_enabled.to_string()),
            new_value: Some("true".to_string()),
            change_reason: reason,
            requires_admin_to_undo: false,
        });

        Ok(())
    }

    /// Disable DDS networking immediately
    pub fn disable_dds(
        &mut self,
        env_id: Uuid,
        changed_by: Option<String>,
        reason: Option<String>,
    ) -> Result<()> {
        let config = self
            .configs
            .get_mut(&env_id)
            .ok_or_else(|| DdsError::ConfigNotFound(env_id.to_string()))?;

        config.enabled = false;
        config.modified_at = Utc::now();
        config.modified_by = changed_by.clone();

        self.audit_log.push(DdsAuditEvent {
            id: Uuid::new_v4(),
            env_id,
            event_type: DdsEventType::DdsDisabled,
            timestamp: Utc::now(),
            changed_by,
            change_details: "DDS networking disabled".to_string(),
            old_value: Some("true".to_string()),
            new_value: Some("false".to_string()),
            change_reason: reason,
            requires_admin_to_undo: false,
        });

        Ok(())
    }

    /// Toggle a specific DDS feature
    pub fn toggle_feature(
        &mut self,
        env_id: Uuid,
        feature: &str,
        enabled: bool,
        changed_by: Option<String>,
    ) -> Result<()> {
        let config = self
            .configs
            .get_mut(&env_id)
            .ok_or_else(|| DdsError::ConfigNotFound(env_id.to_string()))?;

        let old_value = match feature {
            "discovery" => {
                let old = config.features.discovery_enabled;
                config.features.discovery_enabled = enabled;
                old.to_string()
            }
            "multicast_discovery" => {
                let old = config.features.multicast_discovery_enabled;
                config.features.multicast_discovery_enabled = enabled;
                old.to_string()
            }
            "unicast_discovery" => {
                let old = config.features.unicast_discovery_enabled;
                config.features.unicast_discovery_enabled = enabled;
                old.to_string()
            }
            "router" => {
                let old = config.features.router_enabled;
                config.features.router_enabled = enabled;
                old.to_string()
            }
            "relay" => {
                let old = config.features.relay_enabled;
                config.features.relay_enabled = enabled;
                old.to_string()
            }
            "bridge" => {
                let old = config.features.bridge_enabled;
                config.features.bridge_enabled = enabled;
                old.to_string()
            }
            "monitoring" => {
                let old = config.features.monitoring_enabled;
                config.features.monitoring_enabled = enabled;
                old.to_string()
            }
            "topic_inspection" => {
                let old = config.features.topic_inspection_enabled;
                config.features.topic_inspection_enabled = enabled;
                old.to_string()
            }
            "packet_capture" => {
                let old = config.features.packet_capture_enabled;
                config.features.packet_capture_enabled = enabled;
                old.to_string()
            }
            "telemetry" => {
                let old = config.features.telemetry_enabled;
                config.features.telemetry_enabled = enabled;
                old.to_string()
            }
            "security" => {
                let old = config.features.security_enabled;
                config.features.security_enabled = enabled;
                old.to_string()
            }
            "cross_host_communication" => {
                let old = config.features.cross_host_communication_enabled;
                config.features.cross_host_communication_enabled = enabled;
                old.to_string()
            }
            "wan_communication" => {
                let old = config.features.wan_communication_enabled;
                config.features.wan_communication_enabled = enabled;
                old.to_string()
            }
            "vpn_integration" => {
                let old = config.features.vpn_integration_enabled;
                config.features.vpn_integration_enabled = enabled;
                old.to_string()
            }
            _ => return Err(DdsError::InvalidConfiguration(format!("Unknown feature: {}", feature))),
        };

        config.modified_at = Utc::now();
        config.modified_by = changed_by.clone();

        self.audit_log.push(DdsAuditEvent {
            id: Uuid::new_v4(),
            env_id,
            event_type: DdsEventType::FeatureChanged,
            timestamp: Utc::now(),
            changed_by,
            change_details: format!("{} feature toggled", feature),
            old_value: Some(old_value),
            new_value: Some(enabled.to_string()),
            change_reason: None,
            requires_admin_to_undo: false,
        });

        Ok(())
    }

    /// Apply a DDS profile
    pub fn apply_profile(
        &mut self,
        env_id: Uuid,
        profile: DdsProfile,
        changed_by: Option<String>,
        reason: Option<String>,
    ) -> Result<()> {
        let config = self
            .configs
            .get_mut(&env_id)
            .ok_or_else(|| DdsError::ConfigNotFound(env_id.to_string()))?;

        let old_profile = if config.enabled {
            "enabled with mixed features"
        } else {
            "disabled"
        };

        config.apply_profile(profile.clone());

        self.audit_log.push(DdsAuditEvent {
            id: Uuid::new_v4(),
            env_id,
            event_type: DdsEventType::ProfileApplied,
            timestamp: Utc::now(),
            changed_by,
            change_details: format!("DDS profile applied: {:?}", profile),
            old_value: Some(old_profile.to_string()),
            new_value: Some(format!("{:?}", profile)),
            change_reason: reason,
            requires_admin_to_undo: false,
        });

        Ok(())
    }

    /// Get audit log for an environment
    pub fn get_audit_log(&self, env_id: Uuid) -> Vec<DdsAuditEvent> {
        self.audit_log
            .iter()
            .filter(|e| e.env_id == env_id)
            .cloned()
            .collect()
    }

    /// Export audit log as JSON
    pub fn export_audit_log(&self, env_id: Uuid) -> String {
        let events = self.get_audit_log(env_id);
        serde_json::to_string_pretty(&events).unwrap_or_default()
    }

    /// Get compliance report for environment
    pub fn get_compliance_report(&self, env_id: Uuid) -> Result<DdsComplianceReport> {
        let config = self.get_config(env_id)?;
        let audit_events = self.get_audit_log(env_id);

        Ok(DdsComplianceReport {
            env_id,
            dds_enabled: config.enabled,
            enabled_features: config.enabled_features().iter().map(|s| s.to_string()).collect(),
            total_features_enabled: config.enabled_features().len(),
            security_enabled: config.security.encryption_enabled
                || config.security.authentication_enabled
                || config.security.access_control_enabled,
            audit_events_count: audit_events.len(),
            last_change: audit_events.last().map(|e| e.timestamp),
            changed_by: audit_events.last().and_then(|e| e.changed_by.clone()),
        })
    }

    /// List all environments with DDS enabled
    pub fn list_dds_enabled(&self) -> Vec<Uuid> {
        self.configs
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Count total DDS configurations
    pub fn count(&self) -> usize {
        self.configs.len()
    }

    /// Count DDS-enabled configurations
    pub fn count_enabled(&self) -> usize {
        self.configs.values().filter(|c| c.enabled).count()
    }
}

impl Default for DdsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_config() {
        let mut manager = DdsManager::new();
        let env_id = Uuid::new_v4();
        let config = manager.create_config(env_id).unwrap();
        assert!(!config.enabled);
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_duplicate_config() {
        let mut manager = DdsManager::new();
        let env_id = Uuid::new_v4();
        manager.create_config(env_id).unwrap();
        let result = manager.create_config(env_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_dds() {
        let mut manager = DdsManager::new();
        let env_id = Uuid::new_v4();
        manager.create_config(env_id).unwrap();
        manager
            .enable_dds(env_id, Some("admin".to_string()), Some("testing".to_string()))
            .unwrap();

        let config = manager.get_config(env_id).unwrap();
        assert!(config.enabled);
    }

    #[test]
    fn test_disable_dds() {
        let mut manager = DdsManager::new();
        let env_id = Uuid::new_v4();
        manager.create_config(env_id).unwrap();
        manager.enable_dds(env_id, None, None).unwrap();
        manager.disable_dds(env_id, None, None).unwrap();

        let config = manager.get_config(env_id).unwrap();
        assert!(!config.enabled);
    }

    #[test]
    fn test_toggle_feature() {
        let mut manager = DdsManager::new();
        let env_id = Uuid::new_v4();
        manager.create_config(env_id).unwrap();
        manager
            .toggle_feature(env_id, "discovery", true, Some("admin".to_string()))
            .unwrap();

        let config = manager.get_config(env_id).unwrap();
        assert!(config.features.discovery_enabled);
    }

    #[test]
    fn test_audit_log() {
        let mut manager = DdsManager::new();
        let env_id = Uuid::new_v4();
        manager.create_config(env_id).unwrap();
        manager.enable_dds(env_id, None, None).unwrap();

        let log = manager.get_audit_log(env_id);
        assert!(log.len() >= 2); // Creation + enablement
    }

    #[test]
    fn test_compliance_report() {
        let mut manager = DdsManager::new();
        let env_id = Uuid::new_v4();
        manager.create_config(env_id).unwrap();
        manager.enable_dds(env_id, None, None).unwrap();
        manager
            .toggle_feature(env_id, "discovery", true, None)
            .unwrap();

        let report = manager.get_compliance_report(env_id).unwrap();
        assert!(report.dds_enabled);
        assert_eq!(report.total_features_enabled, 1);
        assert!(report.audit_events_count >= 2);
    }

    #[test]
    fn test_apply_profile() {
        let mut manager = DdsManager::new();
        let env_id = Uuid::new_v4();
        manager.create_config(env_id).unwrap();
        manager
            .apply_profile(env_id, DdsProfile::Ros2Minimal, None, None)
            .unwrap();

        let config = manager.get_config(env_id).unwrap();
        assert!(config.enabled);
        assert!(config.features.discovery_enabled);
    }

    #[test]
    fn test_list_enabled() {
        let mut manager = DdsManager::new();
        let env1 = Uuid::new_v4();
        let env2 = Uuid::new_v4();

        manager.create_config(env1).unwrap();
        manager.create_config(env2).unwrap();
        manager.enable_dds(env1, None, None).unwrap();

        let enabled = manager.list_dds_enabled();
        assert_eq!(enabled.len(), 1);
        assert!(enabled.contains(&env1));
    }
}
