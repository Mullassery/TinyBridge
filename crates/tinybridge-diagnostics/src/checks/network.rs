use super::DiagnosticCheck;
use crate::result::{CheckResult, DiagnosticSeverity};
use async_trait::async_trait;

pub struct NetworkCheck;

#[async_trait]
impl DiagnosticCheck for NetworkCheck {
    async fn run(&self) -> CheckResult {
        let connectivity = check_network_connectivity().await;
        let dns = check_dns_resolution().await;

        if connectivity && dns {
            CheckResult::new(
                "Network Connectivity",
                DiagnosticSeverity::Pass,
                "Network connectivity and DNS resolution working correctly",
            )
        } else {
            let mut issues = Vec::new();
            if !connectivity {
                issues.push("No network connectivity");
            }
            if !dns {
                issues.push("DNS resolution failing");
            }

            CheckResult::new(
                "Network Connectivity",
                DiagnosticSeverity::Warning,
                &issues.join(", "),
            )
            .with_recommendation(
                "Check network connection: ping 8.8.8.8 or check DNS: nslookup google.com",
            )
        }
    }
}

async fn check_network_connectivity() -> bool {
    use std::process::Command;

    let output = Command::new("ping")
        .args(["-c", "1", "-W", "2", "8.8.8.8"])
        .output();

    output.is_ok() && output.unwrap().status.success()
}

async fn check_dns_resolution() -> bool {
    use std::process::Command;

    let output = Command::new("nslookup").args(["google.com"]).output();

    output.is_ok() && output.unwrap().status.success()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_check_runs() {
        let check = NetworkCheck;
        let result = check.run().await;
        assert!(!result.message.is_empty());
    }
}
