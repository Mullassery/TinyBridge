use crate::okf_updater::ProductionMetrics;
use serde_json::json;

/// Quality gate definition
#[derive(Debug, Clone)]
pub struct QualityGate {
    pub name: String,
    pub description: String,
    pub metric_name: String,
    pub threshold: f64,
    pub comparison: GateComparison,
    pub required: bool,         // Must pass before shipping
    pub blocker_for_ship: bool, // Blocks entire Phase if fails
}

/// How to compare metric against threshold
#[derive(Debug, Clone, PartialEq)]
pub enum GateComparison {
    LessThan,    // metric < threshold (for latency, error rate)
    GreaterThan, // metric > threshold (for availability, throughput)
}

/// Quality gate validation result
#[derive(Debug, Clone)]
pub struct GateResult {
    pub gate: QualityGate,
    pub passed: bool,
    pub current_value: f64,
    pub margin: f64, // How far above/below threshold
}

/// Quality gates validator for Phase 1 Week 4
pub struct QualityGatesValidator;

impl QualityGatesValidator {
    pub fn phase_1_week_4_gates() -> Vec<QualityGate> {
        vec![
            // Boot time SLO (critical)
            QualityGate {
                name: "boot_time_slo".to_string(),
                description: "Boot time under 5 seconds (SSH ready)".to_string(),
                metric_name: "boot_time_ms".to_string(),
                threshold: 5000.0,
                comparison: GateComparison::LessThan,
                required: true,
                blocker_for_ship: true,
            },
            // Error rate SLO
            QualityGate {
                name: "error_rate_slo".to_string(),
                description: "Error rate under 0.1%".to_string(),
                metric_name: "error_rate".to_string(),
                threshold: 0.1,
                comparison: GateComparison::LessThan,
                required: true,
                blocker_for_ship: false,
            },
            // Availability SLO
            QualityGate {
                name: "availability_slo".to_string(),
                description: "Availability over 99.9%".to_string(),
                metric_name: "availability_percent".to_string(),
                threshold: 99.9,
                comparison: GateComparison::GreaterThan,
                required: true,
                blocker_for_ship: true,
            },
            // Resource constraints
            QualityGate {
                name: "cpu_utilization".to_string(),
                description: "CPU utilization under 90%".to_string(),
                metric_name: "cpu_usage_percent".to_string(),
                threshold: 90.0,
                comparison: GateComparison::LessThan,
                required: false,
                blocker_for_ship: false,
            },
            QualityGate {
                name: "memory_utilization".to_string(),
                description: "Memory utilization under 90%".to_string(),
                metric_name: "memory_usage_percent".to_string(),
                threshold: 90.0,
                comparison: GateComparison::LessThan,
                required: false,
                blocker_for_ship: false,
            },
        ]
    }

    pub fn validate_metrics(metrics: &ProductionMetrics, gates: &[QualityGate]) -> Vec<GateResult> {
        gates
            .iter()
            .map(|gate| Self::validate_gate(metrics, gate))
            .collect()
    }

    fn validate_gate(metrics: &ProductionMetrics, gate: &QualityGate) -> GateResult {
        let current_value = Self::get_metric_value(metrics, &gate.metric_name);
        let passed = Self::compare_value(current_value, gate.threshold, &gate.comparison);
        let margin = Self::calculate_margin(current_value, gate.threshold, &gate.comparison);

        GateResult {
            gate: gate.clone(),
            passed,
            current_value,
            margin,
        }
    }

    fn get_metric_value(metrics: &ProductionMetrics, metric_name: &str) -> f64 {
        match metric_name {
            "boot_time_ms" => metrics.boot_time_ms as f64,
            "error_rate" => metrics.error_rate as f64,
            "availability_percent" => metrics.availability_percent as f64,
            "cpu_usage_percent" => metrics.cpu_usage_percent as f64,
            "memory_usage_percent" => metrics.memory_usage_percent as f64,
            _ => 0.0,
        }
    }

    fn compare_value(current: f64, threshold: f64, comparison: &GateComparison) -> bool {
        match comparison {
            GateComparison::LessThan => current < threshold,
            GateComparison::GreaterThan => current > threshold,
        }
    }

    fn calculate_margin(current: f64, threshold: f64, comparison: &GateComparison) -> f64 {
        match comparison {
            GateComparison::LessThan => threshold - current, // Positive = margin to breach
            GateComparison::GreaterThan => current - threshold, // Positive = margin above target
        }
    }

    pub fn are_all_required_passing(results: &[GateResult]) -> bool {
        results.iter().filter(|r| r.gate.required).all(|r| r.passed)
    }

    pub fn are_blockers_passing(results: &[GateResult]) -> bool {
        results
            .iter()
            .filter(|r| r.gate.blocker_for_ship)
            .all(|r| r.passed)
    }

    pub fn export_json(results: &[GateResult]) -> serde_json::Value {
        let gates: Vec<_> = results
            .iter()
            .map(|r| {
                json!({
                    "name": r.gate.name,
                    "description": r.gate.description,
                    "passed": r.passed,
                    "current_value": r.current_value,
                    "threshold": r.gate.threshold,
                    "margin": r.margin,
                    "required": r.gate.required,
                    "blocker": r.gate.blocker_for_ship,
                    "status": if r.passed { "PASS" } else { "FAIL" }
                })
            })
            .collect();

        json!({
            "gates": gates,
            "all_required_passing": Self::are_all_required_passing(results),
            "blockers_passing": Self::are_blockers_passing(results),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_metrics() -> ProductionMetrics {
        ProductionMetrics {
            timestamp: Utc::now(),
            env_name: "test".to_string(),
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
    fn test_boot_time_gate_pass() {
        let metrics = create_test_metrics();
        let gates = QualityGatesValidator::phase_1_week_4_gates();
        let results = QualityGatesValidator::validate_metrics(&metrics, &gates);

        let boot_gate = results
            .iter()
            .find(|r| r.gate.name == "boot_time_slo")
            .unwrap();
        assert!(boot_gate.passed);
        assert!(boot_gate.margin > 0.0); // Has margin to 5000ms
    }

    #[test]
    fn test_boot_time_gate_fail() {
        let mut metrics = create_test_metrics();
        metrics.boot_time_ms = 5500; // Over limit
        let gates = QualityGatesValidator::phase_1_week_4_gates();
        let results = QualityGatesValidator::validate_metrics(&metrics, &gates);

        let boot_gate = results
            .iter()
            .find(|r| r.gate.name == "boot_time_slo")
            .unwrap();
        assert!(!boot_gate.passed);
    }

    #[test]
    fn test_availability_gate() {
        let metrics = create_test_metrics();
        let gates = QualityGatesValidator::phase_1_week_4_gates();
        let results = QualityGatesValidator::validate_metrics(&metrics, &gates);

        let avail_gate = results
            .iter()
            .find(|r| r.gate.name == "availability_slo")
            .unwrap();
        assert!(avail_gate.passed);
    }

    #[test]
    fn test_all_required_passing() {
        let metrics = create_test_metrics();
        let gates = QualityGatesValidator::phase_1_week_4_gates();
        let results = QualityGatesValidator::validate_metrics(&metrics, &gates);

        assert!(QualityGatesValidator::are_all_required_passing(&results));
    }

    #[test]
    fn test_blockers_passing() {
        let metrics = create_test_metrics();
        let gates = QualityGatesValidator::phase_1_week_4_gates();
        let results = QualityGatesValidator::validate_metrics(&metrics, &gates);

        assert!(QualityGatesValidator::are_blockers_passing(&results));
    }
}
