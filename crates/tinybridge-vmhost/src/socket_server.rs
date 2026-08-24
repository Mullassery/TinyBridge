use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use tinybridge_core::pid_lock::PidLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::vm_controller::VmController;

pub struct SocketServer {
    socket_path: PathBuf,
    vm_controller: VmController,
    // Held for SocketServer's lifetime; Drop removes the PID lock file on
    // clean shutdown. Never read after construction, so field access
    // itself isn't expected -- its existence is the point.
    _pid_lock: PidLock,
}

impl SocketServer {
    pub fn new(socket_path: PathBuf, vm_controller: VmController) -> Result<Self> {
        // Only removes an existing socket once confirmed orphaned (left by
        // a process that's no longer running, e.g. after a force-kill),
        // not a still-live instance actively using it. See pid_lock.rs.
        let pid_lock = PidLock::acquire(&socket_path)?;

        Ok(SocketServer {
            socket_path,
            vm_controller,
            _pid_lock: pid_lock,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let listener = UnixListener::bind(&self.socket_path)?;

        // Harden the control socket to owner-only access. The default create-mode is
        // governed by the process umask, which is not guaranteed to be restrictive (e.g.
        // 022 leaves the socket group/world-readable+writable) - a peer able to connect to
        // this socket can drive VM start/stop/force_stop, so it must not be reachable by
        // other local users regardless of umask.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.socket_path, std::fs::Permissions::from_mode(0o600))?;
        }

        tracing::info!(socket = %self.socket_path.display(), "Listening for JSON-RPC connections");

        loop {
            let (stream, _) = listener.accept().await?;
            let vm_id = self.vm_controller.env_id().to_string();
            let controller = self.vm_controller.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, controller).await {
                    tracing::error!(env_id = %vm_id, error = %e, "Connection error");
                }
            });
        }
    }

    async fn handle_connection(stream: UnixStream, controller: VmController) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        while let Some(line) = lines.next_line().await? {
            if line.is_empty() {
                continue;
            }

            let request: serde_json::Value = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(_) => {
                    let response = json!({"error": "Invalid JSON", "jsonrpc": "2.0"});
                    writer.write_all(response.to_string().as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    continue;
                }
            };

            let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
            let id = request.get("id").cloned();

            let response = match method {
                "vmhost.status" => match controller.status().await {
                    Ok(status) => json!({
                        "jsonrpc": "2.0",
                        "result": serde_json::from_str::<serde_json::Value>(&status).unwrap_or_default(),
                        "id": id
                    }),
                    Err(e) => json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32000, "message": e.to_string()},
                        "id": id
                    }),
                },
                "vmhost.start" => match controller.start().await {
                    Ok(_) => json!({
                        "jsonrpc": "2.0",
                        "result": "ok",
                        "id": id
                    }),
                    Err(e) => json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32000, "message": e.to_string()},
                        "id": id
                    }),
                },
                "vmhost.stop" => match controller.stop().await {
                    Ok(_) => json!({
                        "jsonrpc": "2.0",
                        "result": "ok",
                        "id": id
                    }),
                    Err(e) => json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32000, "message": e.to_string()},
                        "id": id
                    }),
                },
                "vmhost.force_stop" => match controller.force_stop().await {
                    Ok(_) => json!({
                        "jsonrpc": "2.0",
                        "result": "ok",
                        "id": id
                    }),
                    Err(e) => json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32000, "message": e.to_string()},
                        "id": id
                    }),
                },
                "vmhost.show_window" => match controller.show_window().await {
                    Ok(_) => json!({
                        "jsonrpc": "2.0",
                        "result": "ok",
                        "id": id
                    }),
                    Err(e) => json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32000, "message": e.to_string()},
                        "id": id
                    }),
                },
                "vmhost.hide_window" => match controller.hide_window().await {
                    Ok(_) => json!({
                        "jsonrpc": "2.0",
                        "result": "ok",
                        "id": id
                    }),
                    Err(e) => json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32000, "message": e.to_string()},
                        "id": id
                    }),
                },
                _ => json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32601, "message": "Method not found"},
                    "id": id
                }),
            };

            writer.write_all(response.to_string().as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }

        Ok(())
    }
}
