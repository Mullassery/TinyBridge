/// OpenTelemetry Trace Provider Setup
/// Phase 4.0.2: OTel Integration
///
/// Initialize trace provider with Jaeger backend and Prometheus metrics
use std::time::Duration;

/// OTel provider configuration
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// Service name
    pub service_name: String,
    /// Jaeger endpoint (e.g., "http://localhost:6831")
    pub jaeger_endpoint: String,
    /// Enable Jaeger tracing
    pub enable_tracing: bool,
    /// Enable Prometheus metrics
    pub enable_metrics: bool,
    /// Trace sampling rate (0.0-1.0)
    pub sampling_rate: f64,
    /// Batch size for trace export
    pub batch_size: usize,
    /// Timeout for export operations
    pub export_timeout: Duration,
}

impl Default for OtelConfig {
    fn default() -> Self {
        OtelConfig {
            service_name: "tinybridge".to_string(),
            jaeger_endpoint: "http://localhost:6831".to_string(),
            enable_tracing: true,
            enable_metrics: true,
            sampling_rate: 1.0,
            batch_size: 512,
            export_timeout: Duration::from_secs(10),
        }
    }
}

/// OTel provider initialization result
pub struct OtelProvider {
    /// Service name
    pub service_name: String,
    /// Whether tracing is enabled
    pub tracing_enabled: bool,
    /// Whether metrics are enabled
    pub metrics_enabled: bool,
}

impl OtelProvider {
    /// Initialize OTel provider with configuration
    pub fn init(config: OtelConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Trace provider initialization: not implemented. The `otel` feature is
        // declared in Cargo.toml as a no-op on purpose (see that file's comment) --
        // this crate does not depend on `opentelemetry`/`opentelemetry_jaeger`, so
        // there is deliberately no `#[cfg(feature = "otel")]` block here that would
        // reference them. A real implementation needs those crates added as
        // dependencies first; until then, enabling `otel` must keep compiling and
        // keep doing nothing, matching `OtelProvider::has_tracing()`'s contract of
        // reflecting only what `config.enable_tracing` was set to, not real tracer
        // installation.
        let _ = config.enable_tracing;

        // Metrics initialization: not implemented, for the same reason (no real
        // `metrics`/`prometheus` dependency exists yet). See above.
        let _ = config.enable_metrics;

        Ok(OtelProvider {
            service_name: config.service_name,
            tracing_enabled: config.enable_tracing,
            metrics_enabled: config.enable_metrics,
        })
    }

    /// Check if tracing is available
    pub fn has_tracing(&self) -> bool {
        self.tracing_enabled
    }

    /// Check if metrics are available
    pub fn has_metrics(&self) -> bool {
        self.metrics_enabled
    }
}

/// Trace context for operations
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Parent span ID (if nested)
    pub parent_span_id: Option<String>,
    /// Operation name
    pub operation: String,
    /// Custom attributes
    pub attributes: std::collections::HashMap<String, String>,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new(operation: impl Into<String>) -> Self {
        use std::collections::HashMap;

        TraceContext {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_span_id: None,
            operation: operation.into(),
            attributes: HashMap::new(),
        }
    }

    /// Add a custom attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Set parent span
    pub fn with_parent(mut self, parent: String) -> Self {
        self.parent_span_id = Some(parent);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otel_config_default() {
        let config = OtelConfig::default();
        assert_eq!(config.service_name, "tinybridge");
        assert_eq!(config.sampling_rate, 1.0);
        assert!(config.enable_tracing);
        assert!(config.enable_metrics);
    }

    #[test]
    fn test_otel_provider_init() {
        let config = OtelConfig {
            enable_tracing: false,
            enable_metrics: false,
            ..Default::default()
        };

        let provider = OtelProvider::init(config).unwrap();
        assert_eq!(provider.service_name, "tinybridge");
        assert!(!provider.has_tracing());
        assert!(!provider.has_metrics());
    }

    /// Regression test: `OtelConfig::default()` sets `enable_tracing`/`enable_metrics`
    /// to `true`, and `init()` must still succeed without panicking or requiring any
    /// real `opentelemetry`/`opentelemetry_jaeger`/`prometheus` dependency -- those
    /// crates are deliberately not wired up yet (see `init()`'s doc comment and the
    /// `otel`/`metrics` feature comments in Cargo.toml). This is the exact case that
    /// previously only compiled with the `otel` feature *disabled*; building with
    /// `--features otel` (or `--all-features`) failed with unresolved imports because
    /// the feature-gated block referenced crates that were never added as
    /// dependencies, even though enabling `otel` is documented as a no-op.
    #[test]
    fn test_otel_provider_init_with_tracing_and_metrics_enabled_does_not_panic() {
        let config = OtelConfig::default();
        assert!(config.enable_tracing);
        assert!(config.enable_metrics);

        let provider = OtelProvider::init(config).unwrap();
        assert_eq!(provider.service_name, "tinybridge");
        // No real tracer/metrics backend exists yet, so these just reflect the
        // config flags that were passed in, not a real installed provider.
        assert!(provider.has_tracing());
        assert!(provider.has_metrics());
    }

    #[test]
    fn test_trace_context_creation() {
        let ctx = TraceContext::new("vm.boot");
        assert_eq!(ctx.operation, "vm.boot");
        assert!(!ctx.trace_id.is_empty());
        assert!(!ctx.span_id.is_empty());
        assert!(ctx.parent_span_id.is_none());
    }

    #[test]
    fn test_trace_context_attributes() {
        let ctx = TraceContext::new("cli.up")
            .with_attribute("env_name", "dev-1")
            .with_attribute("cpu", "4");

        assert_eq!(ctx.attributes.get("env_name"), Some(&"dev-1".to_string()));
        assert_eq!(ctx.attributes.get("cpu"), Some(&"4".to_string()));
    }

    #[test]
    fn test_trace_context_parent() {
        let parent_id = "parent-span-123";
        let ctx = TraceContext::new("vz.create_vm").with_parent(parent_id.to_string());

        assert_eq!(ctx.parent_span_id, Some(parent_id.to_string()));
    }
}
