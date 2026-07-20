mod daemon;
mod manager;
mod server;
mod state;
mod vz;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "tinybridged",
    version,
    about = "TinyBridge daemon",
    long_about = "TinyBridge environment manager daemon"
)]
struct Args {
    #[arg(long, env = "TINYBRIDGE_SOCKET", help = "Unix socket path")]
    socket: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count, help = "Verbosity level")]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = match args.verbose {
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

    let socket = args
        .socket
        .unwrap_or_else(tinybridge_core::TinyBridgeConfig::socket_path);

    tracing::info!("TinyBridge daemon starting");
    tracing::info!("Socket: {:?}", socket);

    daemon::run(socket).await
}
