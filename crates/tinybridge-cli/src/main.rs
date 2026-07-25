mod client;
mod commands;
mod output;
mod terminal;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "tinybridge",
    version,
    about = "macOS Linux development substrate",
    long_about = "TinyBridge: Open-source Linux development environment for macOS with intelligent tier routing"
)]
struct Cli {
    #[arg(long, env = "TINYBRIDGE_SOCKET", global = true)]
    socket: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch a new environment (new primary command for 2.0)
    Launch(commands::LaunchArgs),

    /// Start an environment (legacy, use 'launch' instead)
    Up(commands::UpArgs),

    /// Stop an environment
    Down(commands::DownArgs),

    /// Restart an environment
    Restart(commands::RestartArgs),

    /// Destroy an environment
    Destroy(commands::DestroyArgs),

    /// Show environment status
    Status(commands::StatusArgs),

    /// List all environments
    List(commands::ListArgs),

    /// Open shell in environment
    Shell(commands::ShellArgs),

    /// Show logs
    Logs(commands::LogsArgs),

    /// Run system diagnostics
    Doctor(commands::DoctorArgs),

    /// List available templates
    Templates(commands::TemplatesArgs),

    /// List available images
    Images(commands::ImagesArgs),

    /// Manage DDS networking
    Dds(commands::dds::DdsArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(format!("tinybridge={}", log_level))),
        )
        .init();

    let socket = cli.socket.clone();

    match cli.command {
        Commands::Launch(args) => commands::launch::execute(args, socket).await,
        Commands::Up(args) => commands::up::execute(args, socket).await,
        Commands::Down(args) => commands::down::execute(args, socket).await,
        Commands::Restart(args) => commands::restart::execute(args, socket).await,
        Commands::Destroy(args) => commands::destroy::execute(args, socket).await,
        Commands::Status(args) => commands::status::execute(args, socket).await,
        Commands::List(args) => commands::list::execute(args, socket).await,
        Commands::Shell(args) => commands::shell::execute(args, socket).await,
        Commands::Logs(args) => commands::logs::execute(args, socket).await,
        Commands::Doctor(args) => commands::doctor::execute(args, socket).await,
        Commands::Templates(args) => commands::templates::execute(args, socket).await,
        Commands::Images(args) => commands::images::execute(args, socket).await,
        Commands::Dds(args) => {
            let mut client = client::DaemonClient::new(socket)?;
            commands::dds::execute(args, &mut client).await
        }
    }
}
