use crate::error_propagation::ErrorPropagator;
use crate::health::{HealthChecker, HealthReport};
use serde_json::{json, Value};
use tinybridge_core::JsonRpcResponse;
use tinybridge_error::BridgeError;

/// Handle RPC method calls with error propagation
pub fn handle_rpc_method(
    method: &str,
    params: &Value,
    request_id: i64,
    health_checker: &HealthChecker,
) -> Option<JsonRpcResponse> {
    match method {
        "health" => handle_health_check(request_id, health_checker),
        "health.full" => handle_full_health_check(request_id, health_checker),
        _ => None, // Let other handlers process unknown methods
    }
}

/// Handle /health endpoint (lightweight)
fn handle_health_check(request_id: i64, health_checker: &HealthChecker) -> Option<JsonRpcResponse> {
    let report = health_checker.check_all();

    Some(JsonRpcResponse::success(
        request_id,
        json!({
            "status": report.status,
            "uptime_seconds": report.uptime_seconds,
            "timestamp": report.timestamp,
            "resource_count": report.resources.len(),
        }),
    ))
}

/// Handle /health/full endpoint (detailed)
fn handle_full_health_check(
    request_id: i64,
    health_checker: &HealthChecker,
) -> Option<JsonRpcResponse> {
    let report = health_checker.check_all();
    Some(JsonRpcResponse::success(request_id, report))
}

/// Convert a BridgeError to JSON-RPC error response
pub fn error_to_response(error: BridgeError, request_id: i64) -> JsonRpcResponse {
    let (code, message, data) = ErrorPropagator::to_json_rpc_error(&error);

    if let Some(error_data) = data {
        JsonRpcResponse::error_with_data(request_id, code, &message, error_data)
    } else {
        JsonRpcResponse::error(request_id, code, &message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_endpoint() {
        let health_checker = HealthChecker::new();
        let response = handle_health_check(1, &health_checker);

        assert!(response.is_some());
        if let Some(resp) = response {
            // Verify response structure
            assert_eq!(resp.id, 1);
        }
    }

    #[test]
    fn test_full_health_endpoint() {
        let health_checker = HealthChecker::new();
        let response = handle_full_health_check(1, &health_checker);

        assert!(response.is_some());
    }

    #[test]
    fn test_error_to_response() {
        let error = BridgeError::vm("Test error".to_string());
        let response = error_to_response(error, 1);

        assert_eq!(response.id, 1);
        // Response should have error field
    }

    #[test]
    fn test_unknown_method() {
        let health_checker = HealthChecker::new();
        let response = handle_rpc_method("unknown_method", &json!({}), 1, &health_checker);

        assert!(response.is_none()); // Unknown methods return None for other handlers
    }
}
