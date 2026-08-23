use serde_json::{json, Value};
use tinybridge_error::BridgeError;

/// Error code constants for JSON-RPC error responses
pub mod error_codes {
    pub const VIRTUALIZATION_ERROR: i32 = -32000;
    pub const MEMORY_ERROR: i32 = -32001;
    pub const DISK_ERROR: i32 = -32002;
    pub const NETWORK_ERROR: i32 = -32003;
    pub const STORAGE_ERROR: i32 = -32004;
    pub const PERMISSION_ERROR: i32 = -32005;
    pub const CONFIGURATION_ERROR: i32 = -32006;
    pub const UNKNOWN_ERROR: i32 = -32099;
}

/// Converts a BridgeError to a JSON-RPC error response with structured context
pub struct ErrorPropagator;

impl ErrorPropagator {
    /// Convert BridgeError to JSON-RPC error value with full context
    pub fn to_json_rpc_error(error: &BridgeError) -> (i32, String, Option<Value>) {
        let error_code = match error.kind {
            tinybridge_error::ErrorKind::Bootstrap => error_codes::VIRTUALIZATION_ERROR,
            tinybridge_error::ErrorKind::Vm => error_codes::VIRTUALIZATION_ERROR,
            tinybridge_error::ErrorKind::Cli => error_codes::UNKNOWN_ERROR,
            tinybridge_error::ErrorKind::Network => error_codes::NETWORK_ERROR,
            tinybridge_error::ErrorKind::Storage => error_codes::STORAGE_ERROR,
            tinybridge_error::ErrorKind::Permission => error_codes::PERMISSION_ERROR,
            tinybridge_error::ErrorKind::Configuration => error_codes::CONFIGURATION_ERROR,
            tinybridge_error::ErrorKind::Unknown => error_codes::UNKNOWN_ERROR,
        };

        let message = error.message.clone();

        let data = if error.context.is_some() || error.suggestion.is_some() {
            Some(json!({
                "severity": serde_json::to_value(error.severity).unwrap_or_else(|_| Value::String("Error".to_string())),
                "kind": format!("{}", error.kind),
                "context": error.context.as_ref().map(|ctx| {
                    ctx.details.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()
                }),
                "suggestion": error.suggestion.as_ref().map(|sugg| {
                    json!({
                        "steps": &sugg.steps,
                        "docs_link": &sugg.docs_link
                    })
                }),
            }))
        } else {
            None
        };

        (error_code, message, data)
    }

    /// Convert Result to JSON-RPC error, preserving BridgeError structure
    pub fn from_result<T>(
        result: Result<T, Box<dyn std::error::Error>>,
    ) -> Result<T, (i32, String, Option<Value>)> {
        match result {
            Ok(value) => Ok(value),
            Err(e) => {
                // Try to downcast to BridgeError
                if let Some(bridge_err) = e.downcast_ref::<BridgeError>() {
                    let (code, msg, data) = Self::to_json_rpc_error(bridge_err);
                    Err((code, msg, data))
                } else {
                    // Generic error handling
                    Err((error_codes::UNKNOWN_ERROR, e.to_string(), None))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tinybridge_error::{BridgeError, ErrorContext, ErrorSeverity, RecoverySuggestion};

    #[test]
    fn test_memory_error_propagation() {
        let err = BridgeError::vm("Memory allocation failed".to_string())
            .with_severity(ErrorSeverity::Error)
            .with_context(ErrorContext::memory_allocation(4096))
            .with_suggestion(RecoverySuggestion::increase_memory(4096));

        let (code, msg, data) = ErrorPropagator::to_json_rpc_error(&err);

        assert_eq!(code, error_codes::VIRTUALIZATION_ERROR);
        assert!(msg.contains("Memory allocation failed"));
        assert!(data.is_some());

        if let Some(json_data) = data {
            assert!(json_data.get("context").is_some());
            assert!(json_data.get("suggestion").is_some());
            assert_eq!(json_data["severity"], "Error");
        }
    }

    #[test]
    fn test_network_error_propagation() {
        let err = BridgeError::network("DNS resolution failed".to_string())
            .with_severity(ErrorSeverity::Warning);

        let (code, msg, _) = ErrorPropagator::to_json_rpc_error(&err);

        assert_eq!(code, error_codes::NETWORK_ERROR);
        assert!(msg.contains("DNS resolution failed"));
    }

    #[test]
    fn test_error_code_mapping() {
        let bootstrap_err = BridgeError::bootstrap("Test".to_string());
        let (code, _, _) = ErrorPropagator::to_json_rpc_error(&bootstrap_err);
        assert_eq!(code, error_codes::VIRTUALIZATION_ERROR);

        let storage_err = BridgeError::storage("Test".to_string());
        let (code, _, _) = ErrorPropagator::to_json_rpc_error(&storage_err);
        assert_eq!(code, error_codes::STORAGE_ERROR);

        let permission_err = BridgeError::permission("Test".to_string());
        let (code, _, _) = ErrorPropagator::to_json_rpc_error(&permission_err);
        assert_eq!(code, error_codes::PERMISSION_ERROR);
    }

    #[test]
    fn test_error_without_context() {
        let err = BridgeError::cli("Simple error".to_string());
        let (_, msg, data) = ErrorPropagator::to_json_rpc_error(&err);

        assert!(msg.contains("Simple error"));
        assert!(data.is_none());
    }
}
