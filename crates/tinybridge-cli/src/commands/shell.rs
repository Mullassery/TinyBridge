use anyhow::Result;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::commands::ShellArgs;
use crate::output;

pub async fn execute(args: ShellArgs, socket: Option<PathBuf>) -> Result<()> {
    let _client = DaemonClient::new(socket)?;

    output::print_info(&format!(
        "Opening shell in environment: {:?}",
        args.name.as_deref().unwrap_or("default")
    ));

    if let Some(cmd) = args.command {
        output::print_info(&format!("Executing command: {}", cmd));
    }

    output::print_info("Shell functionality will be enabled in Week 5");

    Ok(())
}
