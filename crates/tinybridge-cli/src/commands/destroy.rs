use anyhow::Result;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::output;
use clap::Parser;

#[derive(Parser)]
pub struct DestroyArgs {
    /// Environment name
    pub name: Option<String>,

    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,
}

pub async fn execute(args: DestroyArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;
    let env_name = args.name.as_deref().unwrap_or("default");

    if !args.force {
        output::print_warning(&format!(
            "This will permanently delete the {} environment and all data.",
            env_name
        ));

        print!("Continue? [y/N] ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            output::print_info("Cancelled.");
            return Ok(());
        }
    }

    client.down(args.name.clone(), true).await?;

    output::print_success(&format!("✅ {} destroyed", env_name));

    Ok(())
}
