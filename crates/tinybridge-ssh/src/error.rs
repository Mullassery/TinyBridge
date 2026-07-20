use thiserror::Error;

#[derive(Error, Debug)]
pub enum SshError {
    #[error("Failed to generate SSH key: {0}")]
    KeyGenerationError(String),

    #[error("Failed to read SSH key: {0}")]
    KeyReadError(String),

    #[error("Failed to write SSH key: {0}")]
    KeyWriteError(String),

    #[error("SSH key not found: {0}")]
    KeyNotFound(String),

    #[error("Failed to read SSH config: {0}")]
    ConfigReadError(String),

    #[error("Failed to write SSH config: {0}")]
    ConfigWriteError(String),

    #[error("Invalid SSH config: {0}")]
    InvalidConfig(String),

    #[error("SSH provisioning failed: {0}")]
    ProvisioningError(String),

    #[error("Cloud-init template error: {0}")]
    TemplateError(String),

    #[error("Audit log error: {0}")]
    AuditError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid environment: {0}")]
    InvalidEnvironment(String),

    #[error("SSH key operation failed: {0}")]
    KeyOperationError(String),
}

pub type Result<T> = std::result::Result<T, SshError>;
