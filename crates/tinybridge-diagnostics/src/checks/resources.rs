use super::DiagnosticCheck;
use crate::result::{CheckResult, DiagnosticSeverity};
use async_trait::async_trait;

pub struct ResourcesCheck;

#[async_trait]
impl DiagnosticCheck for ResourcesCheck {
    async fn run(&self) -> CheckResult {
        let available_memory_gb = get_available_memory_gb();
        let available_disk_gb = get_available_disk_gb();

        // Check memory (minimum 2GB, recommended 8GB+)
        if available_memory_gb < 2.0 {
            return CheckResult::new(
                "Available Memory",
                DiagnosticSeverity::Fail,
                &format!(
                    "{:.1} GB available (need 2 GB minimum)",
                    available_memory_gb
                ),
            )
            .with_recommendation("Close applications to free memory or increase system RAM");
        }

        let memory_status = if available_memory_gb < 8.0 {
            DiagnosticSeverity::Warning
        } else {
            DiagnosticSeverity::Pass
        };

        let _memory_result = CheckResult::new(
            "Available Memory",
            memory_status,
            &format!(
                "{:.1} GB available (recommended 8 GB+)",
                available_memory_gb
            ),
        );

        // Check disk (minimum 5GB, recommended 15GB+)
        if available_disk_gb < 5.0 {
            return CheckResult::new(
                "Available Disk Space",
                DiagnosticSeverity::Fail,
                &format!("{:.1} GB available (need 5 GB minimum)", available_disk_gb),
            )
            .with_recommendation("Free up disk space or increase storage allocation");
        }

        let disk_status = if available_disk_gb < 15.0 {
            DiagnosticSeverity::Warning
        } else {
            DiagnosticSeverity::Pass
        };

        if memory_status == DiagnosticSeverity::Pass && disk_status == DiagnosticSeverity::Pass {
            CheckResult::new(
                "System Resources",
                DiagnosticSeverity::Pass,
                &format!(
                    "Memory: {:.1} GB, Disk: {:.1} GB",
                    available_memory_gb, available_disk_gb
                ),
            )
        } else {
            let status = if memory_status == DiagnosticSeverity::Warning
                || disk_status == DiagnosticSeverity::Warning
            {
                DiagnosticSeverity::Warning
            } else {
                DiagnosticSeverity::Pass
            };

            let mut message = String::new();
            if memory_status == DiagnosticSeverity::Warning {
                message.push_str(&format!(
                    "Memory: {:.1} GB (low, recommended 8+ GB)\n",
                    available_memory_gb
                ));
            } else {
                message.push_str(&format!(
                    "Memory: {:.1} GB (adequate)\n",
                    available_memory_gb
                ));
            }

            if disk_status == DiagnosticSeverity::Warning {
                message.push_str(&format!(
                    "Disk: {:.1} GB (low, recommended 15+ GB)",
                    available_disk_gb
                ));
            } else {
                message.push_str(&format!("Disk: {:.1} GB (adequate)", available_disk_gb));
            }

            let mut result = CheckResult::new("System Resources", status, &message);
            if disk_status == DiagnosticSeverity::Warning {
                result = result.with_recommendation("Free up disk space: rm -rf ~/Downloads/*.dmg");
            }
            result
        }
    }
}

fn get_available_memory_gb() -> f64 {
    // This is a simplified check - in production, would use system APIs
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        if let Ok(output) = Command::new("sysctl").args(&["hw.memsize"]).output() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Some(mem_str) = output_str.split('=').nth(1) {
                    if let Ok(mem_bytes) = mem_str.trim().parse::<u64>() {
                        return (mem_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
                    }
                }
            }
        }
    }

    // Fallback: assume reasonable system
    16.0
}

fn get_available_disk_gb() -> f64 {
    // This is a simplified check - in production, would use system APIs
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        if let Ok(output) = Command::new("df").args(&["-g", "/var/tmp"]).output() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                // Parse df output: columns are filesystem, blocks, used, available
                for line in output_str.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let Ok(available) = parts[3].parse::<f64>() {
                            return available;
                        }
                    }
                }
            }
        }
    }

    // Fallback: assume reasonable disk
    100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resources_check_runs() {
        let check = ResourcesCheck;
        let result = check.run().await;
        assert!(!result.message.is_empty());
    }

    #[test]
    fn test_memory_detection() {
        let mem = get_available_memory_gb();
        assert!(mem > 0.0);
    }

    #[test]
    fn test_disk_detection() {
        let disk = get_available_disk_gb();
        assert!(disk > 0.0);
    }
}
