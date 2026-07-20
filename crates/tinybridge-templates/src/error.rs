use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Template instantiation failed: {0}")]
    InstantiationError(String),
}

pub type Result<T> = std::result::Result<T, TemplateError>;
