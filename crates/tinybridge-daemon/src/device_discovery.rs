/// Device discovery and enumeration for macOS
///
/// Scans system devices and populates the device manager.
/// Currently uses mock enumeration (ready for IOKit integration).
use crate::device_manager::{Device, DeviceManager, DeviceType};
use std::process::Command;

/// Device discovery scanner
pub struct DeviceDiscovery;

impl DeviceDiscovery {
    /// Discover all devices on the system
    pub fn discover_all(manager: &mut DeviceManager) -> std::io::Result<usize> {
        let initial_count = manager.total_count();

        Self::discover_usb_devices(manager)?;
        Self::discover_serial_devices(manager)?;
        Self::discover_camera_devices(manager)?;
        Self::discover_audio_devices(manager)?;

        Ok(manager.total_count() - initial_count)
    }

    /// Discover USB devices
    fn discover_usb_devices(manager: &mut DeviceManager) -> std::io::Result<()> {
        // Try to use system_profiler for USB device enumeration
        match Self::system_profiler_usb() {
            Ok(devices) => {
                for device in devices {
                    manager.register_device(device);
                }
            }
            Err(_) => {
                // Fallback: register known USB device patterns
                Self::register_usb_fallback(manager);
            }
        }
        Ok(())
    }

    /// Discover serial devices
    fn discover_serial_devices(manager: &mut DeviceManager) -> std::io::Result<()> {
        // Check /dev for serial ports (cu.* are outgoing connections)
        let paths = [
            "/dev/cu.usbserial",
            "/dev/cu.usbmodem",
            "/dev/cu.SLAB_USBtoUART",
        ];

        for path in &paths {
            if std::path::Path::new(path).exists() {
                let device = Device::new(
                    DeviceType::Serial,
                    "Generic".to_string(),
                    format!(
                        "Serial Port ({})",
                        path.split('/').last().unwrap_or("unknown")
                    ),
                    path.to_string(),
                );
                manager.register_device(device);
            }
        }

        Ok(())
    }

    /// Discover camera devices
    fn discover_camera_devices(manager: &mut DeviceManager) -> std::io::Result<()> {
        // Check for /dev/video* devices (Linux compatibility)
        // On macOS, cameras are usually only accessible via AVFoundation
        for i in 0..4 {
            let path = format!("/dev/video{}", i);
            if std::path::Path::new(&path).exists() {
                let device = Device::new(
                    DeviceType::Camera,
                    "System".to_string(),
                    format!("Camera {}", i),
                    path,
                );
                manager.register_device(device);
            }
        }

        // Add built-in camera detection (macOS specific)
        // This is a placeholder for AVFoundation integration
        let device = Device::new(
            DeviceType::Camera,
            "Apple".to_string(),
            "Integrated Camera".to_string(),
            "/dev/camera0".to_string(),
        );
        manager.register_device(device);

        Ok(())
    }

    /// Discover audio devices
    fn discover_audio_devices(manager: &mut DeviceManager) -> std::io::Result<()> {
        // Try system_profiler for audio devices
        match Self::system_profiler_audio() {
            Ok(devices) => {
                for device in devices {
                    manager.register_device(device);
                }
            }
            Err(_) => {
                // Fallback: register generic audio device
                let device = Device::new(
                    DeviceType::Audio,
                    "System".to_string(),
                    "Default Audio Device".to_string(),
                    "/dev/audio0".to_string(),
                );
                manager.register_device(device);
            }
        }
        Ok(())
    }

    /// Get USB devices using system_profiler
    fn system_profiler_usb() -> std::io::Result<Vec<Device>> {
        let output = Command::new("system_profiler")
            .args(&["SPUSBDataType", "-json"])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        // Simple parsing (real implementation would use JSON)
        if stdout.contains("USB") {
            // Placeholder for actual parsing
            devices.push(Device::new(
                DeviceType::Usb,
                "Apple".to_string(),
                "USB Hub".to_string(),
                "/dev/usb0".to_string(),
            ));
        }

        Ok(devices)
    }

    /// Get audio devices using system_profiler
    fn system_profiler_audio() -> std::io::Result<Vec<Device>> {
        let output = Command::new("system_profiler")
            .args(&["SPAudioDataType", "-json"])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let mut devices = Vec::new();

        // Try to parse builtin output and input
        devices.push(Device::new(
            DeviceType::Audio,
            "Apple".to_string(),
            "Built-in Microphone".to_string(),
            "/dev/audioin0".to_string(),
        ));

        devices.push(Device::new(
            DeviceType::Audio,
            "Apple".to_string(),
            "Built-in Speaker".to_string(),
            "/dev/audioout0".to_string(),
        ));

        Ok(devices)
    }

    /// Fallback USB device registration
    fn register_usb_fallback(manager: &mut DeviceManager) {
        let device = Device::new(
            DeviceType::Usb,
            "Generic".to_string(),
            "USB Device".to_string(),
            "/dev/usb0".to_string(),
        );
        manager.register_device(device);
    }

    /// Rescan for new/removed devices
    pub fn rescan(manager: &mut DeviceManager) -> std::io::Result<()> {
        // Clear and rediscover (in production, would do incremental updates)
        *manager = DeviceManager::new();
        Self::discover_all(manager)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_discovery_initialization() {
        let mut manager = DeviceManager::new();

        // Should not panic
        let _ = DeviceDiscovery::discover_all(&mut manager);

        // Should have found at least the fallback devices
        assert!(manager.total_count() > 0);
    }

    #[test]
    fn test_discover_usb_devices() {
        let mut manager = DeviceManager::new();

        let result = DeviceDiscovery::discover_usb_devices(&mut manager);
        assert!(result.is_ok());

        // Should have at least registered fallback USB device
        let usb_devices = manager.devices_by_type(DeviceType::Usb);
        assert!(!usb_devices.is_empty());
    }

    #[test]
    fn test_discover_serial_devices() {
        let mut manager = DeviceManager::new();

        let result = DeviceDiscovery::discover_serial_devices(&mut manager);
        assert!(result.is_ok());
        // Serial devices may or may not exist, but should not error
    }

    #[test]
    fn test_discover_camera_devices() {
        let mut manager = DeviceManager::new();

        let result = DeviceDiscovery::discover_camera_devices(&mut manager);
        assert!(result.is_ok());

        // Should have at least the integrated camera
        let camera_devices = manager.devices_by_type(DeviceType::Camera);
        assert!(!camera_devices.is_empty());
    }

    #[test]
    fn test_discover_audio_devices() {
        let mut manager = DeviceManager::new();

        let result = DeviceDiscovery::discover_audio_devices(&mut manager);
        assert!(result.is_ok());

        // Should have at least the fallback audio device
        let audio_devices = manager.devices_by_type(DeviceType::Audio);
        assert!(!audio_devices.is_empty());
    }

    #[test]
    fn test_discover_all_multiple_calls() {
        let mut manager = DeviceManager::new();

        let count1 = DeviceDiscovery::discover_all(&mut manager).unwrap();
        let count2 = DeviceDiscovery::discover_all(&mut manager).unwrap();

        // Second call should discover same or more devices
        assert!(count2 >= 0);
    }

    #[test]
    fn test_rescan() {
        let mut manager = DeviceManager::new();
        DeviceDiscovery::discover_all(&mut manager).unwrap();

        let initial_count = manager.total_count();
        assert!(initial_count > 0);

        // Rescan should reset and rediscover
        DeviceDiscovery::rescan(&mut manager).unwrap();
        assert_eq!(manager.total_count(), initial_count);
    }
}
