use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Type of device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    /// USB device (VID:PID)
    Usb,
    /// Serial port (/dev/ttyUSB*, /dev/ttyACM*)
    Serial,
    /// Camera device (/dev/video*)
    Camera,
    /// Audio device
    Audio,
    /// Generic character device
    CharDevice,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Usb => write!(f, "USB"),
            DeviceType::Serial => write!(f, "Serial"),
            DeviceType::Camera => write!(f, "Camera"),
            DeviceType::Audio => write!(f, "Audio"),
            DeviceType::CharDevice => write!(f, "CharDevice"),
        }
    }
}

/// Device attachment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    /// Device is available but not attached
    Available,
    /// Device is attached to an environment
    Attached,
    /// Device was detached
    Detached,
    /// Device encountered an error
    Error,
}

/// Detailed device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Unique device ID
    pub id: Uuid,

    /// Human-readable device name
    pub name: String,

    /// Device type
    pub device_type: DeviceType,

    /// Host-side device path (e.g., /dev/ttyUSB0, /dev/video0)
    pub host_path: PathBuf,

    /// VM-side device path (where it appears in VM)
    pub vm_path: PathBuf,

    /// Current status
    pub status: DeviceStatus,

    /// Environment ID this device is attached to (if any)
    pub attached_to_env: Option<Uuid>,

    /// Device vendor ID (for USB devices)
    pub vendor_id: Option<u16>,

    /// Device product ID (for USB devices)
    pub product_id: Option<u16>,

    /// Serial number (if available)
    pub serial_number: Option<String>,

    /// Baud rate (for serial devices)
    pub baud_rate: Option<u32>,

    /// Attachment timestamp
    pub attached_at: Option<DateTime<Utc>>,

    /// Last error message (if status == Error)
    pub error_message: Option<String>,
}

impl Device {
    /// Create a new device
    pub fn new(
        name: String,
        device_type: DeviceType,
        host_path: PathBuf,
    ) -> Self {
        // Auto-determine VM path based on device type and host path
        let vm_path = match device_type {
            DeviceType::Serial => {
                // /dev/ttyUSB0 → /dev/ttyUSB0 (same in VM)
                host_path.clone()
            }
            DeviceType::Camera => {
                // /dev/video0 → /dev/video0 (same in VM)
                host_path.clone()
            }
            _ => host_path.clone(),
        };

        Self {
            id: Uuid::new_v4(),
            name,
            device_type,
            host_path,
            vm_path,
            status: DeviceStatus::Available,
            attached_to_env: None,
            vendor_id: None,
            product_id: None,
            serial_number: None,
            baud_rate: None,
            attached_at: None,
            error_message: None,
        }
    }

    /// Create a USB device with VID/PID
    pub fn usb(
        name: String,
        host_path: PathBuf,
        vendor_id: u16,
        product_id: u16,
    ) -> Self {
        let mut device = Self::new(name, DeviceType::Usb, host_path);
        device.vendor_id = Some(vendor_id);
        device.product_id = Some(product_id);
        device
    }

    /// Create a serial device with baud rate
    pub fn serial(
        name: String,
        host_path: PathBuf,
        baud_rate: u32,
    ) -> Self {
        let mut device = Self::new(name, DeviceType::Serial, host_path);
        device.baud_rate = Some(baud_rate);
        device
    }

    /// Attach device to an environment
    pub fn attach(&mut self, env_id: Uuid) -> Result<(), String> {
        if self.status == DeviceStatus::Attached {
            return Err(format!("Device already attached to {:?}", self.attached_to_env));
        }

        self.status = DeviceStatus::Attached;
        self.attached_to_env = Some(env_id);
        self.attached_at = Some(Utc::now());
        self.error_message = None;
        Ok(())
    }

    /// Detach device from environment
    pub fn detach(&mut self) {
        self.status = DeviceStatus::Detached;
        self.attached_to_env = None;
        self.attached_at = None;
    }

    /// Mark device as errored
    pub fn set_error(&mut self, message: String) {
        self.status = DeviceStatus::Error;
        self.error_message = Some(message);
    }

    /// Clear error state
    pub fn clear_error(&mut self) {
        self.status = DeviceStatus::Available;
        self.error_message = None;
    }

    /// Check if device is currently available
    pub fn is_available(&self) -> bool {
        self.status == DeviceStatus::Available
    }

    /// Check if device is attached
    pub fn is_attached(&self) -> bool {
        self.status == DeviceStatus::Attached
    }

    /// Get human-readable status string
    pub fn status_string(&self) -> String {
        match self.status {
            DeviceStatus::Available => "Available".to_string(),
            DeviceStatus::Attached => format!("Attached to env {:?}", self.attached_to_env),
            DeviceStatus::Detached => "Detached".to_string(),
            DeviceStatus::Error => format!("Error: {}", self.error_message.as_ref().unwrap_or(&"Unknown".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_device() {
        let device = Device::new(
            "Arduino".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );

        assert_eq!(device.name, "Arduino");
        assert_eq!(device.device_type, DeviceType::Serial);
        assert_eq!(device.status, DeviceStatus::Available);
        assert!(device.is_available());
    }

    #[test]
    fn test_usb_device() {
        let device = Device::usb(
            "Camera".to_string(),
            PathBuf::from("/dev/bus/usb/001/010"),
            0x1234,
            0x5678,
        );

        assert_eq!(device.vendor_id, Some(0x1234));
        assert_eq!(device.product_id, Some(0x5678));
    }

    #[test]
    fn test_serial_device() {
        let device = Device::serial(
            "FTDI".to_string(),
            PathBuf::from("/dev/ttyUSB0"),
            9600,
        );

        assert_eq!(device.baud_rate, Some(9600));
    }

    #[test]
    fn test_attach_device() {
        let mut device = Device::new(
            "Test".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );

        let env_id = Uuid::new_v4();
        device.attach(env_id).unwrap();

        assert_eq!(device.status, DeviceStatus::Attached);
        assert_eq!(device.attached_to_env, Some(env_id));
        assert!(device.is_attached());
    }

    #[test]
    fn test_detach_device() {
        let mut device = Device::new(
            "Test".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );

        let env_id = Uuid::new_v4();
        device.attach(env_id).unwrap();
        device.detach();

        assert_eq!(device.status, DeviceStatus::Detached);
        assert_eq!(device.attached_to_env, None);
    }

    #[test]
    fn test_device_error() {
        let mut device = Device::new(
            "Test".to_string(),
            DeviceType::Serial,
            PathBuf::from("/dev/ttyUSB0"),
        );

        device.set_error("Connection refused".to_string());
        assert_eq!(device.status, DeviceStatus::Error);
        assert!(device.error_message.is_some());

        device.clear_error();
        assert_eq!(device.status, DeviceStatus::Available);
        assert!(device.error_message.is_none());
    }
}
