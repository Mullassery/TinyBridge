use anyhow::Result;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::commands::UpArgs;
use crate::output;

pub async fn execute(args: UpArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let env_name = args.name.as_deref().unwrap_or("default");
    output::print_info(&format!("Starting environment: {:?}", env_name));

    client.up(args.name.clone(), args.file.clone()).await?;

    output::print_success(&format!("Environment {} is ready", env_name));

    Ok(())
}
