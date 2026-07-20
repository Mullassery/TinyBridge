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
            output::print_info("No environments running");
        } else {
            output::print_header("Environments:");
            for env in response.environments {
                let ip = env.ip_address.as_deref().unwrap_or("—");
                let uptime = env
                    .uptime_secs
                    .map(|s| format!("{}s", s))
                    .unwrap_or_else(|| "—".to_string());
                println!("  {} | {} | {} | {}", env.name, env.status, uptime, ip);
            }
        }
    }

    Ok(())
}
