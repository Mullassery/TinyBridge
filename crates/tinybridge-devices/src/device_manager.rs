use crate::device::Device;
use crate::device::DeviceType;
use crate::error::{DeviceError, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Device manager for lifecycle operations
pub struct DeviceManager {
    devices: HashMap<Uuid, Device>,
    host_path_to_id: HashMap<PathBuf, Uuid>, // Reverse lookup
}

impl DeviceManager {
    /// Create a new device manager
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            host_path_to_id: HashMap::new(),
        }
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

    /// Attach device to environment
    pub fn attach(&mut self, device_id: Uuid, env_id: Uuid) -> Result<()> {
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
        manager.attach(device_id, env_id).unwrap();

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
