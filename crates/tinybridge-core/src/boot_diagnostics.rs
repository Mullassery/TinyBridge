/// Boot Diagnostics & Health Monitoring
/// Phase 5: Production Hardening
///
/// Health checks, diagnostics, and performance profiling
use crate::boot_stages::BootTier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Component health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component not yet initialized
    #[default]
    Uninitialied,
    /// Component is healthy
    Healthy,
    /// Component is degraded but functional
    Degraded,
    /// Component is unhealthy
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Uninitialied => write!(f, "uninitialized"),
        }
    }
}

/// Component health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Current health status
    pub status: HealthStatus,
    /// Last check time (ms since boot)
    pub last_check_ms: Option<u64>,
    /// Check interval (ms)
    pub check_interval_ms: u64,
    /// Failure count
    pub failure_count: u32,
    /// Last error message
    pub last_error: Option<String>,
    /// Response time (ms)
    pub response_time_ms: Option<u64>,
}

impl ComponentHealth {
    /// Create new component health
    pub fn new(name: impl Into<String>) -> Self {
        ComponentHealth {
            name: name.into(),
            status: HealthStatus::Uninitialied,
            last_check_ms: None,
            check_interval_ms: 5000,
            failure_count: 0,
            last_error: None,
            response_time_ms: None,
        }
    }

    /// Mark as healthy
    pub fn mark_healthy(mut self, response_time_ms: u64) -> Self {
        self.status = HealthStatus::Healthy;
        self.failure_count = 0;
        self.last_error = None;
        self.response_time_ms = Some(response_time_ms);
        self
    }

    /// Mark as degraded
    pub fn mark_degraded(mut self, error: impl Into<String>) -> Self {
        self.status = HealthStatus::Degraded;
        self.failure_count += 1;
        self.last_error = Some(error.into());
        self
    }

    /// Mark as unhealthy
    pub fn mark_unhealthy(mut self, error: impl Into<String>) -> Self {
        self.status = HealthStatus::Unhealthy;
        self.failure_count += 1;
        self.last_error = Some(error.into());
        self
    }

    /// Set check interval
    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.check_interval_ms = interval_ms;
        self
    }

    /// Check if needs health check
    pub fn needs_check(&self, current_ms: u64) -> bool {
        if let Some(last_check) = self.last_check_ms {
            current_ms - last_check >= self.check_interval_ms
        } else {
            true
        }
    }
}

/// Boot performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BootPerformanceMetrics {
    /// Phase timings (phase_name, ms)
    pub phase_timings: Vec<(String, u64)>,
    /// Component load times (component_name, ms)
    pub component_times: Vec<(String, u64)>,
    /// Resource utilization (cpu%, mem_mb, disk_mb)
    pub resource_usage: Option<(f64, u64, u64)>,
    /// Total boot time
    pub total_time_ms: u64,
}

impl BootPerformanceMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        BootPerformanceMetrics::default()
    }

    /// Add phase timing
    pub fn add_phase(mut self, name: impl Into<String>, time_ms: u64) -> Self {
        self.phase_timings.push((name.into(), time_ms));
        self
    }

    /// Add component time
    pub fn add_component(mut self, name: impl Into<String>, time_ms: u64) -> Self {
        self.component_times.push((name.into(), time_ms));
        self
    }

    /// Find slowest phase
    pub fn slowest_phase(&self) -> Option<(String, u64)> {
        self.phase_timings
            .iter()
            .max_by_key(|(_, time)| time)
            .cloned()
    }

    /// Find slowest component
    pub fn slowest_component(&self) -> Option<(String, u64)> {
        self.component_times
            .iter()
            .max_by_key(|(_, time)| time)
            .cloned()
    }

    /// Calculate average phase time
    pub fn avg_phase_time(&self) -> f64 {
        if self.phase_timings.is_empty() {
            return 0.0;
        }
        let total: u64 = self.phase_timings.iter().map(|(_, t)| t).sum();
        total as f64 / self.phase_timings.len() as f64
    }
}

/// Boot diagnostic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootDiagnosticReport {
    /// Reached tier
    pub reached_tier: BootTier,
    /// Boot success
    pub success: bool,
    /// Total time (ms)
    pub total_time_ms: u64,
    /// Component health status (name, status)
    pub component_health: Vec<(String, HealthStatus)>,
    /// Performance metrics
    pub performance: BootPerformanceMetrics,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Generated at (ms since boot)
    pub generated_at_ms: u64,
}

impl BootDiagnosticReport {
    /// Create new diagnostic report
    pub fn new(reached_tier: BootTier, total_time_ms: u64) -> Self {
        BootDiagnosticReport {
            reached_tier,
            success: true,
            total_time_ms,
            component_health: Vec::new(),
            performance: BootPerformanceMetrics::new(),
            recommendations: Vec::new(),
            generated_at_ms: 0,
        }
    }

    /// Add component health
    pub fn with_component(mut self, name: impl Into<String>, status: HealthStatus) -> Self {
        self.component_health.push((name.into(), status));
        self
    }

    /// Add recommendation
    pub fn with_recommendation(mut self, rec: impl Into<String>) -> Self {
        self.recommendations.push(rec.into());
        self
    }

    /// Count unhealthy components
    pub fn unhealthy_count(&self) -> usize {
        self.component_health
            .iter()
            .filter(|(_, status)| *status == HealthStatus::Unhealthy)
            .count()
    }

    /// Generate human-readable report
    pub fn to_string_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Boot Diagnostic Report\n");
        report.push_str("======================\n");
        report.push_str(&format!("Reached Tier: {:?}\n", self.reached_tier));
        report.push_str(&format!("Success: {}\n", self.success));
        report.push_str(&format!("Total Time: {}ms\n\n", self.total_time_ms));

        report.push_str("Component Health:\n");
        for (name, status) in &self.component_health {
            report.push_str(&format!("  {}: {}\n", name, status));
        }

        if let Some((phase, time)) = self.performance.slowest_phase() {
            report.push_str(&format!("\nSlowest Phase: {} ({}ms)\n", phase, time));
        }

        if let Some((component, time)) = self.performance.slowest_component() {
            report.push_str(&format!("Slowest Component: {} ({}ms)\n", component, time));
        }

        if !self.recommendations.is_empty() {
            report.push_str("\nRecommendations:\n");
            for (i, rec) in self.recommendations.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, rec));
            }
        }

        report
    }
}

/// Health check engine
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthCheckEngine {
    /// Component health states
    pub components: HashMap<String, ComponentHealth>,
    /// Overall system health
    pub overall_status: HealthStatus,
}

impl HealthCheckEngine {
    /// Create new health check engine
    pub fn new() -> Self {
        HealthCheckEngine {
            components: HashMap::new(),
            overall_status: HealthStatus::Uninitialied,
        }
    }

    /// Register component for health checking
    pub fn register(&mut self, health: ComponentHealth) {
        self.components.insert(health.name.clone(), health);
    }

    /// Update component health
    pub fn update_component(&mut self, name: &str, status: HealthStatus) {
        if let Some(comp) = self.components.get_mut(name) {
            comp.status = status;
        }
    }

    /// Get all unhealthy components
    pub fn unhealthy_components(&self) -> Vec<&str> {
        self.components
            .iter()
            .filter(|(_, health)| health.status == HealthStatus::Unhealthy)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Calculate overall health
    pub fn calculate_health(&mut self) {
        let total = self.components.len();
        if total == 0 {
            self.overall_status = HealthStatus::Uninitialied;
            return;
        }

        let unhealthy = self
            .components
            .values()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .count();
        let degraded = self
            .components
            .values()
            .filter(|c| c.status == HealthStatus::Degraded)
            .count();

        if unhealthy > 0 {
            self.overall_status = HealthStatus::Unhealthy;
        } else if degraded > 0 {
            self.overall_status = HealthStatus::Degraded;
        } else {
            self.overall_status = HealthStatus::Healthy;
        }
    }

    /// Generate health report
    pub fn health_report(&self) -> HealthReport {
        let healthy = self
            .components
            .values()
            .filter(|c| c.status == HealthStatus::Healthy)
            .count();
        let degraded = self
            .components
            .values()
            .filter(|c| c.status == HealthStatus::Degraded)
            .count();
        let unhealthy = self
            .components
            .values()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .count();

        HealthReport {
            overall_status: self.overall_status,
            healthy_count: healthy,
            degraded_count: degraded,
            unhealthy_count: unhealthy,
            total_components: self.components.len(),
        }
    }
}

/// Health report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall system health
    pub overall_status: HealthStatus,
    /// Number of healthy components
    pub healthy_count: usize,
    /// Number of degraded components
    pub degraded_count: usize,
    /// Number of unhealthy components
    pub unhealthy_count: usize,
    /// Total components
    pub total_components: usize,
}

impl HealthReport {
    /// Get health percentage
    pub fn health_percent(&self) -> f64 {
        if self.total_components == 0 {
            return 0.0;
        }
        (self.healthy_count as f64 / self.total_components as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health_creation() {
        let health = ComponentHealth::new("test_component");
        assert_eq!(health.status, HealthStatus::Uninitialied);
        assert_eq!(health.failure_count, 0);
    }

    #[test]
    fn test_component_health_mark_healthy() {
        let health = ComponentHealth::new("test").mark_healthy(100);
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.response_time_ms, Some(100));
        assert_eq!(health.failure_count, 0);
    }

    #[test]
    fn test_component_health_mark_degraded() {
        let health = ComponentHealth::new("test").mark_degraded("timeout");
        assert_eq!(health.status, HealthStatus::Degraded);
        assert_eq!(health.failure_count, 1);
        assert!(health.last_error.is_some());
    }

    #[test]
    fn test_component_health_mark_unhealthy() {
        let health = ComponentHealth::new("test").mark_unhealthy("connection failed");
        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert_eq!(health.failure_count, 1);
    }

    #[test]
    fn test_component_health_needs_check() {
        let health = ComponentHealth::new("test").with_interval(5000);
        assert!(health.needs_check(0));
        assert!(health.needs_check(10000));
    }

    #[test]
    fn test_boot_performance_metrics() {
        let metrics = BootPerformanceMetrics::new()
            .add_phase("preflight", 100)
            .add_phase("config", 200)
            .add_component("network", 150)
            .add_component("ssh", 200);

        assert_eq!(metrics.phase_timings.len(), 2);
        assert_eq!(metrics.component_times.len(), 2);
    }

    #[test]
    fn test_slowest_phase() {
        let metrics = BootPerformanceMetrics::new()
            .add_phase("preflight", 100)
            .add_phase("config", 500)
            .add_phase("setup", 200);

        let (name, time) = metrics.slowest_phase().unwrap();
        assert_eq!(name, "config");
        assert_eq!(time, 500);
    }

    #[test]
    fn test_avg_phase_time() {
        let metrics = BootPerformanceMetrics::new()
            .add_phase("p1", 100)
            .add_phase("p2", 200)
            .add_phase("p3", 300);

        assert_eq!(metrics.avg_phase_time(), 200.0);
    }

    #[test]
    fn test_boot_diagnostic_report() {
        let report = BootDiagnosticReport::new(BootTier::Usable, 5000)
            .with_component("config", HealthStatus::Healthy)
            .with_component("health", HealthStatus::Degraded)
            .with_recommendation("Upgrade health monitor");

        assert_eq!(report.reached_tier, BootTier::Usable);
        assert_eq!(report.component_health.len(), 2);
        assert_eq!(report.unhealthy_count(), 0);
    }

    #[test]
    fn test_health_check_engine() {
        let mut engine = HealthCheckEngine::new();
        engine.register(ComponentHealth::new("api").mark_healthy(100));
        engine.register(ComponentHealth::new("db").mark_unhealthy("connection failed"));

        engine.calculate_health();
        assert_eq!(engine.overall_status, HealthStatus::Unhealthy);
        assert_eq!(engine.unhealthy_components().len(), 1);
    }

    #[test]
    fn test_health_report() {
        let mut engine = HealthCheckEngine::new();
        engine.register(ComponentHealth::new("api").mark_healthy(100));
        engine.register(ComponentHealth::new("db").mark_healthy(150));

        let report = engine.health_report();
        assert_eq!(report.healthy_count, 2);
        assert_eq!(report.health_percent(), 100.0);
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
    }

    #[test]
    fn test_diagnostic_report_string() {
        let report = BootDiagnosticReport::new(BootTier::Full, 120000)
            .with_component("api", HealthStatus::Healthy)
            .with_recommendation("All systems nominal");

        let text = report.to_string_report();
        assert!(text.contains("Boot Diagnostic Report"));
        assert!(text.contains("api: healthy"));
    }
}
