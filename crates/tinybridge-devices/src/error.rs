use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Failed to attach device: {0}")]
    AttachError(String),

    #[error("Failed to detach device: {0}")]
    DetachError(String),

    #[error("Permission denied for device: {0}")]
    PermissionDenied(String),

    #[error("Device already attached: {0}")]
    AlreadyAttached(String),

    #[error("Invalid device path: {0}")]
    InvalidPath(String),

    #[error("Unsupported device type: {0}")]
    UnsupportedDevice(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, DeviceError>;
