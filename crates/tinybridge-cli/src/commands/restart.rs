use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::output;
use clap::Parser;

#[derive(Parser)]
pub struct RestartArgs {
    /// Environment name
    pub name: Option<String>,

    /// Force restart (skip graceful shutdown)
    #[arg(long, short = 'f')]
    pub force: bool,
}

pub async fn execute(args: RestartArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;
    let env_name = args.name.as_deref().unwrap_or("default");

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    pb.set_message(format!("Stopping {}...", env_name));
    client.down(args.name.clone(), args.force).await?;

    pb.set_message(format!("Starting {}...", env_name));
    client.up(args.name.clone(), None, None, None, None, None, None).await?;

    pb.finish_with_message(format!("✅ {} restarted", env_name));

    output::print_success(&format!("SSH: ssh vm@{}", env_name));

    Ok(())
}
