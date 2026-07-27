use anyhow::{anyhow, Result};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::process::Child;
use tinybridge_core::{config::TinyBridgeConfig, Resources};
use uuid::Uuid;

struct VmhostProcess {
    env_id: String,
    socket_path: PathBuf,
    child: Child,
}

pub struct VmManager {
    vmhosts: HashMap<Uuid, VmhostProcess>,
}

impl VmManager {
    pub fn new() -> Self {
        VmManager {
            vmhosts: HashMap::new(),
        }
    }

    pub async fn create_vm(
        &mut self,
        id: Uuid,
        name: String,
        kernel_path: String,
        disk_path: String,
        resources: Resources,
    ) -> Result<()> {
        let env_id = name.clone();
        let config = TinyBridgeConfig::default();
        let socket_path = config.vmhost_socket_path(&env_id);

        // Ensure vmhost directory exists
        std::fs::create_dir_all(socket_path.parent().unwrap())?;

        // Spawn tinybridge-vmhost child process
        let child = tokio::process::Command::new("tinybridge-vmhost")
            .env("TINYBRIDGE_ENV_ID", &env_id)
            .env("TINYBRIDGE_KERNEL_PATH", &kernel_path)
            .env("TINYBRIDGE_DISK_PATH", &disk_path)
            .env("TINYBRIDGE_CPU_COUNT", resources.cpu.to_string())
            .env("TINYBRIDGE_MEMORY_BYTES", resources.memory_bytes.to_string())
            .env("TINYBRIDGE_DISK_BYTES", resources.disk_bytes.to_string())
            .env("TINYBRIDGE_DISPLAY_WIDTH", "1920")
            .env("TINYBRIDGE_DISPLAY_HEIGHT", "1080")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.vmhosts.insert(
            id,
            VmhostProcess {
                env_id: env_id.clone(),
                socket_path,
                child,
            },
        );

        Ok(())
    }

    pub async fn start_vm(&self, id: Uuid) -> Result<()> {
        let vmhost = self
            .vmhosts
            .get(&id)
            .ok_or_else(|| anyhow!("VM host process not found"))?;

        // Give vmhost a moment to start listening on socket
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Connect and send status check
        let mut stream = UnixStream::connect(&vmhost.socket_path).await?;
        let request = json!({"jsonrpc": "2.0", "method": "vmhost.status", "id": 1});
        stream.write_all(request.to_string().as_bytes()).await?;
        stream.write_all(b"\n").await?;

        Ok(())
    }

    pub async fn stop_vm(&self, id: Uuid) -> Result<()> {
        let vmhost = self
            .vmhosts
            .get(&id)
            .ok_or_else(|| anyhow!("VM host process not found"))?;

        self.send_rpc(&vmhost.socket_path, "vmhost.stop", json!({}))
            .await?;

        Ok(())
    }

    pub async fn force_stop_vm(&mut self, id: Uuid) -> Result<()> {
        if let Some(mut vmhost) = self.vmhosts.remove(&id) {
            let _ = vmhost.child.kill().await;
        }
        Ok(())
    }

    pub fn destroy_vm(&mut self, id: Uuid) -> Result<()> {
        self.vmhosts.remove(&id);
        Ok(())
    }

    async fn send_rpc(&self, socket_path: &PathBuf, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let mut stream = UnixStream::connect(socket_path).await?;
        let request = json!({"jsonrpc": "2.0", "method": method, "params": params, "id": 1});
        stream.write_all(request.to_string().as_bytes()).await?;
        stream.write_all(b"\n").await?;

        let (reader, _) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();
        if let Some(line) = lines.next_line().await? {
            let response: serde_json::Value = serde_json::from_str(&line)?;
            Ok(response)
        } else {
            Err(anyhow!("No response from vmhost"))
        }
    }
}

impl Default for VmManager {
    fn default() -> Self {
        Self::new()
    }
}
