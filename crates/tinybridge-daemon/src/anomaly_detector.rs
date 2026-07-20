use crate::okf_updater::{MetricsWindow, ProductionMetrics};

/// Anomaly type classification
#[derive(Debug, Clone, PartialEq)]
pub enum AnomalyType {
    /// Boot time regression (security flag + performance)
    BootTimeRegression,

    /// Unexpected resource usage spike (possible intrusion)
    ResourceSpike,

    /// Gradual resource drain (attack pattern)
    ResourceDrain,

    /// Hidden environment creation (intrusion detection)
    UnexpectedEnvironmentCount,

    /// Availability breach (SLO violation)
    AvailabilityBreach,

    /// Error rate anomaly
    ErrorRateSpike,
}

/// Severity level for anomaly
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Severity {
    Info = 1,
    Warning = 2,
    Critical = 3,
}

/// Detected anomaly with context
#[derive(Debug, Clone)]
pub struct Anomaly {
    pub anomaly_type: AnomalyType,
    pub severity: Severity,
    pub message: String,
    pub confidence: f32, // 0.0-1.0
    pub recommended_action: String,
}

/// Anomaly detector - identifies suspicious patterns
pub struct AnomalyDetector {
    boot_time_baseline: u64,
    cpu_baseline: f32,
    memory_baseline: f32,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        AnomalyDetector {
            boot_time_baseline: 4900, // SLO target
            cpu_baseline: 30.0,       // Expected average
            memory_baseline: 50.0,    // Expected average
        }
    }

    pub fn detect_anomalies(&self, window: &MetricsWindow) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        if window.samples.is_empty() {
            return anomalies;
        }

        let current = &window.samples[window.samples.len() - 1];

        // Check 1: Boot time regression
        if let Some(avg_boot_time) = window.average_boot_time() {
            if current.boot_time_ms > self.boot_time_baseline {
                let delta_ms = current.boot_time_ms - self.boot_time_baseline;
                let delta_pct = (delta_ms as f32 / self.boot_time_baseline as f32) * 100.0;

                anomalies.push(Anomaly {
                    anomaly_type: AnomalyType::BootTimeRegression,
                    severity: if delta_ms > 500 {
                        Severity::Critical
                    } else if delta_ms > 100 {
                        Severity::Warning
                    } else {
                        Severity::Info
                    },
                    message: format!(
                        "Boot time regression: {:.0}ms (target: {}ms, delta: +{:.1}%)",
                        current.boot_time_ms, self.boot_time_baseline, delta_pct
                    ),
                    confidence: 0.95,
                    recommended_action: "Investigate kernel changes or system overhead".to_string(),
                });
            }
        }

        // Check 2: CPU usage spike
        if current.cpu_usage_percent > self.cpu_baseline + 50.0 {
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::ResourceSpike,
                severity: if current.cpu_usage_percent > 95.0 {
                    Severity::Critical
                } else {
                    Severity::Warning
                },
                message: format!(
                    "CPU usage spike: {:.1}% (baseline: {:.1}%, delta: +{:.1}%)",
                    current.cpu_usage_percent,
                    self.cpu_baseline,
                    current.cpu_usage_percent - self.cpu_baseline
                ),
                confidence: 0.90,
                recommended_action: "Check for unexpected processes or hidden VMs".to_string(),
            });
        }

        // Check 3: Memory usage spike
        if current.memory_usage_percent > self.memory_baseline + 40.0 {
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::ResourceSpike,
                severity: if current.memory_usage_percent > 95.0 {
                    Severity::Critical
                } else {
                    Severity::Warning
                },
                message: format!(
                    "Memory usage spike: {:.1}% (baseline: {:.1}%, delta: +{:.1}%)",
                    current.memory_usage_percent,
                    self.memory_baseline,
                    current.memory_usage_percent - self.memory_baseline
                ),
                confidence: 0.88,
                recommended_action: "Monitor for memory leaks or unauthorized processes"
                    .to_string(),
            });
        }

        // Check 4: Gradual resource drain (trending upward)
        if window.samples.len() >= 3 {
            let trend = self.calculate_trend(window, |m| m.cpu_usage_percent);
            if trend > 2.0 {
                anomalies.push(Anomaly {
                    anomaly_type: AnomalyType::ResourceDrain,
                    severity: Severity::Warning,
                    message: format!(
                        "CPU trending upward: +{:.1}%/sample (potential resource drain)",
                        trend
                    ),
                    confidence: 0.75,
                    recommended_action: "Monitor for gradual attacks or runaway processes"
                        .to_string(),
                });
            }
        }

        // Check 5: Availability breach
        if current.availability_percent < 99.5 {
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::AvailabilityBreach,
                severity: if current.availability_percent < 99.0 {
                    Severity::Critical
                } else {
                    Severity::Warning
                },
                message: format!(
                    "Availability SLO breach: {:.2}% (target: 99.90%)",
                    current.availability_percent
                ),
                confidence: 0.99,
                recommended_action: "Investigate downtime events and error logs".to_string(),
            });
        }

        // Check 6: Error rate spike
        if current.error_rate > 0.1 {
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::ErrorRateSpike,
                severity: if current.error_rate > 1.0 {
                    Severity::Critical
                } else if current.error_rate > 0.5 {
                    Severity::Warning
                } else {
                    Severity::Info
                },
                message: format!(
                    "Error rate spike: {:.2}% (baseline: 0.05%)",
                    current.error_rate
                ),
                confidence: 0.92,
                recommended_action: "Check application logs for errors or service failures"
                    .to_string(),
            });
        }

        anomalies
    }

    fn calculate_trend(
        &self,
        window: &MetricsWindow,
        getter: impl Fn(&ProductionMetrics) -> f32,
    ) -> f32 {
        if window.samples.len() < 2 {
            return 0.0;
        }

        let mut total_delta = 0.0;
        for i in 1..window.samples.len() {
            let prev = getter(&window.samples[i - 1]);
            let curr = getter(&window.samples[i]);
            total_delta += curr - prev;
        }

        total_delta / (window.samples.len() as f32 - 1.0)
    }

    pub fn is_intrusion_likely(&self, anomalies: &[Anomaly]) -> bool {
        // Intrusion indicators:
        // 1. Multiple high-severity anomalies
        // 2. Resource spike + boot time regression
        // 3. Gradual resource drain over time

        let critical_count = anomalies
            .iter()
            .filter(|a| a.severity == Severity::Critical)
            .count();
        let resource_spike = anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::ResourceSpike);
        let boot_regression = anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::BootTimeRegression);

        critical_count >= 2 || (resource_spike && boot_regression)
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metrics(boot_ms: u64, cpu: f32, mem: f32) -> ProductionMetrics {
        ProductionMetrics {
            timestamp: chrono::Utc::now(),
            env_name: "test".to_string(),
            boot_time_ms: boot_ms,
            boot_tier: 1,
            cpu_usage_percent: cpu,
            memory_usage_percent: mem,
            disk_usage_percent: 50.0,
            error_rate: 0.05,
            availability_percent: 99.95,
        }
    }

    #[test]
    fn test_boot_time_regression_detection() {
        let detector = AnomalyDetector::new();
        let mut window = MetricsWindow::new(5);

        window.add_sample(create_test_metrics(4700, 30.0, 50.0));

        let anomalies = detector.detect_anomalies(&window);
        // Should not detect anomaly for healthy boot time
        assert!(anomalies.is_empty());

        // Add metric with regression
        window.add_sample(create_test_metrics(5200, 30.0, 50.0));
        let anomalies = detector.detect_anomalies(&window);
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::BootTimeRegression));
    }

    #[test]
    fn test_cpu_spike_detection() {
        let detector = AnomalyDetector::new();
        let mut window = MetricsWindow::new(5);

        window.add_sample(create_test_metrics(4700, 95.0, 50.0)); // 95% CPU is a spike

        let anomalies = detector.detect_anomalies(&window);
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::ResourceSpike));
    }

    #[test]
    fn test_intrusion_detection() {
        let detector = AnomalyDetector::new();

        // Single anomaly = not intrusion
        let anomalies = vec![Anomaly {
            anomaly_type: AnomalyType::ResourceSpike,
            severity: Severity::Critical,
            message: "test".to_string(),
            confidence: 0.9,
            recommended_action: "test".to_string(),
        }];
        assert!(!detector.is_intrusion_likely(&anomalies));

        // Multiple critical anomalies = likely intrusion
        let anomalies = vec![
            Anomaly {
                anomaly_type: AnomalyType::ResourceSpike,
                severity: Severity::Critical,
                message: "CPU spike".to_string(),
                confidence: 0.9,
                recommended_action: "test".to_string(),
            },
            Anomaly {
                anomaly_type: AnomalyType::BootTimeRegression,
                severity: Severity::Critical,
                message: "Boot regression".to_string(),
                confidence: 0.9,
                recommended_action: "test".to_string(),
            },
        ];
        assert!(detector.is_intrusion_likely(&anomalies));
    }
}
