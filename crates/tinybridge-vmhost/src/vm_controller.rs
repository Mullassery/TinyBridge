use anyhow::{anyhow, Result};
use serde_json::json;
use std::sync::Arc;
use tinybridge_vz::{VirtualMachine, VmConfig};
use tokio::sync::Mutex;

/// Holds either a real, FFI-backed `VirtualMachine` or the reason one could not be created.
///
/// Previously this controller never touched `tinybridge_vz::VirtualMachine` at all and
/// `status()` unconditionally returned `"status": "running"`. That is fixed here: VM
/// creation is attempted for real in `VmController::new`, and every lifecycle call below
/// (`start`/`stop`/`force_stop`/`status`/`show_window`/`hide_window`) drives the actual
/// `tinybridge_vz` FFI layer. If Virtualization.framework is unavailable, the process lacks
/// the `com.apple.security.virtualization` entitlement, or the config is invalid, that real
/// failure is captured and surfaced through `status()` instead of being papered over.
enum VmSlot {
    Created(VirtualMachine),
    Unavailable(String),
}

#[derive(Clone)]
pub struct VmController {
    env_id: Arc<str>,
    config: Arc<VmConfig>,
    vm: Arc<Mutex<VmSlot>>,
}

impl VmController {
    pub fn new(env_id: String, config: VmConfig) -> Result<Self> {
        let slot = match VirtualMachine::new(env_id.clone(), config.clone()) {
            Ok(vm) => VmSlot::Created(vm),
            Err(e) => {
                tracing::error!(
                    env_id = %env_id,
                    error = %e,
                    "Failed to create real VirtualMachine via tinybridge-vz FFI; VM will report as unavailable"
                );
                VmSlot::Unavailable(e.to_string())
            }
        };

        Ok(VmController {
            env_id: Arc::from(env_id),
            config: Arc::new(config),
            vm: Arc::new(Mutex::new(slot)),
        })
    }

    pub fn env_id(&self) -> &str {
        &self.env_id
    }

    /// Start the real VM. This dispatches an async `VZVirtualMachine.start()` on the Swift
    /// side; a successful return here means the request was accepted, not that the guest
    /// has finished booting - poll `status()` for the real state transition.
    pub async fn start(&self) -> Result<()> {
        let guard = self.vm.lock().await;
        match &*guard {
            VmSlot::Created(vm) => vm.start().map_err(|e| anyhow!("VM start failed: {e}")),
            VmSlot::Unavailable(reason) => Err(anyhow!("VM unavailable, cannot start: {reason}")),
        }
    }

    pub async fn stop(&self) -> Result<()> {
        let guard = self.vm.lock().await;
        match &*guard {
            VmSlot::Created(vm) => vm.stop().map_err(|e| anyhow!("VM stop failed: {e}")),
            VmSlot::Unavailable(reason) => Err(anyhow!("VM unavailable, cannot stop: {reason}")),
        }
    }

    pub async fn force_stop(&self) -> Result<()> {
        let guard = self.vm.lock().await;
        match &*guard {
            VmSlot::Created(vm) => vm
                .force_stop()
                .map_err(|e| anyhow!("VM force_stop failed: {e}")),
            VmSlot::Unavailable(reason) => {
                Err(anyhow!("VM unavailable, cannot force_stop: {reason}"))
            }
        }
    }

    pub async fn show_window(&self) -> Result<()> {
        let guard = self.vm.lock().await;
        match &*guard {
            VmSlot::Created(vm) => vm
                .show_window()
                .map_err(|e| anyhow!("show_window failed: {e}")),
            VmSlot::Unavailable(reason) => Err(anyhow!("VM unavailable: {reason}")),
        }
    }

    pub async fn hide_window(&self) -> Result<()> {
        let guard = self.vm.lock().await;
        match &*guard {
            VmSlot::Created(vm) => vm
                .hide_window()
                .map_err(|e| anyhow!("hide_window failed: {e}")),
            VmSlot::Unavailable(reason) => Err(anyhow!("VM unavailable: {reason}")),
        }
    }

    /// Real, live status read from Virtualization.framework through the FFI boundary. Never
    /// hardcoded - if the VM could not be created, or the FFI status query fails, that is
    /// reported honestly rather than claiming "running".
    pub async fn status(&self) -> Result<String> {
        let guard = self.vm.lock().await;
        let status_json = match &*guard {
            VmSlot::Created(vm) => match vm.status() {
                Ok(status) => json!({
                    "env_id": self.env_id.as_ref(),
                    "state": format!("{:?}", status.state),
                    "cpu_usage_pct": status.cpu_usage_pct,
                    "memory_used_bytes": status.memory_used_bytes,
                    "memory_total_bytes": status.memory_total_bytes,
                    "ip_address": status.ip_address,
                    "config": {
                        "cpu": self.config.cpu_count,
                        "memory_gb": self.config.memory_bytes / (1024 * 1024 * 1024),
                    }
                }),
                Err(e) => json!({
                    "env_id": self.env_id.as_ref(),
                    "state": "unknown",
                    "error": e.to_string(),
                }),
            },
            VmSlot::Unavailable(reason) => json!({
                "env_id": self.env_id.as_ref(),
                "state": "unavailable",
                "error": reason,
            }),
        };

        Ok(status_json.to_string())
    }
}
