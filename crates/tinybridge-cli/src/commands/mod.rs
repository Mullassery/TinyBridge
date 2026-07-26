pub mod dds;
pub mod destroy;
pub mod doctor;
pub mod down;
pub mod images;
pub mod launch;
pub mod list;
pub mod logs;
pub mod restart;
pub mod shell;
pub mod status;
pub mod templates;
pub mod up;

use clap::Parser;

#[derive(Parser)]
pub struct UpArgs {
    #[arg(help = "Environment name")]
    pub name: Option<String>,

    #[arg(long, short, help = "Path to env.yaml file")]
    pub file: Option<String>,

    #[arg(long, help = "Wait for environment to be ready")]
    pub wait: bool,
}

#[derive(Parser)]
pub struct DownArgs {
    #[arg(help = "Environment name")]
    pub name: Option<String>,

    #[arg(long, short, help = "Force stop")]
    pub force: bool,
}

#[derive(Parser)]
pub struct StatusArgs {
    #[arg(help = "Environment name (all if not specified)")]
    pub name: Option<String>,

    #[arg(long, help = "Output as JSON")]
    pub json: bool,
}

#[derive(Parser)]
pub struct ListArgs {
    #[arg(long, help = "Output as JSON")]
    pub json: bool,
}

#[derive(Parser)]
pub struct ShellArgs {
    #[arg(help = "Environment name")]
    pub name: Option<String>,

    #[arg(short, long, help = "Command to execute")]
    pub command: Option<String>,
}

// Re-export new command args for main.rs
pub use destroy::DestroyArgs;
pub use doctor::DoctorArgs;
pub use images::ImagesArgs;
pub use launch::LaunchArgs;
pub use logs::LogsArgs;
pub use restart::RestartArgs;
pub use templates::TemplatesArgs;
