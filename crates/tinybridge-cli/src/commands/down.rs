use anyhow::Result;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::commands::DownArgs;
use crate::output;

pub async fn execute(args: DownArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let env_name = args.name.as_deref().unwrap_or("default");
    output::print_info(&format!("Stopping environment: {:?}", env_name));

    client.down(args.name.clone(), args.force).await?;

    output::print_success(&format!("Environment {} stopped", env_name));

    Ok(())
}
