use anyhow::Result;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::Arc;
use tinybridge_dds::DdsManager;
use tokio::sync::Mutex;

use crate::manager::EnvironmentManager;
use crate::server;

pub async fn run(socket_path: PathBuf) -> Result<()> {
    // Remove existing socket if it exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    // Create parent directories if needed
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    listener.set_nonblocking(true)?;

    // Harden the daemon's control socket to owner-only access rather than relying on the
    // process umask (see SECURITY.md - VM Control Socket). Any peer able to connect can
    // drive full environment lifecycle operations.
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
    }

    tracing::info!("Listening on {:?}", socket_path);

    let manager = Arc::new(Mutex::new(EnvironmentManager::new()));
    let dds_manager = Arc::new(parking_lot::Mutex::new(DdsManager::new()));
    let listener = tokio::net::UnixListener::from_std(listener)?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let manager = Arc::clone(&manager);
                let dds_manager = Arc::clone(&dds_manager);
                tokio::spawn(async move {
                    if let Err(e) = server::handle_connection(stream, manager, dds_manager).await {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            Err(e) => {
                tracing::error!("Accept error: {}", e);
            }
        }
    }
}
