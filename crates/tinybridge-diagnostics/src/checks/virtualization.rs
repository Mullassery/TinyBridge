use super::DiagnosticCheck;
use crate::result::{CheckResult, DiagnosticSeverity};
use async_trait::async_trait;

pub struct VirtualizationCheck;

#[async_trait]
impl DiagnosticCheck for VirtualizationCheck {
    async fn run(&self) -> CheckResult {
        // Check if running on macOS
        if cfg!(target_os = "macos") {
            // Check if Apple Virtualization Framework is available (basic check)
            let apple_silicon = check_apple_silicon();

            if apple_silicon {
                CheckResult::new(
                    "Apple Virtualization Framework",
                    DiagnosticSeverity::Pass,
                    "Apple Virtualization Framework available on Apple Silicon",
                )
                .with_details("macOS running on ARM64 architecture with VZ support")
            } else {
                CheckResult::new(
                    "Apple Virtualization Framework",
                    DiagnosticSeverity::Warning,
                    "Apple Virtualization Framework available but on Intel architecture",
                )
                .with_details("TinyBridge is optimized for Apple Silicon (M1/M2/M3+)")
                .with_recommendation(
                    "Consider upgrading to Apple Silicon Mac for better performance",
                )
            }
        } else {
            CheckResult::new(
                "Apple Virtualization Framework",
                DiagnosticSeverity::Fail,
                "Not running on macOS",
            )
            .with_recommendation("TinyBridge requires macOS 13 or later")
        }
    }
}

fn check_apple_silicon() -> bool {
    cfg!(target_arch = "aarch64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_virtualization_check_runs() {
        let check = VirtualizationCheck;
        let result = check.run().await;
        assert!(!result.message.is_empty());
    }

    #[test]
    fn test_apple_silicon_detection() {
        let is_arm = check_apple_silicon();
        #[cfg(target_arch = "aarch64")]
        assert!(is_arm);
        #[cfg(not(target_arch = "aarch64"))]
        assert!(!is_arm);
    }
}
