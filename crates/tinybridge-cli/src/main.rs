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
    /// Start an environment
    Up(commands::UpArgs),

    /// Stop an environment
    Down(commands::DownArgs),

    /// Show environment status
    Status(commands::StatusArgs),

    /// List all environments
    List(commands::ListArgs),

    /// Open shell in environment
    Shell(commands::ShellArgs),
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
        Commands::Up(args) => commands::up::execute(args, socket).await,
        Commands::Down(args) => commands::down::execute(args, socket).await,
        Commands::Status(args) => commands::status::execute(args, socket).await,
        Commands::List(args) => commands::list::execute(args, socket).await,
        Commands::Shell(args) => commands::shell::execute(args, socket).await,
    }
}
