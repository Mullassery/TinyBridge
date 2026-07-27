use anyhow::Result;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use serde_json::json;

use crate::vm_controller::VmController;

pub struct SocketServer {
    socket_path: PathBuf,
    vm_controller: VmController,
}

impl SocketServer {
    pub fn new(socket_path: PathBuf, vm_controller: VmController) -> Result<Self> {
        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        Ok(SocketServer {
            socket_path,
            vm_controller,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let listener = UnixListener::bind(&self.socket_path)?;
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
                "vmhost.status" => {
                    match controller.status().await {
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
                    }
                }
                "vmhost.show_window" => {
                    match controller.show_window() {
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
                    }
                }
                "vmhost.hide_window" => {
                    match controller.hide_window() {
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
                    }
                }
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
