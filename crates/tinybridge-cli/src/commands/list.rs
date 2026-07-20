use anyhow::Result;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::commands::ListArgs;
use crate::output;

pub async fn execute(args: ListArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let response = client.list().await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        if response.environments.is_empty() {
            output::print_info("No environments. Create one with: tinybridge up");
        } else {
            output::print_header("Environments");
            println!(
                "{:<20} {:<20} {:<15} {:<15}",
                "Name", "Status", "Uptime", "IP Address"
            );
            println!("{}", "─".repeat(70));

            for env in response.environments {
                let status_str = output::status_badge(&env.status);
                let ip = env.ip_address.as_deref().unwrap_or("—");
                let uptime = env
                    .uptime_secs
                    .map(|s| {
                        if s < 60 {
                            format!("{}s", s)
                        } else if s < 3600 {
                            format!("{}m", s / 60)
                        } else {
                            format!("{}h", s / 3600)
                        }
                    })
                    .unwrap_or_else(|| "—".to_string());

                println!(
                    "{:<20} {:<20} {:<15} {:<15}",
                    env.name, status_str, uptime, ip
                );
            }
        }
    }

    Ok(())
}
