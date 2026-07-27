use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::commands::UpArgs;
use crate::output;

pub async fn execute(args: UpArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let env_name = args.name.as_deref().unwrap_or("default");

    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Starting environment: {}", env_name));

    client
        .up(
            args.name.clone(),
            args.file.clone(),
            args.cpu,
            args.memory,
            args.disk,
            args.gpu,
            args.gpu_memory,
        )
        .await?;

    pb.finish_with_message(format!("✓ Environment {} is ready", env_name));

    output::print_success(&format!("SSH: ssh vm@{}", env_name));
    output::print_success(&format!("Shell: tinybridge shell {}", env_name));

    Ok(())
}
