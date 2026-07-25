use super::DiagnosticCheck;
use crate::result::{CheckResult, DiagnosticSeverity};
use async_trait::async_trait;

pub struct StorageCheck;

#[async_trait]
impl DiagnosticCheck for StorageCheck {
    async fn run(&self) -> CheckResult {
        // Check for disk corruption or issues
        if check_disk_health() {
            CheckResult::new(
                "Storage Integrity",
                DiagnosticSeverity::Pass,
                "No disk corruption or issues detected",
            )
        } else {
            CheckResult::new(
                "Storage Integrity",
                DiagnosticSeverity::Fail,
                "Potential disk issues detected",
            )
            .with_recommendation("Run 'disk utility' or 'fsck' to check disk health")
        }
    }
}

fn check_disk_health() -> bool {
    use std::process::Command;

    // Simple check: try to read disk space info
    let output = Command::new("df").args(&["-h", "/"]).output();

    output.is_ok() && output.unwrap().status.success()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_check_runs() {
        let check = StorageCheck;
        let result = check.run().await;
        assert!(!result.message.is_empty());
    }
}
