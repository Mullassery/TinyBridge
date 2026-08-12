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
        let env_id =
            std::env::var("TINYBRIDGE_ENV_ID").map_err(|_| anyhow!("Missing TINYBRIDGE_ENV_ID"))?;
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

/// Virtualization.framework callbacks (`VZVirtualMachine.start()`/`stop()` completion,
/// `VZVirtualMachineDelegate` methods) are dispatched onto the process's main GCD queue by
/// the Swift bridge (see swift/Sources/TinyBridgeVZBridge/TinyBridgeVZ.swift). A plain
/// `#[tokio::main]` process never drains that queue - Tokio's scheduler owns the thread
/// instead - so every VM lifecycle call would silently hang forever with no error, ever.
/// This was verified directly: `tb_vm_start` returns immediately (dispatch accepted) but
/// the guest never transitions out of "Stopped" state until something pumps the main queue.
///
/// The fix: run the actual async server on a background OS thread with its own Tokio
/// runtime, and dedicate the real process main thread to draining the macOS main dispatch
/// queue via `dispatch_main()` for the lifetime of the process.
#[cfg(target_os = "macos")]
mod macos_main_queue {
    extern "C" {
        pub fn dispatch_main() -> !;
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let args = VmhostArgs::from_env()?;

    let worker = std::thread::Builder::new()
        .name("tinybridge-vmhost-async".to_string())
        .spawn(move || {
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("tinybridge-vmhost: failed to start Tokio runtime: {e}");
                    std::process::exit(1);
                }
            };

            let result = runtime.block_on(async_main(args));
            match result {
                Ok(()) => std::process::exit(0),
                Err(e) => {
                    tracing::error!(error = %e, "tinybridge-vmhost exited with error");
                    std::process::exit(1);
                }
            }
        })
        .map_err(|e| anyhow!("failed to spawn async runtime thread: {e}"))?;
    let _ = &worker; // used below only in the non-macOS branch

    #[cfg(target_os = "macos")]
    unsafe {
        // Never returns; the worker thread above calls std::process::exit() when it's done.
        macos_main_queue::dispatch_main();
    }

    // Off-macOS this binary has no real VZ backend to drive anyway (tinybridge-vz only links
    // against Virtualization.framework on macOS), so just wait for the worker.
    #[cfg(not(target_os = "macos"))]
    {
        let _ = worker.join();
        Ok(())
    }
}

async fn async_main(args: VmhostArgs) -> Result<()> {
    // Create VM configuration
    let vm_config = VmConfig::new(
        args.kernel_path.clone(),
        args.disk_path.clone(),
        Resources {
            cpu: args.cpu_count,
            memory_bytes: args.memory_bytes,
            disk_bytes: args.disk_bytes,
            gpu: None, // Use defaults
        },
    )
    .with_cmdline(
        "root=/dev/vda1 rw console=hvc0 quiet systemd.unified_cgroup_hierarchy=1".to_string(),
    )
    .with_display(args.display_width, args.display_height);

    // Initialize VM controller. This now attempts to create a real tinybridge_vz::VirtualMachine
    // (see vm_controller.rs) - if Virtualization.framework is unavailable or the process lacks
    // the com.apple.security.virtualization entitlement, that failure is captured and reported
    // via status() rather than silently pretending the VM is running.
    let vm_controller = vm_controller::VmController::new(args.env_id.clone(), vm_config)?;

    // Create socket server for JSON-RPC commands
    let socket_path = TinyBridgeConfig::default().vmhost_socket_path(&args.env_id);
    let mut socket_server = socket_server::SocketServer::new(socket_path, vm_controller)?;

    tracing::info!(env_id = %args.env_id, "tinybridge-vmhost started");

    // Run socket server (blocks until shutdown)
    socket_server.run().await?;

    Ok(())
}
