use thiserror::Error;

#[derive(Error, Debug)]
pub enum VzError {
    #[error("VM creation failed")]
    CreationFailed,

    #[error("VM start failed")]
    StartFailed,

    #[error("VM stop failed")]
    StopFailed,

    #[error("Invalid configuration")]
    InvalidConfig,

    #[error("VZ framework not available")]
    NotAvailable,

    #[error("VirtioFS mount failed")]
    VirtioFSMountFailed,

    #[error("Status query failed")]
    StatusQueryFailed,
}

pub type Result<T> = std::result::Result<T, VzError>;
