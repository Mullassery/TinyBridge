pub mod dds;
pub mod destroy;
pub mod doctor;
pub mod down;
pub mod gui;
pub mod headless;
pub mod images;
pub mod launch;
pub mod list;
pub mod logs;
pub mod repair;
pub mod restart;
pub mod resume;
pub mod shell;
pub mod snapshot;
pub mod ssh;
pub mod status;
pub mod suspend;
pub mod shutdown;
pub mod templates;
pub mod update;
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

    #[arg(long, help = "Launch with GUI window")]
    pub gui: bool,

    #[arg(long, help = "CPU cores (default: 4)")]
    pub cpu: Option<u32>,

    #[arg(long, help = "Memory in GB (default: 8)")]
    pub memory: Option<u64>,

    #[arg(long, help = "Disk size in GB (default: 50)")]
    pub disk: Option<u64>,

    #[arg(long, help = "Enable GPU acceleration (Metal on macOS, default: true)")]
    pub gpu: Option<bool>,

    #[arg(long, help = "GPU memory in GB (0 = auto allocation)")]
    pub gpu_memory: Option<u32>,
}

#[derive(Parser)]
pub struct GuiArgs {
    #[arg(help = "Environment name")]
    pub name: Option<String>,
}

#[derive(Parser)]
pub struct HeadlessArgs {
    #[arg(help = "Environment name")]
    pub name: Option<String>,
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
pub use repair::RepairArgs;
pub use restart::RestartArgs;
pub use templates::TemplatesArgs;
