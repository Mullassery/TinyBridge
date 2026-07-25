pub mod checks;
pub mod result;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use result::{CheckResult, DiagnosticSeverity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckType {
    Virtualization,
    Resources,
    Network,
    Storage,
    Guest,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub timestamp: String,
    pub checks: Vec<CheckResult>,
    pub summary: DiagnosticSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    pub total_checks: usize,
    pub passed: usize,
    pub warnings: usize,
    pub failures: usize,
    pub status: DiagnosticStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticStatus {
    Healthy,
    Warning,
    Error,
}

pub struct DiagnosticRunner {
    checks: HashMap<String, Box<dyn checks::DiagnosticCheck>>,
}

impl DiagnosticRunner {
    pub fn new() -> Self {
        let mut checks: HashMap<String, Box<dyn checks::DiagnosticCheck>> = HashMap::new();

        // Register all diagnostic checks
        checks.insert(
            "virtualization".to_string(),
            Box::new(checks::virtualization::VirtualizationCheck),
        );
        checks.insert(
            "resources".to_string(),
            Box::new(checks::resources::ResourcesCheck),
        );
        checks.insert(
            "network".to_string(),
            Box::new(checks::network::NetworkCheck),
        );
        checks.insert(
            "storage".to_string(),
            Box::new(checks::storage::StorageCheck),
        );
        checks.insert("guest".to_string(), Box::new(checks::guest::GuestCheck));

        DiagnosticRunner { checks }
    }

    pub async fn run(&self, check_type: CheckType) -> Result<DiagnosticReport> {
        let mut results = Vec::new();
        let mut passed = 0;
        let mut warnings = 0;
        let mut failures = 0;

        let checks_to_run: Vec<&str> = match check_type {
            CheckType::All => vec!["virtualization", "resources", "network", "storage", "guest"],
            CheckType::Virtualization => vec!["virtualization"],
            CheckType::Resources => vec!["resources"],
            CheckType::Network => vec!["network"],
            CheckType::Storage => vec!["storage"],
            CheckType::Guest => vec!["guest"],
        };

        for check_name in checks_to_run {
            if let Some(check) = self.checks.get(check_name) {
                let result = check.run().await;
                match result.severity {
                    DiagnosticSeverity::Pass => passed += 1,
                    DiagnosticSeverity::Warning => warnings += 1,
                    DiagnosticSeverity::Fail => failures += 1,
                }
                results.push(result);
            }
        }

        let total_checks = results.len();
        let status = match failures {
            0 if warnings == 0 => DiagnosticStatus::Healthy,
            0 => DiagnosticStatus::Warning,
            _ => DiagnosticStatus::Error,
        };

        Ok(DiagnosticReport {
            timestamp: chrono::Local::now().to_rfc3339(),
            checks: results,
            summary: DiagnosticSummary {
                total_checks,
                passed,
                warnings,
                failures,
                status,
            },
        })
    }
}

impl Default for DiagnosticRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_diagnostic_runner_creation() {
        let runner = DiagnosticRunner::new();
        assert_eq!(runner.checks.len(), 5);
    }

    #[tokio::test]
    async fn test_diagnostic_report_all_checks() {
        let runner = DiagnosticRunner::new();
        let report = runner.run(CheckType::All).await.unwrap();
        assert_eq!(report.checks.len(), 5);
        assert!(report.summary.total_checks > 0);
    }

    #[tokio::test]
    async fn test_diagnostic_report_single_check() {
        let runner = DiagnosticRunner::new();
        let report = runner.run(CheckType::Virtualization).await.unwrap();
        assert!(report.checks.len() >= 1);
    }

    #[test]
    fn test_diagnostic_summary_calculation() {
        let summary = DiagnosticSummary {
            total_checks: 5,
            passed: 4,
            warnings: 1,
            failures: 0,
            status: DiagnosticStatus::Warning,
        };
        assert_eq!(summary.passed, 4);
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.status, DiagnosticStatus::Warning);
    }
}
