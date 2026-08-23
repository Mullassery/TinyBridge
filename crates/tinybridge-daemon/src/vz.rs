use anyhow::{anyhow, Result};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tinybridge_core::{config::TinyBridgeConfig, Resources};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::process::Child;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

struct VmhostProcess {
    #[allow(dead_code)] // kept for diagnostics/logging call sites that may want it later
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
        let socket_dir = socket_path.parent().ok_or_else(|| {
            anyhow!(
                "vmhost socket path {} has no parent directory",
                socket_path.display()
            )
        })?;
        std::fs::create_dir_all(socket_dir)?;

        // Spawn tinybridge-vmhost child process. This process is the one that actually owns
        // the real tinybridge_vz::VirtualMachine (see crates/tinybridge-vmhost) - it must be
        // codesigned with the com.apple.security.virtualization entitlement
        // (crates/tinybridge-vmhost/tinybridge-vmhost.entitlements) or every VM lifecycle
        // call below will fail with a real, honestly-surfaced FFI error.
        let child = tokio::process::Command::new("tinybridge-vmhost")
            .env("TINYBRIDGE_ENV_ID", &env_id)
            .env("TINYBRIDGE_KERNEL_PATH", &kernel_path)
            .env("TINYBRIDGE_DISK_PATH", &disk_path)
            .env("TINYBRIDGE_CPU_COUNT", resources.cpu.to_string())
            .env(
                "TINYBRIDGE_MEMORY_BYTES",
                resources.memory_bytes.to_string(),
            )
            .env("TINYBRIDGE_DISK_BYTES", resources.disk_bytes.to_string())
            .env("TINYBRIDGE_DISPLAY_WIDTH", "1920")
            .env("TINYBRIDGE_DISPLAY_HEIGHT", "1080")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("failed to spawn tinybridge-vmhost: {e}"))?;

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

    /// Wait for the vmhost's control socket to appear, then issue a real `vmhost.start`
    /// RPC. Any error returned by the vmhost (including a real Virtualization.framework
    /// failure surfaced through tinybridge-vmhost's VmController) propagates here instead
    /// of being silently ignored.
    pub async fn start_vm(&self, id: Uuid) -> Result<()> {
        let vmhost = self
            .vmhosts
            .get(&id)
            .ok_or_else(|| anyhow!("VM host process not found"))?;

        self.wait_for_socket(&vmhost.socket_path).await?;

        let response = self
            .send_rpc(&vmhost.socket_path, "vmhost.start", json!({}))
            .await?;
        Self::check_rpc_error(&response)
    }

    pub async fn stop_vm(&self, id: Uuid) -> Result<()> {
        let vmhost = self
            .vmhosts
            .get(&id)
            .ok_or_else(|| anyhow!("VM host process not found"))?;

        let response = self
            .send_rpc(&vmhost.socket_path, "vmhost.stop", json!({}))
            .await?;
        Self::check_rpc_error(&response)
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

    /// Real, live VM status (state/ip/resource usage) as reported by tinybridge-vmhost,
    /// which in turn reads it straight from Virtualization.framework. Callers (e.g.
    /// `manager.rs::up()`) should use this instead of fabricating a status/IP.
    pub async fn status_vm(&self, id: Uuid) -> Result<serde_json::Value> {
        let vmhost = self
            .vmhosts
            .get(&id)
            .ok_or_else(|| anyhow!("VM host process not found"))?;

        let response = self
            .send_rpc(&vmhost.socket_path, "vmhost.status", json!({}))
            .await?;
        Self::check_rpc_error(&response)?;
        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow!("vmhost.status response missing 'result' field"))
    }

    async fn wait_for_socket(&self, socket_path: &PathBuf) -> Result<()> {
        const MAX_ATTEMPTS: u32 = 50; // 50 * 100ms = 5s
        for attempt in 0..MAX_ATTEMPTS {
            if UnixStream::connect(socket_path).await.is_ok() {
                return Ok(());
            }
            if attempt + 1 == MAX_ATTEMPTS {
                return Err(anyhow!(
                    "tinybridge-vmhost did not open its control socket at {} within {}ms",
                    socket_path.display(),
                    MAX_ATTEMPTS * 100
                ));
            }
            sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }

    fn check_rpc_error(response: &serde_json::Value) -> Result<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown vmhost error");
            return Err(anyhow!("vmhost RPC error: {message}"));
        }
        Ok(())
    }

    async fn send_rpc(
        &self,
        socket_path: &PathBuf,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
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
