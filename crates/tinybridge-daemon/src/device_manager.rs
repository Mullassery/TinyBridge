/// Device passthrough management for TinyBridge
///
/// Handles USB, serial, camera, and audio device passthrough to VMs.
/// Provides device discovery, enumeration, and request routing.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Device type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    /// USB devices (keyboards, mice, storage, etc.)
    Usb,
    /// Serial ports (robotics, microcontrollers)
    Serial,
    /// Camera devices (inference, streaming)
    Camera,
    /// Audio devices (input/output)
    Audio,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Usb => write!(f, "usb"),
            Self::Serial => write!(f, "serial"),
            Self::Camera => write!(f, "camera"),
            Self::Audio => write!(f, "audio"),
        }
    }
}

/// Device status on the host
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    /// Device is available and ready
    Available,
    /// Device is in use by another VM
    InUse,
    /// Device is unavailable (unplugged, etc)
    Unavailable,
    /// Device passthrough is blocked by policy
    Blocked,
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Available => write!(f, "available"),
            Self::InUse => write!(f, "in_use"),
            Self::Unavailable => write!(f, "unavailable"),
            Self::Blocked => write!(f, "blocked"),
        }
    }
}

/// Device descriptor on the host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Unique device ID (UUID)
    pub id: String,
    /// Device type
    pub device_type: DeviceType,
    /// Vendor name (e.g., "Apple", "Arduino")
    pub vendor: String,
    /// Product name (e.g., "MacBook Camera", "USB Serial Adapter")
    pub product: String,
    /// Device path on macOS (e.g., "/dev/cu.usbserial-12345")
    pub path: String,
    /// Current status
    pub status: DeviceStatus,
    /// Environment currently using this device (if in use)
    pub in_use_by: Option<String>,
}

impl Device {
    /// Create new device descriptor
    pub fn new(device_type: DeviceType, vendor: String, product: String, path: String) -> Self {
        Device {
            id: Uuid::new_v4().to_string(),
            device_type,
            vendor,
            product,
            path,
            status: DeviceStatus::Available,
            in_use_by: None,
        }
    }

    /// Get human-readable device description
    pub fn description(&self) -> String {
        format!("{} - {} {}", self.device_type, self.vendor, self.product)
    }

    /// Check if device is available for passthrough
    pub fn is_available(&self) -> bool {
        matches!(self.status, DeviceStatus::Available)
    }
}

/// Passthrough request from a VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassthroughRequest {
    /// Request ID for tracking
    pub request_id: String,
    /// Environment name requesting the device
    pub environment: String,
    /// Device ID being requested
    pub device_id: String,
    /// Optional device filter (e.g., vendor ID for USB)
    pub filter: Option<String>,
}

impl PassthroughRequest {
    pub fn new(environment: String, device_id: String) -> Self {
        PassthroughRequest {
            request_id: Uuid::new_v4().to_string(),
            environment,
            device_id,
            filter: None,
        }
    }

    pub fn with_filter(mut self, filter: String) -> Self {
        self.filter = Some(filter);
        self
    }
}

/// Result of a passthrough request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassthroughResult {
    /// Request ID
    pub request_id: String,
    /// Success or error
    pub success: bool,
    /// Detailed message
    pub message: String,
    /// Device information if successful
    pub device: Option<Device>,
}

/// Device manager for coordinating passthrough
pub struct DeviceManager {
    /// All detected devices
    devices: HashMap<String, Device>,
    /// Passthrough history
    history: Vec<PassthroughRequest>,
}

impl DeviceManager {
    /// Create new device manager
    pub fn new() -> Self {
        DeviceManager {
            devices: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Register a device
    pub fn register_device(&mut self, device: Device) -> String {
        let id = device.id.clone();
        self.devices.insert(id.clone(), device);
        id
    }

    /// Get device by ID
    pub fn get_device(&self, device_id: &str) -> Option<&Device> {
        self.devices.get(device_id)
    }

    /// Get all devices of a specific type
    pub fn devices_by_type(&self, device_type: DeviceType) -> Vec<&Device> {
        self.devices
            .values()
            .filter(|d| d.device_type == device_type)
            .collect()
    }

    /// Get all available devices
    pub fn available_devices(&self) -> Vec<&Device> {
        self.devices.values().filter(|d| d.is_available()).collect()
    }

    /// Get devices in use by an environment
    pub fn devices_for_environment(&self, env_name: &str) -> Vec<&Device> {
        self.devices
            .values()
            .filter(|d| d.in_use_by.as_deref() == Some(env_name))
            .collect()
    }

    /// Request passthrough for a device
    pub fn request_passthrough(&mut self, request: PassthroughRequest) -> PassthroughResult {
        self.history.push(request.clone());

        let device = match self.devices.get_mut(&request.device_id) {
            Some(d) => d,
            None => {
                return PassthroughResult {
                    request_id: request.request_id,
                    success: false,
                    message: "Device not found".to_string(),
                    device: None,
                }
            }
        };

        // Check availability
        if !device.is_available() {
            return PassthroughResult {
                request_id: request.request_id,
                success: false,
                message: format!("Device is {}", device.status),
                device: None,
            };
        }

        // Mark as in use
        device.in_use_by = Some(request.environment.clone());

        PassthroughResult {
            request_id: request.request_id,
            success: true,
            message: "Passthrough granted".to_string(),
            device: Some(device.clone()),
        }
    }

    /// Release passthrough for a device
    pub fn release_device(&mut self, device_id: &str, environment: &str) -> bool {
        if let Some(device) = self.devices.get_mut(device_id) {
            if device.in_use_by.as_deref() == Some(environment) {
                device.in_use_by = None;
                device.status = DeviceStatus::Available;
                return true;
            }
        }
        false
    }

    /// Get passthrough history
    pub fn history(&self) -> &[PassthroughRequest] {
        &self.history
    }

    /// Clear history (for testing)
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Get device count by type
    pub fn count_by_type(&self, device_type: DeviceType) -> usize {
        self.devices
            .values()
            .filter(|d| d.device_type == device_type)
            .count()
    }

    /// Get total device count
    pub fn total_count(&self) -> usize {
        self.devices.len()
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

    #[test]
    fn test_device_creation() {
        let device = Device::new(
            DeviceType::Usb,
            "Apple".to_string(),
            "MacBook Camera".to_string(),
            "/dev/camera0".to_string(),
        );

        assert_eq!(device.device_type, DeviceType::Usb);
        assert_eq!(device.vendor, "Apple");
        assert!(device.is_available());
        assert_eq!(device.status, DeviceStatus::Available);
    }

    #[test]
    fn test_device_manager_registration() {
        let mut manager = DeviceManager::new();
        let device = Device::new(
            DeviceType::Serial,
            "Arduino".to_string(),
            "USB Serial".to_string(),
            "/dev/cu.usbserial-12345".to_string(),
        );

        let id = manager.register_device(device.clone());
        assert_eq!(manager.total_count(), 1);
        assert!(manager.get_device(&id).is_some());
    }

    #[test]
    fn test_devices_by_type() {
        let mut manager = DeviceManager::new();

        manager.register_device(Device::new(
            DeviceType::Usb,
            "Vendor1".to_string(),
            "Device1".to_string(),
            "/dev/usb1".to_string(),
        ));

        manager.register_device(Device::new(
            DeviceType::Serial,
            "Vendor2".to_string(),
            "Device2".to_string(),
            "/dev/serial1".to_string(),
        ));

        assert_eq!(manager.devices_by_type(DeviceType::Usb).len(), 1);
        assert_eq!(manager.devices_by_type(DeviceType::Serial).len(), 1);
        assert_eq!(manager.devices_by_type(DeviceType::Camera).len(), 0);
    }

    #[test]
    fn test_available_devices() {
        let mut manager = DeviceManager::new();
        let device = Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/usb1".to_string(),
        );

        manager.register_device(device);
        assert_eq!(manager.available_devices().len(), 1);
    }

    #[test]
    fn test_passthrough_request() {
        let mut manager = DeviceManager::new();
        let device = Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/usb1".to_string(),
        );
        let device_id = device.id.clone();

        manager.register_device(device);

        let request = PassthroughRequest::new("test-env".to_string(), device_id.clone());
        let result = manager.request_passthrough(request);

        assert!(result.success);
        assert_eq!(result.message, "Passthrough granted");

        let device = manager.get_device(&device_id).unwrap();
        assert_eq!(device.in_use_by, Some("test-env".to_string()));
    }

    #[test]
    fn test_passthrough_unavailable_device() {
        let mut manager = DeviceManager::new();
        let mut device = Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/usb1".to_string(),
        );
        device.status = DeviceStatus::Unavailable;
        let device_id = device.id.clone();

        manager.register_device(device);

        let request = PassthroughRequest::new("test-env".to_string(), device_id);
        let result = manager.request_passthrough(request);

        assert!(!result.success);
        assert!(result.message.contains("unavailable"));
    }

    #[test]
    fn test_release_device() {
        let mut manager = DeviceManager::new();
        let device = Device::new(
            DeviceType::Usb,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/usb1".to_string(),
        );
        let device_id = device.id.clone();

        manager.register_device(device);

        let request = PassthroughRequest::new("test-env".to_string(), device_id.clone());
        manager.request_passthrough(request);

        assert!(manager.release_device(&device_id, "test-env"));
        assert!(manager.get_device(&device_id).unwrap().is_available());
    }

    #[test]
    fn test_devices_for_environment() {
        let mut manager = DeviceManager::new();

        for i in 0..3 {
            let device = Device::new(
                DeviceType::Usb,
                "Vendor".to_string(),
                format!("Device{}", i),
                format!("/dev/usb{}", i),
            );
            let device_id = device.id.clone();
            manager.register_device(device);

            if i < 2 {
                let request = PassthroughRequest::new("test-env".to_string(), device_id);
                manager.request_passthrough(request);
            }
        }

        assert_eq!(manager.devices_for_environment("test-env").len(), 2);
    }

    #[test]
    fn test_passthrough_history() {
        let mut manager = DeviceManager::new();
        let device = Device::new(
            DeviceType::Serial,
            "Vendor".to_string(),
            "Device".to_string(),
            "/dev/serial1".to_string(),
        );
        let device_id = device.id.clone();

        manager.register_device(device);

        for i in 0..3 {
            let request = PassthroughRequest::new(format!("env{}", i), device_id.clone());
            manager.request_passthrough(request);
        }

        assert_eq!(manager.history().len(), 3);
    }

    #[test]
    fn test_device_description() {
        let device = Device::new(
            DeviceType::Camera,
            "Apple".to_string(),
            "FaceTime Camera".to_string(),
            "/dev/video0".to_string(),
        );

        let desc = device.description();
        assert!(desc.contains("camera"));
        assert!(desc.contains("Apple"));
        assert!(desc.contains("FaceTime"));
    }

    #[test]
    fn test_count_by_type() {
        let mut manager = DeviceManager::new();

        for i in 0..3 {
            manager.register_device(Device::new(
                DeviceType::Usb,
                "Vendor".to_string(),
                format!("Device{}", i),
                format!("/dev/usb{}", i),
            ));
        }

        for i in 0..2 {
            manager.register_device(Device::new(
                DeviceType::Serial,
                "Vendor".to_string(),
                format!("Serial{}", i),
                format!("/dev/serial{}", i),
            ));
        }

        assert_eq!(manager.count_by_type(DeviceType::Usb), 3);
        assert_eq!(manager.count_by_type(DeviceType::Serial), 2);
        assert_eq!(manager.count_by_type(DeviceType::Camera), 0);
    }

    #[test]
    fn test_request_nonexistent_device() {
        let mut manager = DeviceManager::new();
        let request = PassthroughRequest::new("test-env".to_string(), "nonexistent-id".to_string());
        let result = manager.request_passthrough(request);

        assert!(!result.success);
        assert!(result.message.contains("not found"));
    }
}
