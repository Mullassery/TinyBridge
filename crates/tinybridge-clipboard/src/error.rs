use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClipboardError {
    #[error("Failed to access macOS pasteboard: {0}")]
    PasteboardError(String),

    #[error("Failed to read clipboard content: {0}")]
    ReadError(String),

    #[error("Failed to write clipboard content: {0}")]
    WriteError(String),

    #[error("SSH connection error: {0}")]
    SshError(String),

    #[error("Clipboard not available")]
    NotAvailable,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, ClipboardError>;
