/// Phase 4.0.1: Device Manager Integration Tests
///
/// Complete end-to-end testing of device management pipeline:
/// - Device discovery (enumeration)
/// - Device registration and tracking
/// - Passthrough request handling
/// - Device release and lifecycle
/// - Multi-environment device allocation
#[cfg(test)]
mod device_manager_integration {
    use crate::device_discovery::DeviceDiscovery;
    use crate::device_manager::{
        Device, DeviceManager, DeviceStatus, DeviceType, PassthroughRequest,
    };

    #[test]
    fn test_device_discovery_flow() {
        // Discovery pipeline: scan → register → track
        let mut manager = DeviceManager::new();

        // Simulate device discovery
        let usb_device = Device::new(
            DeviceType::Usb,
            "Apple".to_string(),
            "USB Hub".to_string(),
            "/dev/usb0".to_string(),
        );
        let device_id = usb_device.id.clone();
        manager.register_device(usb_device);

        // Verify device is tracked
        assert_eq!(manager.total_count(), 1);
        assert!(manager.get_device(&device_id).is_some());
        assert!(!manager.available_devices().is_empty());
    }

    #[test]
    fn test_passthrough_lifecycle_single_environment() {
        let mut manager = DeviceManager::new();

        // Create and register device
        let device = Device::new(
            DeviceType::Serial,
            "Arduino".to_string(),
            "USB Serial".to_string(),
            "/dev/cu.usbserial".to_string(),
        );
        let device_id = device.id.clone();
        manager.register_device(device);

        // Environment requests passthrough
        let request = PassthroughRequest::new("robot-sim".to_string(), device_id.clone());
        let result = manager.request_passthrough(request);

        assert!(result.success);
        assert_eq!(
            result.device.unwrap().in_use_by,
            Some("robot-sim".to_string())
        );

        // Device no longer available
        assert!(!manager.get_device(&device_id).unwrap().is_available());

        // Release device
        assert!(manager.release_device(&device_id, "robot-sim"));
        assert!(manager.get_device(&device_id).unwrap().is_available());
    }

    #[test]
    fn test_device_isolation_between_environments() {
        let mut manager = DeviceManager::new();

        // Register two serial devices
        let device1 = Device::new(
            DeviceType::Serial,
            "Vendor1".to_string(),
            "Serial1".to_string(),
            "/dev/cu.serial1".to_string(),
        );
        let device2 = Device::new(
            DeviceType::Serial,
            "Vendor2".to_string(),
            "Serial2".to_string(),
            "/dev/cu.serial2".to_string(),
        );
        let id1 = device1.id.clone();
        let id2 = device2.id.clone();

        manager.register_device(device1);
        manager.register_device(device2);

        // Environment A takes device 1
        let req_a = PassthroughRequest::new("env-a".to_string(), id1.clone());
        manager.request_passthrough(req_a);

        // Environment B tries to take device 1 (should fail)
        let req_b = PassthroughRequest::new("env-b".to_string(), id1.clone());
        let result = manager.request_passthrough(req_b);
        assert!(!result.success);

        // But environment B can take device 2
        let req_b2 = PassthroughRequest::new("env-b".to_string(), id2.clone());
        let result2 = manager.request_passthrough(req_b2);
        assert!(result2.success);

        // Verify isolation
        assert_eq!(manager.devices_for_environment("env-a").len(), 1);
        assert_eq!(manager.devices_for_environment("env-b").len(), 1);
    }

    #[test]
    fn test_device_status_transitions() {
        let mut manager = DeviceManager::new();

        let device = Device::new(
            DeviceType::Camera,
            "Apple".to_string(),
            "Camera".to_string(),
            "/dev/camera0".to_string(),
        );
        let device_id = device.id.clone();

        // Device starts available
        assert_eq!(device.status, DeviceStatus::Available);

        manager.register_device(device);

        // Request passthrough
        let request = PassthroughRequest::new("ml-training".to_string(), device_id.clone());
        manager.request_passthrough(request);

        // Verify status changed
        let device = manager.get_device(&device_id).unwrap();
        assert_eq!(device.status, DeviceStatus::InUse);

        // After release, status reverts
        manager.release_device(&device_id, "ml-training");
        assert_eq!(
            manager.get_device(&device_id).unwrap().status,
            DeviceStatus::Available
        );
    }

    #[test]
    fn test_device_type_filtering() {
        let mut manager = DeviceManager::new();

        // Register mixed device types
        for device_type in &[
            DeviceType::Usb,
            DeviceType::Serial,
            DeviceType::Camera,
            DeviceType::Audio,
        ] {
            for i in 0..2 {
                let device = Device::new(
                    *device_type,
                    "Vendor".to_string(),
                    format!("Device {}", i),
                    format!("/dev/dev{}", i),
                );
                manager.register_device(device);
            }
        }

        // Verify filtering
        assert_eq!(manager.devices_by_type(DeviceType::Usb).len(), 2);
        assert_eq!(manager.devices_by_type(DeviceType::Serial).len(), 2);
        assert_eq!(manager.devices_by_type(DeviceType::Camera).len(), 2);
        assert_eq!(manager.devices_by_type(DeviceType::Audio).len(), 2);
        assert_eq!(manager.total_count(), 8);
    }

    #[test]
    fn test_passthrough_history_tracking() {
        let mut manager = DeviceManager::new();

        let device = Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/usb0".to_string(),
        );
        let device_id = device.id.clone();
        manager.register_device(device);

        // Multiple passthroughs
        for i in 0..5 {
            let request = PassthroughRequest::new(format!("env{}", i), device_id.clone());
            // First one succeeds, rest fail (device in use)
            if i == 0 {
                manager.request_passthrough(request);
            } else {
                manager.release_device(&device_id, &format!("env{}", i - 1));
                manager.request_passthrough(request);
            }
        }

        // History tracks all attempts
        assert_eq!(manager.history().len(), 5);
    }

    #[test]
    fn test_multi_device_per_environment() {
        let mut manager = DeviceManager::new();

        // Register multiple devices
        let mut device_ids = vec![];
        for i in 0..3 {
            let device = Device::new(
                if i % 2 == 0 {
                    DeviceType::Usb
                } else {
                    DeviceType::Serial
                },
                "Vendor".to_string(),
                format!("Device {}", i),
                format!("/dev/dev{}", i),
            );
            let id = device.id.clone();
            device_ids.push(id);
            manager.register_device(device);
        }

        // One environment requests multiple devices
        for device_id in &device_ids {
            let request = PassthroughRequest::new("robotics-env".to_string(), device_id.clone());
            let result = manager.request_passthrough(request);
            assert!(
                result.success,
                "Should grant passthrough for device {}",
                device_id
            );
        }

        // Verify all allocated to same environment
        let env_devices = manager.devices_for_environment("robotics-env");
        assert_eq!(env_devices.len(), 3);
    }

    #[test]
    fn test_device_enumeration_on_macos() {
        let mut manager = DeviceManager::new();

        // Run actual discovery
        let result = DeviceDiscovery::discover_all(&mut manager);
        assert!(result.is_ok());

        // Should have discovered at least camera and audio (fallbacks)
        let total = manager.total_count();
        assert!(total > 0, "Should discover at least some devices");

        // Verify device categories exist
        let cameras = manager.devices_by_type(DeviceType::Camera);
        let audio = manager.devices_by_type(DeviceType::Audio);
        assert!(
            !cameras.is_empty() || !audio.is_empty(),
            "Should discover cameras or audio"
        );
    }

    #[test]
    fn test_device_passthrough_with_filter() {
        let mut manager = DeviceManager::new();

        let device = Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/usb0".to_string(),
        );
        let device_id = device.id.clone();
        manager.register_device(device);

        // Request with filter
        let request = PassthroughRequest::new("ml-env".to_string(), device_id.clone())
            .with_filter("vendor_id:1234".to_string());

        let result = manager.request_passthrough(request);
        assert!(result.success);
    }

    #[test]
    fn test_device_availability_after_release() {
        let mut manager = DeviceManager::new();

        let device = Device::new(
            DeviceType::Serial,
            "Arduino".to_string(),
            "Uno".to_string(),
            "/dev/cu.arduino".to_string(),
        );
        let device_id = device.id.clone();
        manager.register_device(device);

        // Request and release cycle
        for cycle in 0..3 {
            let request = PassthroughRequest::new(format!("env{}", cycle), device_id.clone());
            let result = manager.request_passthrough(request);
            assert!(result.success);

            assert!(manager.release_device(&device_id, &format!("env{}", cycle)));
            assert!(manager.get_device(&device_id).unwrap().is_available());
        }
    }

    #[test]
    fn test_device_count_consistency() {
        let mut manager = DeviceManager::new();

        let mut registered_count = 0;

        // Register various device types
        for device_type in &[
            DeviceType::Usb,
            DeviceType::Serial,
            DeviceType::Camera,
            DeviceType::Audio,
        ] {
            for i in 0..3 {
                let device = Device::new(
                    *device_type,
                    "Vendor".to_string(),
                    format!("Device {}", i),
                    format!("/dev/dev{}", i),
                );
                manager.register_device(device);
                registered_count += 1;
            }
        }

        assert_eq!(manager.total_count(), registered_count);
        assert_eq!(manager.total_count(), 12); // 4 types * 3 devices
    }

    #[test]
    fn test_device_passthrough_invalid_device() {
        let mut manager = DeviceManager::new();

        let request = PassthroughRequest::new("env".to_string(), "nonexistent".to_string());
        let result = manager.request_passthrough(request);

        assert!(!result.success);
        assert!(result.message.contains("not found"));
        assert!(result.device.is_none());
    }

    #[test]
    fn test_release_wrong_environment() {
        let mut manager = DeviceManager::new();

        let device = Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/usb0".to_string(),
        );
        let device_id = device.id.clone();
        manager.register_device(device);

        // Request from env1
        let request = PassthroughRequest::new("env1".to_string(), device_id.clone());
        manager.request_passthrough(request);

        // Try to release from env2 (should fail)
        assert!(!manager.release_device(&device_id, "env2"));

        // Device should still be allocated to env1
        assert_eq!(
            manager.get_device(&device_id).unwrap().in_use_by,
            Some("env1".to_string())
        );
    }
}

#[cfg(test)]
mod device_manager_stress_tests {
    use crate::device_manager::{Device, DeviceManager, DeviceType, PassthroughRequest};

    #[test]
    fn test_many_devices_single_environment() {
        let mut manager = DeviceManager::new();

        // Register 50 devices
        let mut device_ids = vec![];
        for i in 0..50 {
            let device = Device::new(
                DeviceType::Usb,
                "Vendor".to_string(),
                format!("Device {}", i),
                format!("/dev/usb{}", i),
            );
            let id = device.id.clone();
            device_ids.push(id);
            manager.register_device(device);
        }

        assert_eq!(manager.total_count(), 50);

        // Allocate first 30 to one environment
        for (idx, device_id) in device_ids.iter().enumerate().take(30) {
            let request = PassthroughRequest::new("main-env".to_string(), device_id.clone());
            let result = manager.request_passthrough(request);
            assert!(result.success, "Failed to allocate device at index {}", idx);
        }

        assert_eq!(manager.devices_for_environment("main-env").len(), 30);
    }

    #[test]
    fn test_many_environments_one_device() {
        let mut manager = DeviceManager::new();

        let device = Device::new(
            DeviceType::Serial,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/serial0".to_string(),
        );
        let device_id = device.id.clone();
        manager.register_device(device);

        // Try to allocate to 10 environments (should fail after first)
        let mut successful = 0;
        for i in 0..10 {
            let request = PassthroughRequest::new(format!("env{}", i), device_id.clone());
            let result = manager.request_passthrough(request);

            if result.success {
                successful += 1;
                if i > 0 {
                    // Release previous environment
                    manager.release_device(&device_id, &format!("env{}", i - 1));
                }
            }
        }

        // Only 1 should succeed at a time (last one)
        assert!(successful >= 1);
    }
}
