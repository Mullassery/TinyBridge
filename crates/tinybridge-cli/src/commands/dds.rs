use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::client::DaemonClient;
use console::{style, Emoji};
use serde_json::json;

const SUCCESS: Emoji = Emoji("✓", "");
const INFO: Emoji = Emoji("ℹ️", "i");
const WARN: Emoji = Emoji("⚠️", "!");

#[derive(Parser, Debug)]
#[command(name = "dds", about = "Manage DDS networking")]
pub struct DdsArgs {
    #[command(subcommand)]
    pub command: DdsCommand,
}

#[derive(Subcommand, Debug)]
pub enum DdsCommand {
    /// Check DDS status
    Status {
        #[arg(value_name = "ENV")]
        env: String,
        #[arg(long)]
        json: bool,
    },

    /// List DDS status for all environments
    List {
        #[arg(long)]
        json: bool,
    },

    /// Enable DDS for an environment
    Enable {
        #[arg(value_name = "ENV")]
        env: String,
        #[arg(long, default_value = "custom")]
        profile: String,
        #[arg(long)]
        reason: Option<String>,
    },

    /// Disable DDS for an environment
    Disable {
        #[arg(value_name = "ENV")]
        env: String,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        reason: Option<String>,
    },
}

pub async fn execute(args: DdsArgs, client: &mut DaemonClient) -> Result<()> {
    match args.command {
        DdsCommand::Status { env, json } => {
            let response = client.call("dds.status", json!({"env": env})).await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                println!("\n{} {}", style("DDS Status for").bold(), style(&env).cyan());

                if let Some(enabled) = response.get("dds_enabled") {
                    let status = if enabled.as_bool().unwrap_or(false) {
                        style("Enabled").green()
                    } else {
                        style("Disabled").yellow()
                    };
                    println!("  Status: {}", status);
                }

                if let Some(features) = response.get("total_features_enabled") {
                    println!("  Features: {}/15", features);
                }

                if let Some(events) = response.get("audit_events_count") {
                    println!("  Audit Events: {}", events);
                }
                println!();
            }

            Ok(())
        }

        DdsCommand::List { json } => {
            let response = client.call("dds.list", json!({})).await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                println!("\n{}", style("DDS Summary Across All Environments").bold().underlined());

                if let Some(total) = response.get("total_environments") {
                    println!("  Total: {}", total);
                }
                if let Some(enabled) = response.get("dds_enabled") {
                    println!("  Enabled: {} {}", style(enabled).green(), SUCCESS);
                }
                if let Some(disabled) = response.get("dds_disabled") {
                    println!("  Disabled: {} {}", style(disabled).yellow(), INFO);
                }
                println!();
            }

            Ok(())
        }

        DdsCommand::Enable { env, profile, reason } => {
            println!("{} Enabling DDS for {} (profile: {})...", INFO, style(&env).cyan(), style(&profile).yellow());

            let response = client.call(
                "dds.enable",
                json!({"env": env, "profile": profile, "reason": reason})
            ).await?;

            if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                println!("{} DDS enabled successfully", SUCCESS);
            } else if let Some(error) = response.get("error").and_then(|v| v.as_str()) {
                println!("{} Failed: {}", WARN, error);
            }

            println!();
            Ok(())
        }

        DdsCommand::Disable { env, force, reason } => {
            println!("{} Disabling DDS for {}{}...", INFO, style(&env).cyan(), if force { " (force)" } else { "" });

            let response = client.call(
                "dds.disable",
                json!({"env": env, "force": force, "reason": reason})
            ).await?;

            if response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                println!("{} DDS disabled successfully", SUCCESS);
            } else if let Some(error) = response.get("error").and_then(|v| v.as_str()) {
                println!("{} Failed: {}", WARN, error);
            }

            println!();
            Ok(())
        }
    }
}
