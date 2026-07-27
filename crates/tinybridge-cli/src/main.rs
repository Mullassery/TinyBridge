mod client;
mod commands;
mod interactive;
mod keyboard;
mod output;
mod terminal;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "tinybridge",
    version,
    about = "macOS Linux development substrate",
    long_about = "TinyBridge: Open-source Linux development environment for macOS with intelligent tier routing",
    subcommand_required = false
)]
struct Cli {
    #[arg(long, env = "TINYBRIDGE_SOCKET", global = true)]
    socket: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch a new environment (new primary command for 2.0)
    Launch(commands::LaunchArgs),

    /// Start an environment (legacy, use 'launch' instead)
    Up(commands::UpArgs),

    /// Attach display window to environment
    Gui(commands::GuiArgs),

    /// Detach display window from environment
    Headless(commands::HeadlessArgs),

    /// Stop an environment
    Down(commands::DownArgs),

    /// Suspend an environment (pause VM, preserves state)
    Suspend(commands::suspend::SuspendArgs),

    /// Resume a suspended environment
    Resume(commands::resume::ResumeArgs),

    /// Gracefully shutdown an environment
    Shutdown(commands::shutdown::ShutdownArgs),

    /// Restart an environment
    Restart(commands::RestartArgs),

    /// Repair an environment
    Repair(commands::RepairArgs),

    /// Destroy an environment
    Destroy(commands::DestroyArgs),

    /// Show environment status
    Status(commands::StatusArgs),

    /// List all environments
    List(commands::ListArgs),

    /// Open shell in environment
    Shell(commands::ShellArgs),

    /// SSH into environment
    Ssh(commands::ssh::SshArgs),

    /// Show logs
    Logs(commands::LogsArgs),

    /// Manage environment resources
    Update(commands::update::UpdateArgs),

    /// Manage environment snapshots
    Snapshot(commands::snapshot::SnapshotArgs),

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
async fn main() {
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

    let result = match cli.command {
        Some(Commands::Launch(args)) => commands::launch::execute(args, socket).await,
        Some(Commands::Up(args)) => commands::up::execute(args, socket).await,
        Some(Commands::Gui(args)) => commands::gui::execute(args, socket).await,
        Some(Commands::Headless(args)) => commands::headless::execute(args, socket).await,
        Some(Commands::Down(args)) => commands::down::execute(args, socket).await,
        Some(Commands::Suspend(args)) => commands::suspend::execute(args, socket).await,
        Some(Commands::Resume(args)) => commands::resume::execute(args, socket).await,
        Some(Commands::Shutdown(args)) => commands::shutdown::execute(args, socket).await,
        Some(Commands::Restart(args)) => commands::restart::execute(args, socket).await,
        Some(Commands::Repair(args)) => commands::repair::execute(args, socket).await,
        Some(Commands::Destroy(args)) => commands::destroy::execute(args, socket).await,
        Some(Commands::Status(args)) => commands::status::execute(args, socket).await,
        Some(Commands::List(args)) => commands::list::execute(args, socket).await,
        Some(Commands::Shell(args)) => commands::shell::execute(args, socket).await,
        Some(Commands::Ssh(args)) => commands::ssh::execute(args, socket).await,
        Some(Commands::Logs(args)) => commands::logs::execute(args, socket).await,
        Some(Commands::Update(args)) => commands::update::execute(args, socket).await,
        Some(Commands::Snapshot(args)) => commands::snapshot::execute(args, socket).await,
        Some(Commands::Doctor(args)) => commands::doctor::execute(args, socket).await,
        Some(Commands::Templates(args)) => commands::templates::execute(args, socket).await,
        Some(Commands::Images(args)) => commands::images::execute(args, socket).await,
        Some(Commands::Dds(args)) => match client::DaemonClient::new(socket) {
            Ok(mut client) => commands::dds::execute(args, &mut client).await,
            Err(e) => Err(e),
        },
        None => {
            println!("No subcommand specified. Use 'tinybridge --help' for usage information.");
            Ok(())
        }
    };

    match result {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            output::print_error(&format!("{}", e));
            std::process::exit(1);
        }
    }
}
