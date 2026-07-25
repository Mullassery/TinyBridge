use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Structured logging context for daemon operations
#[derive(Debug, Clone)]
pub struct LogContext {
    pub operation: String,
    pub environment: Option<String>,
    pub correlation_id: String,
    pub user_id: Option<String>,
    pub extra_fields: HashMap<String, serde_json::Value>,
}

impl LogContext {
    pub fn new(operation: &str, correlation_id: &str) -> Self {
        LogContext {
            operation: operation.to_string(),
            environment: None,
            correlation_id: correlation_id.to_string(),
            user_id: None,
            extra_fields: HashMap::new(),
        }
    }

    pub fn with_environment(mut self, env: &str) -> Self {
        self.environment = Some(env.to_string());
        self
    }

    pub fn with_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn with_field(mut self, key: &str, value: serde_json::Value) -> Self {
        self.extra_fields.insert(key.to_string(), value);
        self
    }

    /// Log as JSON for structured analysis
    pub fn to_json(&self) -> serde_json::Value {
        let mut obj = json!({
            "operation": self.operation,
            "correlation_id": self.correlation_id,
        });

        if let Some(env) = &self.environment {
            obj["environment"] = json!(env);
        }

        if let Some(user_id) = &self.user_id {
            obj["user_id"] = json!(user_id);
        }

        for (k, v) in &self.extra_fields {
            obj[k] = v.clone();
        }

        obj
    }
}

/// Log an operation start
pub fn log_operation_start(ctx: &LogContext) {
    info!(
        "Operation started: {}",
        ctx.to_json(),
        // In production, would use OTel spans here
    );
}

/// Log an operation success
pub fn log_operation_success(ctx: &LogContext, duration_ms: u64) {
    info!("Operation succeeded: {} ({}ms)", ctx.to_json(), duration_ms,);
}

/// Log an operation error with context
pub fn log_operation_error(ctx: &LogContext, error_code: i32, error_msg: &str, duration_ms: u64) {
    error!(
        "Operation failed: {} error_code={} error={} ({}ms)",
        ctx.to_json(),
        error_code,
        error_msg,
        duration_ms,
    );
}

/// Log a warning with context
pub fn log_warning(ctx: &LogContext, message: &str) {
    warn!("Warning: {} message={}", ctx.to_json(), message);
}

/// Log resource usage
pub fn log_resource_usage(operation: &str, memory_mb: u64, cpu_pct: f64) {
    info!(
        "Resource usage: operation={} memory_mb={} cpu_pct={}",
        operation, memory_mb, cpu_pct
    );
}

/// Log error with recovery suggestions
pub fn log_error_with_suggestion(
    ctx: &LogContext,
    error_code: i32,
    error_msg: &str,
    suggestion: &str,
) {
    error!(
        "Error with suggestion: {} error_code={} error={} suggestion={}",
        ctx.to_json(),
        error_code,
        error_msg,
        suggestion
    );
}

/// Initialize structured logging (would integrate OTel in production)
pub fn init_structured_logging() {
    // This will be enhanced in Phase 3.0.3 with full OTel integration
    info!("Structured logging initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_context_creation() {
        let ctx = LogContext::new("test_op", "corr-123");
        assert_eq!(ctx.operation, "test_op");
        assert_eq!(ctx.correlation_id, "corr-123");
    }

    #[test]
    fn test_log_context_with_environment() {
        let ctx = LogContext::new("test", "id")
            .with_environment("production")
            .with_user("user-42");

        assert_eq!(ctx.environment, Some("production".to_string()));
        assert_eq!(ctx.user_id, Some("user-42".to_string()));
    }

    #[test]
    fn test_log_context_to_json() {
        let ctx = LogContext::new("op", "id")
            .with_environment("staging")
            .with_field("vm_id", json!("vm-123"));

        let json = ctx.to_json();
        assert_eq!(json["operation"], "op");
        assert_eq!(json["correlation_id"], "id");
        assert_eq!(json["environment"], "staging");
        assert_eq!(json["vm_id"], "vm-123");
    }

    #[test]
    fn test_log_context_extra_fields() {
        let ctx = LogContext::new("op", "id")
            .with_field("attempt", json!(2))
            .with_field("timeout_ms", json!(5000));

        let json = ctx.to_json();
        assert_eq!(json["attempt"], 2);
        assert_eq!(json["timeout_ms"], 5000);
    }

    #[test]
    fn test_log_operations() {
        let ctx = LogContext::new("start_vm", "trace-123");

        // These won't fail, just verify they don't panic
        log_operation_start(&ctx);
        log_operation_success(&ctx, 1500);
        log_operation_error(&ctx, -32000, "Memory error", 1500);
        log_warning(&ctx, "Low disk space");
        log_resource_usage("boot", 2048, 45.5);
        log_error_with_suggestion(&ctx, -32002, "Disk full", "Free up space");
    }
}
