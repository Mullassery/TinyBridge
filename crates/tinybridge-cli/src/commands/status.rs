use anyhow::Result;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::commands::StatusArgs;
use crate::output;

pub async fn execute(args: StatusArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let response = client.status(args.name.clone()).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        if response.environments.is_empty() {
            output::print_info("No environments found");
        } else {
            output::print_header("Environments:");
            for env in response.environments {
                let ip = env.ip_address.as_deref().unwrap_or("(no IP)");
                println!("  {} | {} | IP: {}", env.name, env.status, ip);
            }
        }
    }

    Ok(())
}
