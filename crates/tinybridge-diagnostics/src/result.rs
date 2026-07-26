use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub details: Option<String>,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    #[serde(rename = "pass")]
    Pass,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "fail")]
    Fail,
}

impl CheckResult {
    pub fn new(name: &str, severity: DiagnosticSeverity, message: &str) -> Self {
        CheckResult {
            name: name.to_string(),
            severity,
            message: message.to_string(),
            details: None,
            recommendation: None,
        }
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn with_recommendation(mut self, recommendation: &str) -> Self {
        self.recommendation = Some(recommendation.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_creation() {
        let result = CheckResult::new("test", DiagnosticSeverity::Pass, "All good");
        assert_eq!(result.name, "test");
        assert_eq!(result.severity, DiagnosticSeverity::Pass);
    }

    #[test]
    fn test_check_result_with_details() {
        let result = CheckResult::new("test", DiagnosticSeverity::Warning, "Warning")
            .with_details("Some details");
        assert_eq!(result.details, Some("Some details".to_string()));
    }

    #[test]
    fn test_check_result_with_recommendation() {
        let result = CheckResult::new("test", DiagnosticSeverity::Fail, "Failed")
            .with_recommendation("Fix this");
        assert_eq!(result.recommendation, Some("Fix this".to_string()));
    }
}
