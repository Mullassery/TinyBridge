use thiserror::Error;

#[derive(Error, Debug)]
pub enum DnsError {
    #[error("DNS service error: {0}")]
    ServiceError(String),

    #[error("Failed to register DNS entry: {0}")]
    RegistrationError(String),

    #[error("Failed to resolve DNS: {0}")]
    ResolutionError(String),

    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),

    #[error("mDNS responder failed: {0}")]
    ResponderError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, DnsError>;
