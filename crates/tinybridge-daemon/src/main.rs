#![allow(unused_imports, unused_variables, unexpected_cfgs, dead_code)]

mod anomaly_detector;
mod boot_tiers;
mod clipboard_sync;
mod daemon;
mod daemon_init;
mod dds_network_config;
mod dds_rpc;
mod device_access_control;
mod device_discovery;
mod device_manager;
mod error_propagation;
mod graceful_shutdown;
mod health;
mod integration_tests;
mod ip_monitor;
mod manager;
mod okf_pipeline;
mod okf_updater;
mod otel;
mod otel_export;
mod phase_4_audit_dds_tests;
mod phase_4_device_tests;
mod phase_4_policy_tests;
mod policy_audit_logger;
mod policy_engine;
mod quality_gates;
mod rpc_methods;
mod server;
mod state;
mod structured_logging;
mod vz;

use anyhow::Result;
use clap::Parser;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing_subscriber::EnvFilter;

/// A `MakeWriter` that tees every tracing event to both stderr (as before) and
/// the daemon log file, so `tinybridge logs` has a real source to read.
#[derive(Clone)]
struct TeeWriter {
    file: Arc<Mutex<std::fs::File>>,
}

struct TeeHandle(Arc<Mutex<std::fs::File>>);

impl std::io::Write for TeeHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = std::io::stderr().write_all(buf);
        let mut file = self.0.lock().unwrap();
        file.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let _ = std::io::stderr().flush();
        self.0.lock().unwrap().flush()
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for TeeWriter {
    type Writer = TeeHandle;

    fn make_writer(&'a self) -> Self::Writer {
        TeeHandle(self.file.clone())
    }
}

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

    // Initialize OTel tracing (exports to Jaeger if available, falls back gracefully)
    let _ = otel::init_tracing("tinybridged");

    // Standard tracing subscriber. When the daemon log directory is writable we
    // also persist to `~/Library/Logs/TinyBridge/tinybridged.log` (the file
    // `tinybridge logs` reads); otherwise fall back to stderr only.
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("tinybridge={}", log_level)));

    let log_dir = tinybridge_core::TinyBridgeConfig::logs_dir();
    let _ = std::fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("tinybridged.log");

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(file) => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_ansi(false)
                .with_writer(TeeWriter {
                    file: Arc::new(Mutex::new(file)),
                })
                .init();
            tracing::info!("Logging to {}", log_path.display());
        }
        Err(_) => {
            tracing_subscriber::fmt().with_env_filter(filter).init();
        }
    }

    let socket = args
        .socket
        .unwrap_or_else(tinybridge_core::TinyBridgeConfig::socket_path);

    tracing::info!("TinyBridge daemon starting");

    // Run idempotent migration from legacy paths to Apple conventions (non-fatal errors logged)
    if let Err(e) = tinybridge_core::migration::migrate_legacy_layout() {
        tracing::warn!("Legacy migration warning (non-blocking): {}", e);
    }

    tracing::info!("Socket: {:?}", socket);

    // Initialize daemon state (signal handlers, health checker, shutdown coordinator)
    let _daemon_state = daemon_init::DaemonState::initialize().await?;

    daemon::run(socket).await
}
