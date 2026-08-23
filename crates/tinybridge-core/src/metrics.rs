/// Prometheus Metrics Export
/// Phase 4.0.2: OTel Integration
///
/// Metrics collection for boot performance, resource usage, and daemon health
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prometheus metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Counter => write!(f, "counter"),
            MetricType::Gauge => write!(f, "gauge"),
            MetricType::Histogram => write!(f, "histogram"),
            MetricType::Summary => write!(f, "summary"),
        }
    }
}

/// Single metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Current value
    pub value: f64,
    /// Labels for dimensional data
    pub labels: HashMap<String, String>,
    /// Unit if applicable
    pub unit: Option<String>,
}

impl MetricValue {
    /// Create a new metric
    pub fn new(name: impl Into<String>, metric_type: MetricType, value: f64) -> Self {
        MetricValue {
            name: name.into(),
            metric_type,
            value,
            labels: HashMap::new(),
            unit: None,
        }
    }

    /// Add a label
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set unit
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Format as Prometheus line protocol
    pub fn to_prometheus_line(&self) -> String {
        let mut line = format!("{}", self.name);

        if !self.labels.is_empty() {
            line.push('{');
            let label_strs: Vec<String> = self
                .labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();
            line.push_str(&label_strs.join(","));
            line.push('}');
        }

        line.push_str(&format!(" {}", self.value));
        line
    }
}

/// Boot metrics collection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BootMetrics {
    /// Total boot time in milliseconds
    pub boot_time_ms: f64,
    /// Preflight duration in milliseconds
    pub preflight_ms: f64,
    /// Config load duration in milliseconds
    pub config_load_ms: f64,
    /// Profile selection duration in milliseconds
    pub profile_select_ms: f64,
    /// Resource allocation duration in milliseconds
    pub resource_alloc_ms: f64,
    /// Network setup duration in milliseconds
    pub network_setup_ms: f64,
    /// Daemon init duration in milliseconds
    pub daemon_init_ms: f64,
    /// Health monitor startup duration in milliseconds
    pub health_monitor_ms: f64,
    /// API server startup duration in milliseconds
    pub api_server_ms: f64,
    /// Profile being used
    pub profile: String,
    /// CPU cores allocated
    pub cpus: u32,
    /// Memory in GB
    pub memory: u32,
    /// Disk in GB
    pub disk: u32,
    /// GPU enabled
    pub gpu_enabled: bool,
    /// Boot successful
    pub success: bool,
}

impl BootMetrics {
    /// Create new empty boot metrics
    pub fn new() -> Self {
        BootMetrics::default()
    }

    /// Export as Prometheus metrics
    pub fn to_prometheus(&self) -> Vec<MetricValue> {
        vec![
            MetricValue::new(
                "tinybridge_boot_time_ms",
                MetricType::Gauge,
                self.boot_time_ms,
            )
            .with_unit("milliseconds"),
            MetricValue::new(
                "tinybridge_preflight_duration_ms",
                MetricType::Gauge,
                self.preflight_ms,
            )
            .with_unit("milliseconds"),
            MetricValue::new(
                "tinybridge_config_load_duration_ms",
                MetricType::Gauge,
                self.config_load_ms,
            )
            .with_unit("milliseconds"),
            MetricValue::new(
                "tinybridge_profile_select_duration_ms",
                MetricType::Gauge,
                self.profile_select_ms,
            )
            .with_unit("milliseconds"),
            MetricValue::new(
                "tinybridge_resource_alloc_duration_ms",
                MetricType::Gauge,
                self.resource_alloc_ms,
            )
            .with_unit("milliseconds"),
            MetricValue::new(
                "tinybridge_network_setup_duration_ms",
                MetricType::Gauge,
                self.network_setup_ms,
            )
            .with_unit("milliseconds"),
            MetricValue::new(
                "tinybridge_daemon_init_duration_ms",
                MetricType::Gauge,
                self.daemon_init_ms,
            )
            .with_unit("milliseconds"),
            MetricValue::new(
                "tinybridge_health_monitor_duration_ms",
                MetricType::Gauge,
                self.health_monitor_ms,
            )
            .with_unit("milliseconds"),
            MetricValue::new(
                "tinybridge_api_server_duration_ms",
                MetricType::Gauge,
                self.api_server_ms,
            )
            .with_unit("milliseconds"),
            MetricValue::new(
                "tinybridge_config_cpus",
                MetricType::Gauge,
                self.cpus as f64,
            )
            .with_label("profile", &self.profile),
            MetricValue::new(
                "tinybridge_config_memory_gb",
                MetricType::Gauge,
                self.memory as f64,
            )
            .with_label("profile", &self.profile),
            MetricValue::new(
                "tinybridge_config_disk_gb",
                MetricType::Gauge,
                self.disk as f64,
            )
            .with_label("profile", &self.profile),
            MetricValue::new(
                "tinybridge_boot_success",
                MetricType::Gauge,
                if self.success { 1.0 } else { 0.0 },
            ),
        ]
    }

    /// Serialize as Prometheus text format
    pub fn to_prometheus_text(&self) -> String {
        let metrics = self.to_prometheus();
        metrics
            .iter()
            .map(|m| m.to_prometheus_line())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Resource metrics (CPU, memory, disk)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// Disk usage in MB
    pub disk_mb: f64,
    /// Network bytes in
    pub network_in_bytes: u64,
    /// Network bytes out
    pub network_out_bytes: u64,
}

impl ResourceMetrics {
    /// Create new resource metrics
    pub fn new() -> Self {
        ResourceMetrics::default()
    }

    /// Export as Prometheus metrics
    pub fn to_prometheus(&self) -> Vec<MetricValue> {
        vec![
            MetricValue::new(
                "tinybridge_cpu_usage_percent",
                MetricType::Gauge,
                self.cpu_percent,
            )
            .with_unit("percent"),
            MetricValue::new(
                "tinybridge_memory_usage_mb",
                MetricType::Gauge,
                self.memory_mb,
            )
            .with_unit("megabytes"),
            MetricValue::new("tinybridge_disk_usage_mb", MetricType::Gauge, self.disk_mb)
                .with_unit("megabytes"),
            MetricValue::new(
                "tinybridge_network_in_bytes",
                MetricType::Counter,
                self.network_in_bytes as f64,
            )
            .with_unit("bytes"),
            MetricValue::new(
                "tinybridge_network_out_bytes",
                MetricType::Counter,
                self.network_out_bytes as f64,
            )
            .with_unit("bytes"),
        ]
    }
}

/// Metrics registry for collecting all metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsRegistry {
    /// Boot metrics
    pub boot: Option<BootMetrics>,
    /// Resource metrics
    pub resources: Option<ResourceMetrics>,
    /// Custom metrics
    pub custom: Vec<MetricValue>,
}

impl MetricsRegistry {
    /// Create new metrics registry
    pub fn new() -> Self {
        MetricsRegistry::default()
    }

    /// Export all metrics as Prometheus text
    pub fn to_prometheus_text(&self) -> String {
        let mut output = String::new();

        if let Some(boot) = &self.boot {
            output.push_str(&boot.to_prometheus_text());
            output.push('\n');
        }

        if let Some(resources) = &self.resources {
            let lines: Vec<String> = resources
                .to_prometheus()
                .iter()
                .map(|m| m.to_prometheus_line())
                .collect();
            output.push_str(&lines.join("\n"));
            output.push('\n');
        }

        for metric in &self.custom {
            output.push_str(&metric.to_prometheus_line());
            output.push('\n');
        }

        output
    }

    /// Add custom metric
    pub fn add_metric(&mut self, metric: MetricValue) {
        self.custom.push(metric);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_value_creation() {
        let metric = MetricValue::new("test_metric", MetricType::Gauge, 42.0);
        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.metric_type, MetricType::Gauge);
        assert_eq!(metric.value, 42.0);
    }

    #[test]
    fn test_metric_value_with_labels() {
        let metric = MetricValue::new("http_requests", MetricType::Counter, 100.0)
            .with_label("method", "GET")
            .with_label("status", "200");

        assert_eq!(metric.labels.get("method"), Some(&"GET".to_string()));
        assert_eq!(metric.labels.get("status"), Some(&"200".to_string()));
    }

    #[test]
    fn test_metric_prometheus_line() {
        let metric = MetricValue::new("test_metric", MetricType::Gauge, 42.0);
        let line = metric.to_prometheus_line();
        assert_eq!(line, "test_metric 42");
    }

    #[test]
    fn test_metric_prometheus_line_with_labels() {
        let metric = MetricValue::new("http_requests", MetricType::Counter, 100.0)
            .with_label("method", "GET")
            .with_label("status", "200");

        let line = metric.to_prometheus_line();
        assert!(line.contains("http_requests{"));
        assert!(line.contains("method=\"GET\""));
        assert!(line.contains("status=\"200\""));
        assert!(line.contains(" 100"));
    }

    #[test]
    fn test_boot_metrics_creation() {
        let metrics = BootMetrics {
            boot_time_ms: 1234.0,
            preflight_ms: 100.0,
            config_load_ms: 200.0,
            profile: "development".to_string(),
            cpus: 4,
            memory: 8,
            disk: 40,
            gpu_enabled: false,
            success: true,
            ..Default::default()
        };

        assert_eq!(metrics.boot_time_ms, 1234.0);
        assert_eq!(metrics.profile, "development");
    }

    #[test]
    fn test_boot_metrics_to_prometheus() {
        let metrics = BootMetrics {
            boot_time_ms: 1500.0,
            preflight_ms: 100.0,
            cpus: 4,
            profile: "production".to_string(),
            success: true,
            ..Default::default()
        };

        let prometheus = metrics.to_prometheus();
        assert!(prometheus.len() > 0);
        assert!(prometheus
            .iter()
            .any(|m| m.name == "tinybridge_boot_time_ms" && m.value == 1500.0));
    }

    #[test]
    fn test_boot_metrics_prometheus_text() {
        let metrics = BootMetrics {
            boot_time_ms: 1234.0,
            profile: "testing".to_string(),
            success: true,
            ..Default::default()
        };

        let text = metrics.to_prometheus_text();
        assert!(text.contains("tinybridge_boot_time_ms"));
        assert!(text.contains("1234"));
    }

    #[test]
    fn test_resource_metrics() {
        let metrics = ResourceMetrics {
            cpu_percent: 25.5,
            memory_mb: 512.0,
            disk_mb: 1024.0,
            network_in_bytes: 1000000,
            network_out_bytes: 500000,
        };

        let prometheus = metrics.to_prometheus();
        assert_eq!(prometheus.len(), 5);
    }

    #[test]
    fn test_metrics_registry() {
        let mut registry = MetricsRegistry::new();

        registry.boot = Some(BootMetrics {
            boot_time_ms: 1000.0,
            profile: "production".to_string(),
            success: true,
            ..Default::default()
        });

        registry.add_metric(MetricValue::new("custom_metric", MetricType::Gauge, 99.0));

        let text = registry.to_prometheus_text();
        assert!(text.contains("tinybridge_boot_time_ms"));
        assert!(text.contains("custom_metric"));
    }
}
