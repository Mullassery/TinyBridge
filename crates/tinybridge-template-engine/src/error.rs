use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    NotFound(String),

    #[error("Invalid template format: {0}")]
    InvalidFormat(String),

    #[error("Template parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Template validation failed: {0}")]
    ValidationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_error_not_found() {
        let err = TemplateError::NotFound("test".to_string());
        assert_eq!(err.to_string(), "Template not found: test");
    }
}
