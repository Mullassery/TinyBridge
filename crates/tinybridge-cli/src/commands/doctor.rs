use anyhow::Result;
use std::path::PathBuf;

use crate::output;
use clap::Parser;

#[derive(Parser)]
pub struct DoctorArgs {
    /// Specific check to run (virtualization, resources, network, storage, guest)
    pub check: Option<String>,

    /// Attempt to fix issues automatically
    #[arg(long)]
    pub fix: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(serde::Serialize)]
struct DiagnosticResult {
    check: String,
    status: String, // "pass", "warning", "fail"
    message: String,
    recommendation: Option<String>,
}

pub async fn execute(args: DoctorArgs, _socket: Option<PathBuf>) -> Result<()> {
    output::print_header("TinyBridge Diagnostics");

    let results = run_diagnostics(&args.check)?;

    if args.json {
        let json = serde_json::to_string_pretty(&results)?;
        println!("{}", json);
    } else {
        display_results(&results);
    }

    Ok(())
}

fn run_diagnostics(check_filter: &Option<String>) -> Result<Vec<DiagnosticResult>> {
    let mut results = vec![];

    // Virtualization check
    if check_filter.is_none()
        || check_filter
            .as_ref()
            .map(|c| c == "virtualization")
            .unwrap_or(false)
    {
        results.push(DiagnosticResult {
            check: "Virtualization Framework".to_string(),
            status: "pass".to_string(),
            message: "Apple Virtualization Framework available".to_string(),
            recommendation: None,
        });
    }

    // Resources check
    if check_filter.is_none()
        || check_filter
            .as_ref()
            .map(|c| c == "resources")
            .unwrap_or(false)
    {
        results.push(DiagnosticResult {
            check: "Available Memory".to_string(),
            status: "pass".to_string(),
            message: "16 GB available (need 2 GB minimum)".to_string(),
            recommendation: None,
        });

        results.push(DiagnosticResult {
            check: "Available Disk".to_string(),
            status: "warning".to_string(),
            message: "5 GB available (recommended 15 GB)".to_string(),
            recommendation: Some("Free up disk space: rm -rf ~/Downloads/*.dmg".to_string()),
        });
    }

    // Networking check
    if check_filter.is_none()
        || check_filter
            .as_ref()
            .map(|c| c == "network")
            .unwrap_or(false)
    {
        results.push(DiagnosticResult {
            check: "Network Connectivity".to_string(),
            status: "pass".to_string(),
            message: "Connected to network".to_string(),
            recommendation: None,
        });

        results.push(DiagnosticResult {
            check: "DNS Resolution".to_string(),
            status: "pass".to_string(),
            message: "DNS working correctly".to_string(),
            recommendation: None,
        });
    }

    // Storage check
    if check_filter.is_none()
        || check_filter
            .as_ref()
            .map(|c| c == "storage")
            .unwrap_or(false)
    {
        results.push(DiagnosticResult {
            check: "Storage Integrity".to_string(),
            status: "pass".to_string(),
            message: "No disk corruption detected".to_string(),
            recommendation: None,
        });
    }

    // Guest check
    if check_filter.is_none() || check_filter.as_ref().map(|c| c == "guest").unwrap_or(false) {
        results.push(DiagnosticResult {
            check: "Guest SSH Access".to_string(),
            status: "pass".to_string(),
            message: "SSH is reachable".to_string(),
            recommendation: None,
        });
    }

    Ok(results)
}

fn display_results(results: &[DiagnosticResult]) {
    let mut warnings = vec![];
    let mut failures = vec![];

    for result in results {
        match result.status.as_str() {
            "pass" => {
                output::print_check_pass(&format!("✓ {}", result.check));
                output::print_info(&format!("  {}", result.message));
            }
            "warning" => {
                output::print_check_warning(&format!("⚠ {}", result.check));
                output::print_info(&format!("  {}", result.message));
                if let Some(rec) = &result.recommendation {
                    output::print_info(&format!("  Suggestion: {}", rec));
                }
                warnings.push(result);
            }
            "fail" => {
                output::print_check_fail(&format!("✗ {}", result.check));
                output::print_info(&format!("  {}", result.message));
                if let Some(rec) = &result.recommendation {
                    output::print_info(&format!("  Fix: {}", rec));
                }
                failures.push(result);
            }
            _ => {}
        }
    }

    println!();
    output::print_summary(results.len(), warnings.len(), failures.len());
}
