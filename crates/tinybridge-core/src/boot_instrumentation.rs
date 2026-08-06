/// Boot Process Instrumentation
/// Phase 4.0.2: OTel Integration
///
/// Spans and events for daemon boot phases with config context

use crate::otel_provider::TraceContext;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Boot phase stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootPhase {
    /// Pre-flight checks (permissions, environment, config)
    PreFlight,
    /// Config loading and validation
    ConfigLoad,
    /// Profile selection
    ProfileSelect,
    /// Resource allocation
    ResourceAlloc,
    /// Network setup
    NetworkSetup,
    /// Daemon initialization
    DaemonInit,
    /// Health monitor startup
    HealthMonitor,
    /// API server startup
    ApiServer,
    /// Ready for operations
    Ready,
}

impl std::fmt::Display for BootPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootPhase::PreFlight => write!(f, "preflight"),
            BootPhase::ConfigLoad => write!(f, "config_load"),
            BootPhase::ProfileSelect => write!(f, "profile_select"),
            BootPhase::ResourceAlloc => write!(f, "resource_alloc"),
            BootPhase::NetworkSetup => write!(f, "network_setup"),
            BootPhase::DaemonInit => write!(f, "daemon_init"),
            BootPhase::HealthMonitor => write!(f, "health_monitor"),
            BootPhase::ApiServer => write!(f, "api_server"),
            BootPhase::Ready => write!(f, "ready"),
        }
    }
}

/// Instrumentation for a single boot phase
#[derive(Debug, Clone)]
pub struct BootSpan {
    /// Phase being instrumented
    pub phase: BootPhase,
    /// Trace context
    pub trace_context: TraceContext,
    /// Start timestamp
    pub start_time: Instant,
    /// Duration (set on completion)
    pub duration: Option<Duration>,
    /// Status: success, warning, error
    pub status: SpanStatus,
    /// Config context (CPU, memory, profile)
    pub config_context: ConfigContext,
}

/// Span execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    Running,
    Success,
    Warning,
    Error,
}

/// Configuration context for spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigContext {
    /// Profile being used
    pub profile: String,
    /// CPU allocation
    pub cpus: u32,
    /// Memory allocation
    pub memory: u32,
    /// Disk allocation
    pub disk: u32,
    /// GPU enabled
    pub gpu: bool,
}

impl Default for ConfigContext {
    fn default() -> Self {
        ConfigContext {
            profile: "development".to_string(),
            cpus: 2,
            memory: 4,
            disk: 20,
            gpu: false,
        }
    }
}

impl BootSpan {
    /// Create a new boot span for a phase
    pub fn new(phase: BootPhase) -> Self {
        BootSpan {
            phase,
            trace_context: TraceContext::new(format!("boot.{}", phase)),
            start_time: Instant::now(),
            duration: None,
            status: SpanStatus::Running,
            config_context: ConfigContext::default(),
        }
    }

    /// Set config context
    pub fn with_config(mut self, config: ConfigContext) -> Self {
        self.config_context = config;
        self
    }

    /// Mark span as successfully completed
    pub fn success(mut self) -> Self {
        self.duration = Some(self.start_time.elapsed());
        self.status = SpanStatus::Success;
        self
    }

    /// Mark span with warning
    pub fn warning(mut self) -> Self {
        self.duration = Some(self.start_time.elapsed());
        self.status = SpanStatus::Warning;
        self
    }

    /// Mark span as errored
    pub fn error(mut self) -> Self {
        self.duration = Some(self.start_time.elapsed());
        self.status = SpanStatus::Error;
        self
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> Option<u128> {
        self.duration.map(|d| d.as_millis())
    }

    /// Serialize to JSON for logging
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "phase": self.phase.to_string(),
            "trace_id": self.trace_context.trace_id,
            "span_id": self.trace_context.span_id,
            "status": format!("{:?}", self.status).to_lowercase(),
            "duration_ms": self.duration_ms(),
            "config": {
                "profile": self.config_context.profile,
                "cpus": self.config_context.cpus,
                "memory": self.config_context.memory,
                "disk": self.config_context.disk,
                "gpu": self.config_context.gpu,
            }
        })
    }
}

/// Boot instrumentation tracker
#[derive(Debug, Clone)]
pub struct BootInstrumentation {
    /// Main boot span
    pub main_trace: TraceContext,
    /// All phases executed
    pub phases: Vec<BootSpan>,
    /// Overall start time
    pub start_time: Instant,
}

impl BootInstrumentation {
    /// Create new boot instrumentation
    pub fn new() -> Self {
        BootInstrumentation {
            main_trace: TraceContext::new("daemon.boot"),
            phases: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Record a boot phase
    pub fn record_phase(&mut self, span: BootSpan) {
        self.phases.push(span);
    }

    /// Get total boot time in milliseconds
    pub fn total_time_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }

    /// Check if boot completed successfully
    pub fn is_successful(&self) -> bool {
        self.phases.iter().all(|p| p.status != SpanStatus::Error)
    }

    /// Get boot summary
    pub fn summary(&self) -> BootSummary {
        BootSummary {
            total_time_ms: self.total_time_ms(),
            phases_count: self.phases.len(),
            successful: self.is_successful(),
            last_phase: self.phases.last().map(|p| p.phase),
            trace_id: self.main_trace.trace_id.clone(),
        }
    }
}

impl Default for BootInstrumentation {
    fn default() -> Self {
        Self::new()
    }
}

/// Boot completion summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootSummary {
    /// Total time in milliseconds
    pub total_time_ms: u128,
    /// Number of phases executed
    pub phases_count: usize,
    /// Whether boot was successful
    pub successful: bool,
    /// Last phase executed
    pub last_phase: Option<BootPhase>,
    /// Trace ID for correlation
    pub trace_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_phase_display() {
        assert_eq!(BootPhase::PreFlight.to_string(), "preflight");
        assert_eq!(BootPhase::ConfigLoad.to_string(), "config_load");
        assert_eq!(BootPhase::Ready.to_string(), "ready");
    }

    #[test]
    fn test_boot_span_creation() {
        let span = BootSpan::new(BootPhase::PreFlight);
        assert_eq!(span.phase, BootPhase::PreFlight);
        assert_eq!(span.status, SpanStatus::Running);
        assert!(span.duration.is_none());
    }

    #[test]
    fn test_boot_span_success() {
        let span = BootSpan::new(BootPhase::ConfigLoad).success();
        assert_eq!(span.status, SpanStatus::Success);
        assert!(span.duration.is_some());
        assert!(span.duration_ms().is_some());
    }

    #[test]
    fn test_boot_span_with_config() {
        let config = ConfigContext {
            profile: "production".to_string(),
            cpus: 8,
            memory: 16,
            disk: 100,
            gpu: true,
        };

        let span = BootSpan::new(BootPhase::ResourceAlloc).with_config(config.clone());
        assert_eq!(span.config_context.profile, "production");
        assert_eq!(span.config_context.cpus, 8);
        assert_eq!(span.config_context.gpu, true);
    }

    #[test]
    fn test_boot_span_json() {
        let span = BootSpan::new(BootPhase::DaemonInit)
            .with_config(ConfigContext {
                profile: "testing".to_string(),
                cpus: 2,
                memory: 4,
                disk: 30,
                gpu: false,
            })
            .success();

        let json = span.to_json();
        assert_eq!(json["phase"], "daemon_init");
        assert_eq!(json["status"], "success");
        assert_eq!(json["config"]["profile"], "testing");
        assert!(json["duration_ms"].is_number());
    }

    #[test]
    fn test_boot_instrumentation_recording() {
        let mut instr = BootInstrumentation::new();
        instr.record_phase(BootSpan::new(BootPhase::PreFlight).success());
        instr.record_phase(BootSpan::new(BootPhase::ConfigLoad).success());

        assert_eq!(instr.phases.len(), 2);
        assert!(instr.is_successful());
    }

    #[test]
    fn test_boot_instrumentation_error_handling() {
        let mut instr = BootInstrumentation::new();
        instr.record_phase(BootSpan::new(BootPhase::PreFlight).success());
        instr.record_phase(BootSpan::new(BootPhase::ConfigLoad).error());

        assert_eq!(instr.phases.len(), 2);
        assert!(!instr.is_successful());
    }

    #[test]
    fn test_boot_summary() {
        let mut instr = BootInstrumentation::new();
        instr.record_phase(BootSpan::new(BootPhase::PreFlight).success());
        instr.record_phase(BootSpan::new(BootPhase::ConfigLoad).success());
        instr.record_phase(BootSpan::new(BootPhase::Ready).success());

        let summary = instr.summary();
        assert_eq!(summary.phases_count, 3);
        assert!(summary.successful);
        assert_eq!(summary.last_phase, Some(BootPhase::Ready));
        assert!(summary.total_time_ms >= 0);
    }
}
