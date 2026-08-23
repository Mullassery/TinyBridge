use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::output;

#[derive(clap::Parser)]
pub struct ShutdownArgs {
    #[arg(help = "Environment name")]
    pub name: Option<String>,

    #[arg(long, help = "Force shutdown without waiting for graceful shutdown")]
    pub force: bool,

    #[arg(long, help = "Wait timeout in seconds")]
    pub timeout: Option<u64>,
}

pub async fn execute(args: ShutdownArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let env_name = args.name.as_deref().unwrap_or("default");

    let params = json!({
        "name": args.name,
        "force": args.force,
        "timeout": args.timeout.unwrap_or(60),
    });

    client.call("environment.shutdown", params).await?;

    output::print_success(&format!("Environment '{}' is shutting down", env_name));
    output::print_info("Use 'tinybridge up' to start it again (same state preserved)");

    Ok(())
}
