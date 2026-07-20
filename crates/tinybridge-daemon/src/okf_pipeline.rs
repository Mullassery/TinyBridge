use crate::anomaly_detector::{AnomalyDetector, Severity};
use crate::okf_updater::{OkfUpdater, ProductionMetrics};
use crate::quality_gates::QualityGatesValidator;
use anyhow::Result;

/// OKF Auto-Update Pipeline - complete system for production data flow
///
/// Pipeline stages:
/// 1. Collect OTel metrics from daemon
/// 2. Update OKF snapshot with live data
/// 3. Validate quality gates
/// 4. Detect anomalies (security + performance)
/// 5. Generate alerts if needed
pub struct OkfPipeline {
    updater: OkfUpdater,
    detector: AnomalyDetector,
    quality_validator: QualityGatesValidator,
}

pub struct PipelineResult {
    pub okf_updated: bool,
    pub quality_gates_passing: bool,
    pub anomalies_detected: usize,
    pub intrusion_likely: bool,
    pub summary: String,
}

impl OkfPipeline {
    pub fn new(okf_dir: std::path::PathBuf) -> Self {
        OkfPipeline {
            updater: OkfUpdater::new(okf_dir),
            detector: AnomalyDetector::new(),
            quality_validator: QualityGatesValidator,
        }
    }

    /// Run complete pipeline: metrics → OKF → quality gates → anomaly detection
    pub fn process_metrics(&mut self, metrics: ProductionMetrics) -> Result<PipelineResult> {
        // Stage 1: Update OKF with current metrics
        self.updater.update_from_metrics(metrics.clone())?;

        // Stage 2: Get OKF snapshot for further analysis
        let snapshot = self
            .updater
            .get_snapshot(&metrics.env_name)
            .ok_or_else(|| anyhow::anyhow!("Failed to get OKF snapshot"))?;

        // Stage 3: Validate quality gates
        let gates = QualityGatesValidator::phase_1_week_4_gates();
        let gate_results = QualityGatesValidator::validate_metrics(&metrics, &gates);
        let all_gates_pass = QualityGatesValidator::are_all_required_passing(&gate_results);
        let blockers_pass = QualityGatesValidator::are_blockers_passing(&gate_results);

        // Stage 4: Detect anomalies
        let mut metrics_window = crate::okf_updater::MetricsWindow::new(10);
        for historical_metric in &snapshot.history {
            metrics_window.add_sample(historical_metric.clone());
        }

        let anomalies = self.detector.detect_anomalies(&metrics_window);
        let intrusion_likely = self.detector.is_intrusion_likely(&anomalies);

        // Stage 5: Generate summary
        let summary = self.generate_summary(&metrics, all_gates_pass, blockers_pass, intrusion_likely, &anomalies);

        tracing::info!(
            env_name = metrics.env_name,
            quality_gates = all_gates_pass,
            blockers = blockers_pass,
            anomalies = anomalies.len(),
            intrusion_alert = intrusion_likely,
            "{}", summary
        );

        Ok(PipelineResult {
            okf_updated: true,
            quality_gates_passing: all_gates_pass,
            anomalies_detected: anomalies.len(),
            intrusion_likely,
            summary,
        })
    }

    fn generate_summary(
        &self,
        metrics: &ProductionMetrics,
        gates_pass: bool,
        blockers_pass: bool,
        intrusion: bool,
        anomalies: &[crate::anomaly_detector::Anomaly],
    ) -> String {
        let mut parts = Vec::new();

        // Boot time status
        parts.push(format!("Boot: {:.0}ms", metrics.boot_time_ms));

        // Quality gate status
        if gates_pass {
            parts.push("✓ Quality gates passing".to_string());
        } else {
            parts.push("✗ Quality gates FAILING".to_string());
        }

        // Blocker status
        if blockers_pass {
            parts.push("✓ Ship blockers OK".to_string());
        } else {
            parts.push("✗ Ship blockers BLOCKED".to_string());
        }

        // Anomaly summary
        if !anomalies.is_empty() {
            let critical_count = anomalies
                .iter()
                .filter(|a| a.severity == Severity::Critical)
                .count();
            parts.push(format!("⚠ {} anomalies ({} critical)", anomalies.len(), critical_count));
        }

        // Intrusion alert
        if intrusion {
            parts.push("🚨 INTRUSION LIKELY - INVESTIGATE IMMEDIATELY".to_string());
        }

        parts.join(" | ")
    }

    pub fn export_status(&self) -> serde_json::Value {
        self.updater.export_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_metrics() -> ProductionMetrics {
        ProductionMetrics {
            timestamp: chrono::Utc::now(),
            env_name: "test-env".to_string(),
            boot_time_ms: 4700,
            boot_tier: 1,
            cpu_usage_percent: 45.0,
            memory_usage_percent: 65.0,
            disk_usage_percent: 70.0,
            error_rate: 0.05,
            availability_percent: 99.95,
        }
    }

    #[test]
    fn test_pipeline_healthy_state() {
        let mut pipeline = OkfPipeline::new(PathBuf::from("/tmp/okf"));
        let metrics = create_test_metrics();

        let result = pipeline.process_metrics(metrics).unwrap();
        assert!(result.okf_updated);
        assert!(result.quality_gates_passing);
        assert!(!result.intrusion_likely);
    }

    #[test]
    fn test_pipeline_with_anomalies() {
        let mut pipeline = OkfPipeline::new(PathBuf::from("/tmp/okf"));
        let mut metrics = create_test_metrics();
        metrics.boot_time_ms = 5500; // Over SLO

        let result = pipeline.process_metrics(metrics).unwrap();
        assert!(result.okf_updated);
        assert!(!result.quality_gates_passing); // SLO breached
        assert!(result.anomalies_detected > 0);
    }
}
