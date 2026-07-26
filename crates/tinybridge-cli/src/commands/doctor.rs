use anyhow::Result;
use std::path::PathBuf;
use tinybridge_diagnostics::{CheckType, DiagnosticRunner};

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

pub async fn execute(args: DoctorArgs, _socket: Option<PathBuf>) -> Result<()> {
    output::print_header("TinyBridge Diagnostics");

    let runner = DiagnosticRunner::new();

    let check_type = match args.check.as_deref() {
        Some("virtualization") => CheckType::Virtualization,
        Some("resources") => CheckType::Resources,
        Some("network") => CheckType::Network,
        Some("storage") => CheckType::Storage,
        Some("guest") => CheckType::Guest,
        _ => CheckType::All,
    };

    let report = runner.run(check_type).await?;

    if args.json {
        let json = serde_json::to_string_pretty(&report)?;
        println!("{}", json);
    } else {
        display_report(&report);
    }

    Ok(())
}

fn display_report(report: &tinybridge_diagnostics::DiagnosticReport) {
    for check in &report.checks {
        use tinybridge_diagnostics::DiagnosticSeverity;

        match check.severity {
            DiagnosticSeverity::Pass => {
                output::print_check_pass(&format!("✓ {}", check.name));
            }
            DiagnosticSeverity::Warning => {
                output::print_check_warning(&format!("⚠ {}", check.name));
            }
            DiagnosticSeverity::Fail => {
                output::print_check_fail(&format!("✗ {}", check.name));
            }
        }

        output::print_info(&format!("  {}", check.message));

        if let Some(details) = &check.details {
            output::print_info(&format!("  Details: {}", details));
        }

        if let Some(recommendation) = &check.recommendation {
            output::print_info(&format!("  Suggestion: {}", recommendation));
        }
    }

    println!();
    output::print_summary(
        report.summary.total_checks,
        report.summary.warnings,
        report.summary.failures,
    );
}
