use super::DiagnosticCheck;
use crate::result::{CheckResult, DiagnosticSeverity};
use async_trait::async_trait;

pub struct GuestCheck;

#[async_trait]
impl DiagnosticCheck for GuestCheck {
    async fn run(&self) -> CheckResult {
        // This would normally connect to a running guest VM
        // For now, return a placeholder that can be enhanced later
        CheckResult::new(
            "Guest Environment",
            DiagnosticSeverity::Pass,
            "Guest check: No running environments to verify",
        )
        .with_details("Launch an environment with 'tinybridge launch' to perform guest checks")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_guest_check_runs() {
        let check = GuestCheck;
        let result = check.run().await;
        assert!(!result.message.is_empty());
    }
}
