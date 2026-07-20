use thiserror::Error;

#[derive(Error, Debug)]
pub enum SnapshotError {
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("Failed to create snapshot: {0}")]
    CreationError(String),

    #[error("Failed to restore snapshot: {0}")]
    RestoreError(String),

    #[error("Failed to delete snapshot: {0}")]
    DeletionError(String),

    #[error("Clone operation failed: {0}")]
    CloneError(String),

    #[error("Invalid snapshot: {0}")]
    InvalidSnapshot(String),

    #[error("Snapshot already exists: {0}")]
    SnapshotExists(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SnapshotError>;
