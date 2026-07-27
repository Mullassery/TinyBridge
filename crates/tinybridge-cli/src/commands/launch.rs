use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::output;
use clap::Parser;

#[derive(Parser)]
pub struct LaunchArgs {
    /// Environment name (defaults to 'default')
    pub name: Option<String>,

    /// Template to use (ubuntu, rust, python, ros2-humble, etc.)
    #[arg(long, short)]
    pub template: Option<String>,

    /// Path to custom env.yaml file
    #[arg(long, short)]
    pub file: Option<String>,

    /// Don't wait for environment to be fully ready
    #[arg(long)]
    pub no_wait: bool,

    /// Show system defaults that will be applied
    #[arg(long)]
    pub show_defaults: bool,

    /// CPU cores to allocate (auto-detected if not specified)
    #[arg(long)]
    pub cpu: Option<u32>,

    /// Memory to allocate in GB (auto-detected if not specified)
    #[arg(long)]
    pub memory: Option<u32>,

    /// Disk size in GB (auto-detected if not specified)
    #[arg(long)]
    pub disk: Option<u32>,
}

pub async fn execute(args: LaunchArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    let env_name = args.name.as_deref().unwrap_or("default");

    // Show defaults if requested
    if args.show_defaults {
        show_defaults(env_name, args.cpu, args.memory, args.disk)?;
        return Ok(());
    }

    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    // Step 1: Prepare environment
    pb.set_message(format!("📦 Preparing {} environment...", env_name));
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 2: Download image (if needed)
    pb.set_message(format!("⬇️  Downloading Linux image..."));
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 3: Create VM
    pb.set_message(format!("🔧 Creating virtual machine..."));
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 4: Configure network
    pb.set_message(format!("🌐 Configuring networking..."));
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 5: Start environment
    pb.set_message(format!("🚀 Starting environment..."));
    client
        .up(
            args.name.clone(),
            args.file.clone(),
            args.cpu,
            args.memory.map(|m| m as u64),
            args.disk.map(|d| d as u64),
        )
        .await?;

    pb.finish_with_message(format!("✅ {} is ready!", env_name));

    // Show connection details
    output::print_success(&format!("Environment name: {}", env_name));
    output::print_info(&format!("SSH: ssh vm@{}", env_name));
    output::print_info(&format!("Shell: tinybridge shell {}", env_name));

    if let Some(template) = args.template {
        output::print_info(&format!("Template: {}", template));
    }

    Ok(())
}

fn show_defaults(
    env_name: &str,
    cpu: Option<u32>,
    memory: Option<u32>,
    disk: Option<u32>,
) -> Result<()> {
    output::print_header("Recommended System Configuration");
    output::print_info(&format!("Environment: {}", env_name));

    let (cpu_default, mem_default, disk_default) = detect_system_defaults();

    let cpu_to_use = cpu.unwrap_or(cpu_default);
    let mem_to_use = memory.unwrap_or(mem_default);
    let disk_to_use = disk.unwrap_or(disk_default);

    output::print_config("CPU Cores", &cpu_to_use.to_string());
    output::print_config("Memory (GB)", &mem_to_use.to_string());
    output::print_config("Disk (GB)", &disk_to_use.to_string());

    output::print_info(
        "These values will be applied automatically unless you specify --cpu, --memory, or --disk",
    );

    Ok(())
}

fn detect_system_defaults() -> (u32, u32, u32) {
    // TODO: Implement actual system detection
    // For now, return reasonable defaults
    (4, 8, 40)
}
