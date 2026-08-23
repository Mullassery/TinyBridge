use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::output;

#[derive(clap::Parser)]
pub struct UpdateArgs {
    #[arg(help = "Environment name")]
    pub name: String,

    #[arg(long, help = "CPU cores")]
    pub cpu: Option<u32>,

    #[arg(long, help = "Memory in GB")]
    pub memory: Option<u64>,

    #[arg(long, help = "Disk size in GB")]
    pub disk: Option<u64>,
}

pub async fn execute(args: UpdateArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let params = json!({
        "name": args.name,
        "cpu": args.cpu,
        "memory": args.memory,
        "disk": args.disk,
    });

    client.call("environment.update", params).await?;

    output::print_success(&format!("Environment '{}' resources updated", args.name));
    if let Some(cpu) = args.cpu {
        output::print_success(&format!("  CPU: {} cores", cpu));
    }
    if let Some(mem) = args.memory {
        output::print_success(&format!("  Memory: {} GB", mem));
    }
    if let Some(disk) = args.disk {
        output::print_success(&format!("  Disk: {} GB", disk));
    }

    Ok(())
}
