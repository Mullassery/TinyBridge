use anyhow::Result;
use std::path::PathBuf;
use serde_json::json;

use crate::client::DaemonClient;
use crate::output;

#[derive(clap::Parser)]
pub struct SuspendArgs {
    #[arg(help = "Environment name")]
    pub name: Option<String>,
}

pub async fn execute(args: SuspendArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let env_name = args.name.as_deref().unwrap_or("default");

    let params = json!({"name": args.name});
    client.call("environment.suspend", params).await?;

    output::print_success(&format!("Environment '{}' suspended", env_name));
    output::print_info("Run 'tinybridge resume' to wake it up");

    Ok(())
}
