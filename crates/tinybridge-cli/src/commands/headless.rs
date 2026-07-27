use anyhow::Result;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::commands::HeadlessArgs;
use crate::output;

pub async fn execute(args: HeadlessArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let env_name = args.name.as_deref().unwrap_or("default");

    output::print_info(&format!("Detaching display from environment: {}", env_name));

    client.hide_window(Some(env_name.to_string())).await?;

    output::print_success(&format!("✓ Display detached from {}. VM continues running headless.", env_name));

    Ok(())
}
