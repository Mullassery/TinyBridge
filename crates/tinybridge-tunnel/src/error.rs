use thiserror::Error;

#[derive(Error, Debug)]
pub enum TunnelError {
    #[error("Tunnel not found: {0}")]
    TunnelNotFound(String),

    #[error("SSH connection failed: {0}")]
    SshConnectionError(String),

    #[error("Port binding failed: {0}")]
    PortBindError(String),

    #[error("Tunnel creation failed: {0}")]
    CreationError(String),

    #[error("Tunnel already exists: {0}")]
    TunnelExists(String),

    #[error("Invalid tunnel configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TunnelError>;
