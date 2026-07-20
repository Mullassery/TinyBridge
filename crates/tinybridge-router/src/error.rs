use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Failed to read binary: {0}")]
    BinaryReadError(String),

    #[error("Unsupported binary format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid routing rule: {0}")]
    InvalidRule(String),

    #[error("No routing rules matched for: {0}")]
    NoRuleMatch(String),

    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),

    #[error("Missing capability: {0}")]
    MissingCapability(String),

    #[error("Routing cache error: {0}")]
    CacheError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Anyhow error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, RouterError>;
