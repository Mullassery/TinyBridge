use crate::device::Device;
use crate::device::DeviceType;
use crate::error::{DeviceError, Result};
use crate::policy::{PolicyEngine, PolicyAuditEvent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Compliance report for environment device usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Environment ID
    pub env_id: Uuid,
    /// When this report was generated
    pub generated_at: DateTime<Utc>,
    /// Total devices registered
    pub total_devices: usize,
    /// Devices currently attached to this environment
    pub attached_devices: usize,
    /// Number of active policy rules
    pub policy_rules: usize,
    /// Total audit events for this environment
    pub audit_events_count: usize,
    /// Number of blocked attachment attempts
    pub blocked_attempts: usize,
    /// Detailed audit events
    pub audit_events: Vec<PolicyAuditEvent>,
}

/// Device manager for lifecycle operations with policy enforcement
pub struct DeviceManager {
    devices: HashMap<Uuid, Device>,
    host_path_to_id: HashMap<PathBuf, Uuid>, // Reverse lookup
    policy_engine: PolicyEngine,              // Policy enforcement
}

impl DeviceManager {
    /// Create a new device manager with default allow-all policies
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            host_path_to_id: HashMap::new(),
            policy_engine: PolicyEngine::new(),
        }
    }

    /// Get mutable access to policy engine
    pub fn policy_engine(&mut self) -> &mut PolicyEngine {
        &mut self.policy_engine
    }

    /// Get immutable access to policy engine
    pub fn policies(&self) -> &PolicyEngine {
        &self.policy_engine
    }

    /// Register a device
    pub fn register(&mut self, device: Device) -> Result<Uuid> {
        let id = device.id;
        let host_path = device.host_path.clone();

        // Check for duplicate paths
        if self.host_path_to_id.contains_key(&host_path) {
            return Err(DeviceError::AlreadyAttached(
                format!("Device at {} already registered", host_path.display()),
            ));
        }

        self.host_path_to_id.insert(host_path, id);
        self.devices.insert(id, device);
        Ok(id)
    }

    /// Unregister a device
    pub fn unregister(&mut self, id: Uuid) -> Result<Device> {
        if let Some(device) = self.devices.remove(&id) {
            self.host_path_to_id.remove(&device.host_path);
            Ok(device)
        } else {
            Err(DeviceError::DeviceNotFound(id.to_string()))
        }
    }

    /// Get device by ID
    pub fn get(&self, id: Uuid) -> Option<Device> {
        self.devices.get(&id).cloned()
    }

    /// Get device by host path
    pub fn get_by_path(&self, path: &std::path::Path) -> Option<Device> {
        self.host_path_to_id
            .get(path)
            .and_then(|id| self.devices.get(id).cloned())
    }

    /// Attach device to environment with policy checks
    pub fn attach(&mut self, device_id: Uuid, env_id: Uuid, user_id: Option<&str>) -> Result<()> {
        let device = self
            .devices
            .get(&device_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(device_id.to_string()))?;

        // Check policy before attaching
        let access_result = self.policy_engine.check_access(device.device_type, Some(env_id), user_id);

        if !access_result.allowed {
            return Err(DeviceError::PermissionDenied(
                access_result.user_message(),
            ));
        }

        // Policy check passed, proceed with attachment
        let device = self
            .devices
            .get_mut(&device_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(device_id.to_string()))?;

        device.attach(env_id)
            .map_err(|e| DeviceError::AttachError(e))
    }

    /// Detach device from environment
    pub fn detach(&mut self, device_id: Uuid) -> Result<()> {
        let device = self
            .devices
            .get_mut(&device_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(device_id.to_string()))?;

        device.detach();
        Ok(())
    }

    /// List all devices
    pub fn list_all(&self) -> Vec<Device> {
        self.devices.values().cloned().collect()
    }

    /// List available devices (not attached)
    pub fn list_available(&self) -> Vec<Device> {
        self.devices
            .values()
            .filter(|d| d.is_available())
            .cloned()
            .collect()
    }

    /// List attached devices
    pub fn list_attached(&self) -> Vec<Device> {
        self.devices
            .values()
            .filter(|d| d.is_attached())
            .cloned()
            .collect()
    }

    /// List devices attached to an environment
    pub fn list_for_env(&self, env_id: Uuid) -> Vec<Device> {
        self.devices
            .values()
            .filter(|d| d.attached_to_env == Some(env_id))
            .cloned()
            .collect()
    }

    /// List devices by type
    pub fn list_by_type(&self, device_type: DeviceType) -> Vec<Device> {
        self.devices
            .values()
            .filter(|d| d.device_type == device_type)
            .cloned()
            .collect()
    }

    /// Count total devices
    pub fn count(&self) -> usize {
        self.devices.len()
    }

    /// Count available devices
    pub fn count_available(&self) -> usize {
        self.devices.values().filter(|d| d.is_available()).count()
    }

    /// Count attached devices
    pub fn count_attached(&self) -> usize {
        self.devices.values().filter(|d| d.is_attached()).count()
    }

    /// Check if a device can be attached without actually attaching it
    pub fn can_attach(&mut self, device_type: DeviceType, env_id: Uuid, user_id: Option<&str>) -> (bool, Option<String>) {
        let result = self.policy_engine.check_access(device_type, Some(env_id), user_id);
        (result.allowed, if !result.allowed { Some(result.user_message()) } else { None })
    }

    /// Get audit log from policy engine
    pub fn get_audit_log(&self) -> Vec<crate::policy::PolicyAuditEvent> {
        self.policy_engine.get_audit_log()
    }

    /// Export audit log as JSON string
    pub fn export_audit_log(&self) -> String {
        self.policy_engine.export_audit_log()
    }

    /// Get compliance report for an environment
    pub fn get_compliance_report(&self, env_id: Uuid) -> ComplianceReport {
        let audit_events = self.policy_engine.get_audit_log_for_env(env_id);
        let policies = self.policy_engine.list_rules();
        let attached_devices = self.list_for_env(env_id);

        ComplianceReport {
            env_id,
            generated_at: chrono::Utc::now(),
            total_devices: self.count(),
            attached_devices: attached_devices.len(),
            policy_rules: policies.len(),
            audit_events_count: audit_events.len(),
            blocked_attempts: audit_events
                .iter()
                .filter(|e| e.decision == crate::policy::AccessDecision::Block)
                .count(),
            audit_events,
        }
    }

    /// Detach all devices from an environment
    pub fn detach_all_for_env(&mut self, env_id: Uuid) -> Vec<Uuid> {
        let mut detached = Vec::new();
        for device in self.devices.values_mut() {
            if device.attached_to_env == Some(env_id) {
                device.detach();
                detached.push(device.id);
            }
        }
        detached
    }

    /// Set device error state
    pub fn set_error(&mut self, device_id: Uuid, message: String) -> Result<()> {
        let device = self
            .devices
            .get_mut(&device_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(device_id.to_string()))?;

        device.set_error(message);
        Ok(())
    }

    /// Clear device error state
    pub fn clear_error(&mut self, device_id: Uuid) -> Result<()> {
        let device = self
            .devices
            .get_mut(&device_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(device_id.to_string()))?;

        device.clear_error();
        Ok(())
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::DeviceStatus;

    #[test]
    fn test_register_device() {
        let mut manager = DeviceManager::new();
        let device = Device::new(
            "Arduino".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );

        let id = manager.register(device).unwrap();
        assert!(manager.get(id).is_some());
    }

    #[test]
    fn test_duplicate_path() {
        let mut manager = DeviceManager::new();
        let device1 = Device::new(
            "Device1".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );
        let device2 = Device::new(
            "Device2".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );

        manager.register(device1).unwrap();
        let result = manager.register(device2);
        assert!(result.is_err());
    }

    #[test]
    fn test_attach_detach() {
        let mut manager = DeviceManager::new();
        let device = Device::new(
            "Test".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );
        let device_id = device.id;
        let env_id = Uuid::new_v4();

        manager.register(device).unwrap();
        manager.attach(device_id, env_id, None).unwrap();

        let attached = manager.get(device_id).unwrap();
        assert!(attached.is_attached());

        manager.detach(device_id).unwrap();
        let detached = manager.get(device_id).unwrap();
        assert!(!detached.is_attached());
    }

    #[test]
    fn test_list_by_type() {
        let mut manager = DeviceManager::new();

        let serial = Device::new(
            "Serial".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );
        let camera = Device::new(
            "Camera".to_string(),
            DeviceType::Camera,
            PathBuf::from("/dev/video0"),
        );

        manager.register(serial).unwrap();
        manager.register(camera).unwrap();

        let serials = manager.list_by_type(DeviceType::Serial);
        assert_eq!(serials.len(), 1);

        let cameras = manager.list_by_type(DeviceType::Camera);
        assert_eq!(cameras.len(), 1);
    }

    #[test]
    fn test_device_error_tracking() {
        let mut manager = DeviceManager::new();
        let device = Device::new(
            "Test".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );
        let device_id = device.id;

        manager.register(device).unwrap();
        manager.set_error(device_id, "Connection failed".to_string()).unwrap();

        let device = manager.get(device_id).unwrap();
        assert_eq!(device.status, DeviceStatus::Error);

        manager.clear_error(device_id).unwrap();
        let device = manager.get(device_id).unwrap();
        assert_eq!(device.status, DeviceStatus::Available);
    }
}
