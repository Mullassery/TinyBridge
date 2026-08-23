/// Phase 4.0.2: Policy Engine & Access Control Integration Tests
///
/// Complete testing of hierarchical policy evaluation and enforcement
#[cfg(test)]
mod policy_engine_hierarchy_tests {
    use crate::device_access_control::DeviceAccessController;
    use crate::device_manager::{Device, DeviceManager, DeviceType};
    use crate::policy_engine::{
        AccessDecision, DevicePolicy, PolicyContext, PolicyDecision, PolicyEngine,
    };
    use std::collections::HashSet;

    #[test]
    fn test_policy_hierarchy_platform_base() {
        let mut engine = PolicyEngine::new();

        // Platform allows USB
        engine.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow),
        );

        let ctx = PolicyContext::new("env1".to_string(), "usb-1".to_string(), DeviceType::Usb);
        let decision = engine.evaluate(&ctx);

        assert!(decision.allowed);
        assert_eq!(decision.policy_level, "platform");
    }

    #[test]
    fn test_policy_hierarchy_project_override() {
        let mut engine = PolicyEngine::new();

        // Platform denies Serial
        engine.set_platform_policy(
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Deny),
        );

        // Project allows Serial
        engine.set_project_policy(
            "robotics".to_string(),
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Allow),
        );

        let ctx = PolicyContext::new(
            "robot-env".to_string(),
            "serial-1".to_string(),
            DeviceType::Serial,
        )
        .with_project("robotics".to_string());
        let decision = engine.evaluate(&ctx);

        assert!(decision.allowed);
        assert_eq!(decision.policy_level, "project");
    }

    #[test]
    fn test_policy_hierarchy_environment_priority() {
        let mut engine = PolicyEngine::new();

        // Platform allows
        engine.set_platform_policy(
            DeviceType::Camera,
            DevicePolicy::new(DeviceType::Camera, AccessDecision::Allow),
        );

        // Project denies
        engine.set_project_policy(
            "strict-project".to_string(),
            DeviceType::Camera,
            DevicePolicy::new(DeviceType::Camera, AccessDecision::Deny),
        );

        // Environment allows (highest priority)
        engine.set_environment_policy(
            "special-env".to_string(),
            DeviceType::Camera,
            DevicePolicy::new(DeviceType::Camera, AccessDecision::Allow),
        );

        let ctx = PolicyContext::new(
            "special-env".to_string(),
            "camera-1".to_string(),
            DeviceType::Camera,
        )
        .with_project("strict-project".to_string());
        let decision = engine.evaluate(&ctx);

        assert!(decision.allowed);
        assert_eq!(decision.policy_level, "environment");
    }

    #[test]
    fn test_policy_whitelist_enforcement() {
        let mut engine = PolicyEngine::new();

        let mut whitelist = HashSet::new();
        whitelist.insert("safe-device-1".to_string());
        whitelist.insert("safe-device-2".to_string());

        engine.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow).with_whitelist(whitelist),
        );

        // Whitelisted should pass
        let ctx1 = PolicyContext::new(
            "env1".to_string(),
            "safe-device-1".to_string(),
            DeviceType::Usb,
        );
        assert!(engine.evaluate(&ctx1).allowed);

        // Not whitelisted should fail
        let ctx2 = PolicyContext::new(
            "env1".to_string(),
            "dangerous-device".to_string(),
            DeviceType::Usb,
        );
        assert!(!engine.evaluate(&ctx2).allowed);
    }

    #[test]
    fn test_policy_blacklist_enforcement() {
        let mut engine = PolicyEngine::new();

        let mut blacklist = HashSet::new();
        blacklist.insert("dangerous-device".to_string());

        engine.set_platform_policy(
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Allow).with_blacklist(blacklist),
        );

        // Blacklisted should fail
        let ctx1 = PolicyContext::new(
            "env1".to_string(),
            "dangerous-device".to_string(),
            DeviceType::Serial,
        );
        assert!(!engine.evaluate(&ctx1).allowed);

        // Non-blacklisted should pass
        let ctx2 = PolicyContext::new(
            "env1".to_string(),
            "safe-device".to_string(),
            DeviceType::Serial,
        );
        assert!(engine.evaluate(&ctx2).allowed);
    }

    #[test]
    fn test_policy_all_device_types() {
        let mut engine = PolicyEngine::new();

        // Allow all types
        for device_type in &[
            DeviceType::Usb,
            DeviceType::Serial,
            DeviceType::Camera,
            DeviceType::Audio,
        ] {
            engine.set_platform_policy(
                *device_type,
                DevicePolicy::new(*device_type, AccessDecision::Allow),
            );
        }

        for device_type in &[
            DeviceType::Usb,
            DeviceType::Serial,
            DeviceType::Camera,
            DeviceType::Audio,
        ] {
            let ctx = PolicyContext::new("env1".to_string(), "dev-1".to_string(), *device_type);
            assert!(engine.evaluate(&ctx).allowed);
        }
    }

    #[test]
    fn test_policy_default_deny() {
        let engine = PolicyEngine::new();

        let ctx = PolicyContext::new("env1".to_string(), "dev-1".to_string(), DeviceType::Usb);
        let decision = engine.evaluate(&ctx);

        assert!(!decision.allowed);
        assert_eq!(decision.reason, "No matching policy (default deny)");
    }
}

#[cfg(test)]
mod access_control_integration_tests {
    use crate::device_access_control::DeviceAccessController;
    use crate::device_manager::{Device, DeviceManager, DeviceType};
    use crate::policy_engine::{AccessDecision, DevicePolicy, PolicyEngine};
    use std::collections::HashSet;

    #[test]
    fn test_access_control_policy_allow() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        // Policy allows USB
        policy.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow),
        );

        // Register device
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
        let decision =
            controller.request_device_access("ml-env".to_string(), device_id.clone(), None, None);

        assert!(decision.allowed);
        assert!(decision.passthrough_result.is_some());
        assert!(decision.error.is_none());
    }

    #[test]
    fn test_access_control_policy_deny() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        // Policy denies Camera
        policy.set_platform_policy(
            DeviceType::Camera,
            DevicePolicy::new(DeviceType::Camera, AccessDecision::Deny)
                .with_reason("Security policy".to_string()),
        );

        // Register device
        let device = Device::new(
            DeviceType::Camera,
            "Apple".to_string(),
            "FaceTime Camera".to_string(),
            "/dev/camera0".to_string(),
        );
        let device_id = device.id.clone();
        device_mgr.register_device(device);

        let mut controller = DeviceAccessController::new(policy, device_mgr);

        // Try to request access
        let decision =
            controller.request_device_access("suspicious-env".to_string(), device_id, None, None);

        assert!(!decision.allowed);
        assert!(decision.error.is_some());
        assert!(decision.error.unwrap().contains("denied"));
    }

    #[test]
    fn test_access_control_whitelist() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        let mut whitelist = HashSet::new();
        whitelist.insert("approved-serial".to_string());

        // Platform allows Serial with whitelist
        policy.set_platform_policy(
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Allow).with_whitelist(whitelist),
        );

        // Register devices
        let device1 = Device::new(
            DeviceType::Serial,
            "Arduino".to_string(),
            "Serial 1".to_string(),
            "/dev/serial1".to_string(),
        );
        let id1 = "approved-serial".to_string();
        device_mgr.register_device(device1);

        let device2 = Device::new(
            DeviceType::Serial,
            "Arduino".to_string(),
            "Serial 2".to_string(),
            "/dev/serial2".to_string(),
        );
        let id2 = device2.id.clone();
        device_mgr.register_device(device2);

        let mut controller = DeviceAccessController::new(policy, device_mgr);

        // Whitelisted should succeed
        let decision1 = controller.request_device_access("robot-env".to_string(), id1, None, None);
        assert!(decision1.allowed);

        // Non-whitelisted should fail
        let decision2 = controller.request_device_access("robot-env".to_string(), id2, None, None);
        assert!(!decision2.allowed);
    }

    #[test]
    fn test_access_control_project_override() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        // Platform denies Audio
        policy.set_platform_policy(
            DeviceType::Audio,
            DevicePolicy::new(DeviceType::Audio, AccessDecision::Deny),
        );

        // But AI project allows Audio
        policy.set_project_policy(
            "ai-project".to_string(),
            DeviceType::Audio,
            DevicePolicy::new(DeviceType::Audio, AccessDecision::Allow),
        );

        let device = Device::new(
            DeviceType::Audio,
            "Apple".to_string(),
            "Built-in Microphone".to_string(),
            "/dev/audio0".to_string(),
        );
        let device_id = device.id.clone();
        device_mgr.register_device(device);

        let mut controller = DeviceAccessController::new(policy, device_mgr);

        // General env - denied
        let decision1 = controller.request_device_access(
            "general-env".to_string(),
            device_id.clone(),
            None,
            None,
        );
        assert!(!decision1.allowed);

        // AI project - allowed
        let decision2 = controller.request_device_access(
            "ai-env".to_string(),
            device_id,
            Some("ai-project".to_string()),
            None,
        );
        assert!(decision2.allowed);
    }

    #[test]
    fn test_access_control_device_allocation() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        policy.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow),
        );

        let device = Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/usb0".to_string(),
        );
        let device_id = device.id.clone();
        device_mgr.register_device(device);

        let mut controller = DeviceAccessController::new(policy, device_mgr);

        // First env gets the device
        let decision1 =
            controller.request_device_access("env1".to_string(), device_id.clone(), None, None);
        assert!(decision1.allowed);

        // Second env should fail (device already allocated)
        let decision2 =
            controller.request_device_access("env2".to_string(), device_id.clone(), None, None);
        assert!(!decision2.allowed);

        // But after release, second env should succeed
        assert!(controller.release_device_access(&device_id, "env1"));
        let decision3 = controller.request_device_access("env2".to_string(), device_id, None, None);
        assert!(decision3.allowed);
    }

    #[test]
    fn test_access_control_available_devices() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        // Allow USB, deny Serial
        policy.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow),
        );
        policy.set_platform_policy(
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Deny),
        );

        // Register mixed devices
        device_mgr.register_device(Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "USB 1".to_string(),
            "/dev/usb1".to_string(),
        ));
        device_mgr.register_device(Device::new(
            DeviceType::Serial,
            "Vendor".to_string(),
            "Serial 1".to_string(),
            "/dev/serial1".to_string(),
        ));

        let controller = DeviceAccessController::new(policy, device_mgr);

        // Should only see USB
        let available = controller.get_available_devices_for_environment("env1", None, None);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].device_type, DeviceType::Usb);
    }

    #[test]
    fn test_access_summary_generation() {
        let policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        // Register devices of each type
        for i in 0..2 {
            device_mgr.register_device(Device::new(
                DeviceType::Usb,
                "Vendor".to_string(),
                format!("USB {}", i),
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
        let summary = controller.get_access_summary("test-env", None);

        assert_eq!(summary.total_devices, 5);
        assert_eq!(summary.usb.1, 2);
        assert_eq!(summary.serial.1, 3);
        assert_eq!(summary.overall_access_percentage(), 100.0); // All allowed by default
    }

    #[test]
    fn test_multi_level_policy_cascade() {
        let mut policy = PolicyEngine::new();
        let mut device_mgr = DeviceManager::new();

        // Platform: Allow Usb, Deny Serial
        policy.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow),
        );
        policy.set_platform_policy(
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Deny),
        );

        // Project: Allow Serial
        policy.set_project_policy(
            "robotics".to_string(),
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Allow),
        );

        // Environment: Deny Serial (override project)
        policy.set_environment_policy(
            "restricted-env".to_string(),
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Deny),
        );

        device_mgr.register_device(Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "USB".to_string(),
            "/dev/usb0".to_string(),
        ));
        device_mgr.register_device(Device::new(
            DeviceType::Serial,
            "Vendor".to_string(),
            "Serial".to_string(),
            "/dev/serial0".to_string(),
        ));

        let controller = DeviceAccessController::new(policy, device_mgr);

        // Robotics project: USB allowed, Serial allowed
        let available1 =
            controller.get_available_devices_for_environment("robot-env", Some("robotics"), None);
        assert_eq!(available1.len(), 2);

        // Restricted env in robotics: USB allowed, Serial denied
        let available2 = controller.get_available_devices_for_environment(
            "restricted-env",
            Some("robotics"),
            None,
        );
        assert_eq!(available2.len(), 1);
        assert_eq!(available2[0].device_type, DeviceType::Usb);
    }
}
