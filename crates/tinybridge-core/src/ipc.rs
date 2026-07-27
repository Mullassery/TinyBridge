use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request (newline-delimited over Unix socket)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: impl Into<String>, params: Value) -> Self {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: u64, result: Value) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: u64, code: i32, message: impl Into<String>) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    pub fn error_with_data(id: u64, code: i32, message: impl Into<String>, data: Value) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: Some(data),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Error codes (JSON-RPC 2.0 + custom app codes)
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    pub const ENVIRONMENT_NOT_FOUND: i32 = -32001;
    pub const ALREADY_RUNNING: i32 = -32002;
    pub const NOT_RUNNING: i32 = -32003;
    pub const VM_BOOT_FAILED: i32 = -32004;
    pub const SHELL_ERROR: i32 = -32005;
}

/// Method names
pub mod methods {
    pub const ENVIRONMENT_UP: &str = "environment.up";
    pub const ENVIRONMENT_DOWN: &str = "environment.down";
    pub const ENVIRONMENT_STATUS: &str = "environment.status";
    pub const ENVIRONMENT_LIST: &str = "environment.list";
    pub const ENVIRONMENT_SHELL: &str = "environment.shell";
    pub const ENVIRONMENT_GUI_SHOW: &str = "environment.gui.show";
    pub const ENVIRONMENT_GUI_HIDE: &str = "environment.gui.hide";
    pub const ENVIRONMENT_DOCTOR: &str = "environment.doctor";
    pub const ENVIRONMENT_REPAIR: &str = "environment.repair";

    pub const DDS_STATUS: &str = "dds.status";
    pub const DDS_LIST: &str = "dds.list";
    pub const DDS_ENABLE: &str = "dds.enable";
    pub const DDS_DISABLE: &str = "dds.disable";
    pub const DDS_FEATURES_LIST: &str = "dds.features.list";
    pub const DDS_FEATURE_ENABLE: &str = "dds.feature.enable";
    pub const DDS_FEATURE_DISABLE: &str = "dds.feature.disable";
    pub const DDS_PROFILES_LIST: &str = "dds.profiles.list";
    pub const DDS_PROFILE_APPLY: &str = "dds.profile.apply";
    pub const DDS_SECURITY_ENABLE: &str = "dds.security.enable";
    pub const DDS_POLICIES_LIST: &str = "dds.policies.list";
    pub const DDS_POLICY_CREATE: &str = "dds.policy.create";
    pub const DDS_OVERRIDE_GRANT: &str = "dds.override.grant";
    pub const DDS_AUDIT_EXPORT: &str = "dds.audit.export";
    pub const DDS_COMPLIANCE_REPORT: &str = "dds.compliance.report";
}

/// Request parameter structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_yaml_path: Option<String>,
    #[serde(default)]
    pub wait: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListParams {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default = "default_cols")]
    pub cols: u16,
    #[serde(default = "default_rows")]
    pub rows: u16,
}

fn default_cols() -> u16 {
    80
}

fn default_rows() -> u16 {
    24
}

/// Response structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownResponse {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    pub ip_address: Option<String>,
    pub uptime_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub environments: Vec<EnvironmentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub environments: Vec<EnvironmentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellResponse {
    pub id: String,
    pub shell: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_request_serialize() {
        let req = JsonRpcRequest::new(1, "environment.list", json!({}));
        let json_str = serde_json::to_string(&req).unwrap();
        assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
        assert!(json_str.contains("\"method\":\"environment.list\""));
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let resp = JsonRpcResponse::success(1, json!({"name": "my-env"}));
        assert!(resp.error.is_none());
        assert!(resp.result.is_some());
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let resp = JsonRpcResponse::error(1, error_codes::ENVIRONMENT_NOT_FOUND, "Not found");
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32001);
    }

    #[test]
    fn test_up_params_deserialization() {
        let json = r#"{"name":"test","wait":true}"#;
        let params: UpParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.name, Some("test".to_string()));
        assert!(params.wait);
    }

    #[test]
    fn test_shell_params_defaults() {
        let json = r#"{"name":"test"}"#;
        let params: ShellParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.cols, 80);
        assert_eq!(params.rows, 24);
    }
}
