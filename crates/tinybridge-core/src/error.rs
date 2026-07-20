use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid API version: {0}")]
    InvalidApiVersion(String),

    #[error("Invalid kind: {0}")]
    InvalidKind(String),

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Invalid resource: {0}")]
    InvalidResource(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
