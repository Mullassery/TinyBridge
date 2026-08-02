/// Device access control layer
///
/// Integrates policy engine with device manager to enforce access decisions.
/// Provides the enforcement point where policies are checked before passthrough.

use crate::device_manager::{Device, DeviceManager, DeviceType, PassthroughRequest, PassthroughResult};
use crate::policy_engine::{PolicyContext, PolicyDecision, PolicyEngine};
use serde::{Deserialize, Serialize};

/// Access control result with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlDecision {
    /// Allowed or denied
    pub allowed: bool,
    /// Policy decision (reasoning)
    pub policy_decision: PolicyDecision,
    /// Passthrough result (if allowed)
    pub passthrough_result: Option<PassthroughResult>,
    /// Error message (if denied)
    pub error: Option<String>,
}

impl AccessControlDecision {
    pub fn allowed(policy: PolicyDecision, passthrough: PassthroughResult) -> Self {
        AccessControlDecision {
            allowed: true,
            policy_decision: policy,
            passthrough_result: Some(passthrough),
            error: None,
        }
    }

    pub fn denied(policy: PolicyDecision, error: String) -> Self {
        AccessControlDecision {
            allowed: false,
            policy_decision: policy,
            passthrough_result: None,
            error: Some(error),
        }
    }
}

/// Device access controller
pub struct DeviceAccessController {
    policy_engine: PolicyEngine,
    device_manager: DeviceManager,
}

impl DeviceAccessController {
    pub fn new(policy_engine: PolicyEngine, device_manager: DeviceManager) -> Self {
        DeviceAccessController {
            policy_engine,
            device_manager,
        }
    }

    /// Request device access with policy enforcement
    pub fn request_device_access(
        &mut self,
        environment: String,
        device_id: String,
        project: Option<String>,
        user_id: Option<String>,
    ) -> AccessControlDecision {
        // Get device from manager
        let device = match self.device_manager.get_device(&device_id) {
            Some(d) => d.clone(),
            None => {
                return AccessControlDecision::denied(
                    PolicyDecision::deny("Device not found", "system"),
                    format!("Device {} not found in inventory", device_id),
                )
            }
        };

        // Build policy context
        let mut ctx = PolicyContext::new(environment.clone(), device_id.clone(), device.device_type);
        if let Some(proj) = project {
            ctx = ctx.with_project(proj);
        }
        if let Some(user) = user_id {
            ctx = ctx.with_user(user);
        }

        // Evaluate policy
        let policy_decision = self.policy_engine.evaluate(&ctx);

        // If policy denies, stop here
        if !policy_decision.allowed {
            let error_msg = format!(
                "Device {} access denied: {}",
                device_id, policy_decision.reason
            );
            return AccessControlDecision::denied(policy_decision, error_msg);
        }

        // Policy allows - now try to allocate device
        let request = PassthroughRequest::new(environment, device_id.clone());
        let passthrough_result = self.device_manager.request_passthrough(request);

        if passthrough_result.success {
            AccessControlDecision::allowed(policy_decision, passthrough_result)
        } else {
            AccessControlDecision::denied(
                policy_decision,
                format!("Device allocation failed: {}", passthrough_result.message),
            )
        }
    }

    /// Release device access
    pub fn release_device_access(&mut self, device_id: &str, environment: &str) -> bool {
        self.device_manager.release_device(device_id, environment)
    }

    /// Get available devices for an environment (filtered by policy)
    pub fn get_available_devices_for_environment(
        &self,
        environment: &str,
        project: Option<&str>,
        user_id: Option<&str>,
    ) -> Vec<Device> {
        self.device_manager
            .available_devices()
            .into_iter()
            .filter(|device| {
                let mut ctx = PolicyContext::new(
                    environment.to_string(),
                    device.id.clone(),
                    device.device_type,
                );
                if let Some(proj) = project {
                    ctx = ctx.with_project(proj.to_string());
                }
                if let Some(user) = user_id {
                    ctx = ctx.with_user(user.to_string());
                }

                let decision = self.policy_engine.evaluate(&ctx);
                decision.allowed
            })
            .cloned()
            .collect()
    }

    /// Get devices by type and policy
    pub fn get_devices_by_type_and_policy(
        &self,
        device_type: DeviceType,
        environment: &str,
        project: Option<&str>,
    ) -> Vec<Device> {
        self.device_manager
            .devices_by_type(device_type)
            .into_iter()
            .filter(|device| {
                let mut ctx = PolicyContext::new(
                    environment.to_string(),
                    device.id.clone(),
                    device.device_type,
                );
                if let Some(proj) = project {
                    ctx = ctx.with_project(proj.to_string());
                }

                let decision = self.policy_engine.evaluate(&ctx);
                decision.allowed
            })
            .cloned()
            .collect()
    }

    /// Get access summary for an environment
    pub fn get_access_summary(
        &self,
        environment: &str,
        project: Option<&str>,
    ) -> AccessSummary {
        let all_devices = self.device_manager.available_devices();
        let accessible_devices = self.get_available_devices_for_environment(environment, project, None);

        let usb_total = self.device_manager.count_by_type(DeviceType::Usb);
        let usb_accessible = self
            .get_devices_by_type_and_policy(DeviceType::Usb, environment, project)
            .len();

        let serial_total = self.device_manager.count_by_type(DeviceType::Serial);
        let serial_accessible =
            self
                .get_devices_by_type_and_policy(DeviceType::Serial, environment, project)
                .len();

        let camera_total = self.device_manager.count_by_type(DeviceType::Camera);
        let camera_accessible =
            self
                .get_devices_by_type_and_policy(DeviceType::Camera, environment, project)
                .len();

        let audio_total = self.device_manager.count_by_type(DeviceType::Audio);
        let audio_accessible = self
            .get_devices_by_type_and_policy(DeviceType::Audio, environment, project)
            .len();

        AccessSummary {
            environment: environment.to_string(),
            project: project.map(|s| s.to_string()),
            total_devices: all_devices.len(),
            accessible_devices: accessible_devices.len(),
            usb: (usb_accessible, usb_total),
            serial: (serial_accessible, serial_total),
            camera: (camera_accessible, camera_total),
            audio: (audio_accessible, audio_total),
        }
    }

    pub fn get_policy_engine(&self) -> &PolicyEngine {
        &self.policy_engine
    }

    pub fn get_device_manager(&self) -> &DeviceManager {
        &self.device_manager
    }
}

/// Access summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessSummary {
    pub environment: String,
    pub project: Option<String>,
    pub total_devices: usize,
    pub accessible_devices: usize,
    pub usb: (usize, usize), // (accessible, total)
    pub serial: (usize, usize),
    pub camera: (usize, usize),
    pub audio: (usize, usize),
}

impl AccessSummary {
    pub fn usb_percentage(&self) -> f64 {
        if self.usb.1 == 0 {
            0.0
        } else {
            (self.usb.0 as f64 / self.usb.1 as f64) * 100.0
        }
    }

    pub fn serial_percentage(&self) -> f64 {
        if self.serial.1 == 0 {
            0.0
        } else {
            (self.serial.0 as f64 / self.serial.1 as f64) * 100.0
        }
    }

    pub fn camera_percentage(&self) -> f64 {
        if self.camera.1 == 0 {
            0.0
        } else {
            (self.camera.0 as f64 / self.camera.1 as f64) * 100.0
        }
    }

    pub fn audio_percentage(&self) -> f64 {
        if self.audio.1 == 0 {
            0.0
        } else {
            (self.audio.0 as f64 / self.audio.1 as f64) * 100.0
        }
    }

    pub fn overall_access_percentage(&self) -> f64 {
        if self.total_devices == 0 {
            0.0
        } else {
            (self.accessible_devices as f64 / self.total_devices as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_manager::Device;
    use crate::policy_engine::{AccessDecision, DevicePolicy, PolicyEngine};

    #[test]
    fn test_access_controller_initialization() {
        let policy = PolicyEngine::new();
        let device_mgr = DeviceManager::new();
        let controller = DeviceAccessController::new(policy, device_mgr);

        assert!(controller.get_available_devices_for_environment("env1", None, None).is_empty());
    }

    #[test]
    fn test_request_with_policy_allow() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        // Setup policy
        policy.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow),
        );

        // Setup device
        let device = Device::new(
            DeviceType::Usb,
            "Apple".to_string(),
            "USB Hub".to_string(),
            "/dev/usb0".to_string(),
        );
        let device_id = device.id.clone();
        device_mgr.register_device(device);

        let mut controller = DeviceAccessController::new(policy, device_mgr);

        // Request access
        let decision = controller.request_device_access(
            "ml-env".to_string(),
            device_id,
            None,
            None,
        );

        assert!(decision.allowed);
        assert!(decision.passthrough_result.is_some());
        assert!(decision.error.is_none());
    }

    #[test]
    fn test_request_with_policy_deny() {
        let mut policy = PolicyEngine::new();
        let device_mgr = DeviceManager::new();

        // Setup policy - deny all USB
        policy.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Deny)
                .with_reason("USB disabled".to_string()),
        );

        let mut controller = DeviceAccessController::new(policy, device_mgr);

        // Request access (but no device in manager)
        let decision = controller.request_device_access(
            "ml-env".to_string(),
            "usb-123".to_string(),
            None,
            None,
        );

        assert!(!decision.allowed);
        assert!(decision.error.is_some());
    }

    #[test]
    fn test_access_summary_calculation() {
        let policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        // Register devices
        for i in 0..2 {
            device_mgr.register_device(Device::new(
                DeviceType::Usb,
                "Vendor".to_string(),
                format!("Device {}", i),
                format!("/dev/usb{}", i),
            ));
        }

        for i in 0..3 {
            device_mgr.register_device(Device::new(
                DeviceType::Serial,
                "Vendor".to_string(),
                format!("Serial {}", i),
                format!("/dev/serial{}", i),
            ));
        }

        let controller = DeviceAccessController::new(policy, device_mgr);
        let summary = controller.get_access_summary("env1", None);

        assert_eq!(summary.total_devices, 5);
        assert_eq!(summary.usb.1, 2);
        assert_eq!(summary.serial.1, 3);
    }

    #[test]
    fn test_release_device_access() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        policy.set_platform_policy(
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Allow),
        );

        let device = Device::new(
            DeviceType::Serial,
            "Arduino".to_string(),
            "USB Serial".to_string(),
            "/dev/serial0".to_string(),
        );
        let device_id = device.id.clone();
        device_mgr.register_device(device);

        let mut controller = DeviceAccessController::new(policy, device_mgr);

        // Request access
        let decision = controller.request_device_access(
            "robot-env".to_string(),
            device_id.clone(),
            None,
            None,
        );
        assert!(decision.allowed);

        // Release access
        assert!(controller.release_device_access(&device_id, "robot-env"));
    }

    #[test]
    fn test_project_specific_policies() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        // Platform denies camera
        policy.set_platform_policy(
            DeviceType::Camera,
            DevicePolicy::new(DeviceType::Camera, AccessDecision::Deny),
        );

        // But robotics project allows
        policy.set_project_policy(
            "robotics".to_string(),
            DeviceType::Camera,
            DevicePolicy::new(DeviceType::Camera, AccessDecision::Allow),
        );

        let device = Device::new(
            DeviceType::Camera,
            "Apple".to_string(),
            "Camera".to_string(),
            "/dev/camera0".to_string(),
        );
        let device_id = device.id.clone();
        device_mgr.register_device(device);

        let mut controller = DeviceAccessController::new(policy, device_mgr);

        // General env - should be denied
        let decision1 = controller.request_device_access(
            "general-env".to_string(),
            device_id.clone(),
            None,
            None,
        );
        assert!(!decision1.allowed);

        // Robotics project env - should be allowed
        let decision2 = controller.request_device_access(
            "robot-env".to_string(),
            device_id.clone(),
            Some("robotics".to_string()),
            None,
        );
        assert!(decision2.allowed);
    }

    #[test]
    fn test_access_percentage_calculations() {
        let mut summary = AccessSummary {
            environment: "env1".to_string(),
            project: None,
            total_devices: 10,
            accessible_devices: 5,
            usb: (2, 4),
            serial: (3, 3),
            camera: (0, 2),
            audio: (0, 1),
        };

        assert_eq!(summary.overall_access_percentage(), 50.0);
        assert_eq!(summary.usb_percentage(), 50.0);
        assert_eq!(summary.serial_percentage(), 100.0);
        assert_eq!(summary.camera_percentage(), 0.0);
    }
}
