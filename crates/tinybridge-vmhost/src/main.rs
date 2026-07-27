use anyhow::{anyhow, Result};
use tinybridge_core::{config::TinyBridgeConfig, Resources};
use tinybridge_vz::VmConfig;

mod socket_server;
mod vm_controller;

#[derive(Debug, Clone)]
struct VmhostArgs {
    env_id: String,
    kernel_path: String,
    disk_path: String,
    cpu_count: u32,
    memory_bytes: u64,
    disk_bytes: u64,
    display_width: u32,
    display_height: u32,
}

impl VmhostArgs {
    fn from_env() -> Result<Self> {
        let env_id = std::env::var("TINYBRIDGE_ENV_ID")
            .map_err(|_| anyhow!("Missing TINYBRIDGE_ENV_ID"))?;
        let kernel_path = std::env::var("TINYBRIDGE_KERNEL_PATH")
            .map_err(|_| anyhow!("Missing TINYBRIDGE_KERNEL_PATH"))?;
        let disk_path = std::env::var("TINYBRIDGE_DISK_PATH")
            .map_err(|_| anyhow!("Missing TINYBRIDGE_DISK_PATH"))?;
        let cpu_count: u32 = std::env::var("TINYBRIDGE_CPU_COUNT")
            .unwrap_or_else(|_| "2".to_string())
            .parse()?;
        let memory_bytes: u64 = std::env::var("TINYBRIDGE_MEMORY_BYTES")
            .unwrap_or_else(|_| (4u64 * 1024 * 1024 * 1024).to_string())
            .parse()?;
        let disk_bytes: u64 = std::env::var("TINYBRIDGE_DISK_BYTES")
            .unwrap_or_else(|_| (20u64 * 1024 * 1024 * 1024).to_string())
            .parse()?;
        let display_width: u32 = std::env::var("TINYBRIDGE_DISPLAY_WIDTH")
            .unwrap_or_else(|_| "1920".to_string())
            .parse()?;
        let display_height: u32 = std::env::var("TINYBRIDGE_DISPLAY_HEIGHT")
            .unwrap_or_else(|_| "1080".to_string())
            .parse()?;

        Ok(VmhostArgs {
            env_id,
            kernel_path,
            disk_path,
            cpu_count,
            memory_bytes,
            disk_bytes,
            display_width,
            display_height,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let args = VmhostArgs::from_env()?;

    // Create VM configuration
    let vm_config = VmConfig::new(args.kernel_path.clone(), args.disk_path.clone(), Resources {
        cpu: args.cpu_count,
        memory_bytes: args.memory_bytes,
        disk_bytes: args.disk_bytes,
    })
    .with_cmdline("root=/dev/vda1 rw console=hvc0 quiet systemd.unified_cgroup_hierarchy=1".to_string())
    .with_display(args.display_width, args.display_height);

    // Initialize VM controller
    let vm_controller = vm_controller::VmController::new(args.env_id.clone(), vm_config)?;

    // Create socket server for JSON-RPC commands
    let socket_path = TinyBridgeConfig::default().vmhost_socket_path(&args.env_id);
    let mut socket_server = socket_server::SocketServer::new(socket_path, vm_controller)?;

    tracing::info!(env_id = %args.env_id, "tinybridge-vmhost started");

    // Run socket server (blocks until shutdown)
    socket_server.run().await?;

    Ok(())
}
