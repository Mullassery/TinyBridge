use anyhow::Result;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::output;
use clap::Parser;
use serde_json::json;
use tinybridge_core::methods;

#[derive(Parser)]
pub struct RepairArgs {
    #[arg(help = "Environment name")]
    pub name: Option<String>,

    #[arg(long, help = "Verbose output")]
    pub verbose: bool,
}

pub async fn execute(args: RepairArgs, socket: Option<PathBuf>) -> Result<()> {
    let env_name = args.name.as_deref().unwrap_or("default");

    output::print_info(&format!("Repairing environment: {}", env_name));

    let mut client = DaemonClient::new(socket)?;

    let params = json!({
        "name": env_name,
    });

    let result = client.call(methods::ENVIRONMENT_REPAIR, params).await?;

    // Parse repair result
    if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
        match status {
            "repaired" => {
                output::print_success(&format!(
                    "✓ Environment '{}' repaired successfully",
                    env_name
                ));

                if let Some(validation) = result.get("validation").and_then(|v| v.as_str()) {
                    println!("\nValidation results:");
                    for line in validation.lines() {
                        println!("  {}", line);
                    }
                }
            }
            "repaired_with_warnings" => {
                output::print_warning(&format!(
                    "⚠ Environment '{}' repaired with warnings",
                    env_name
                ));

                if let Some(warning) = result.get("warning").and_then(|v| v.as_str()) {
                    output::print_warning(&format!("Warning: {}", warning));
                }
            }
            _ => {
                output::print_error(&format!("Unknown repair status: {}", status));
            }
        }
    }

    Ok(())
}
