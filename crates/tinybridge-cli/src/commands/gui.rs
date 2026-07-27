use anyhow::Result;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::commands::GuiArgs;
use crate::output;

pub async fn execute(args: GuiArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let env_name = args.name.as_deref().unwrap_or("default");

    output::print_info(&format!("Attaching display to environment: {}", env_name));

    client.show_window(Some(env_name.to_string())).await?;

    output::print_success(&format!("✓ Display attached to {}", env_name));

    Ok(())
}
