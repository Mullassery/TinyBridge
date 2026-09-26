use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::instrument;
use uuid::Uuid;

use tinybridge_core::{
    DownResponse, Environment, EnvironmentStatus, EnvironmentSummary, ListResponse, StatusResponse,
    TinyBridgeConfig, UpResponse,
};
use tinybridge_ssh::{KeyType, SshConfigEntry, SshConfigManager, SshKeyManager};

use crate::boot_tiers::BootTierConfig;
use crate::clipboard_sync::ClipboardSyncManager;
use crate::vz::{VmBootAssets, VmManager};

#[derive(Debug, Clone)]
struct ShellSession {
    id: String,
    env_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub struct EnvironmentManager {
    environments: HashMap<String, Environment>,
    shell_sessions: Arc<RwLock<HashMap<String, ShellSession>>>,
    vm_manager: VmManager,
    ssh_key_manager: SshKeyManager,
    ssh_config_manager: SshConfigManager,
    clipboard_sync_manager: ClipboardSyncManager,
    assets_dir: PathBuf,
    boot_tiers: BootTierConfig,
}

impl EnvironmentManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let assets_dir = TinyBridgeConfig::cache_dir().join("assets");
        let keys_dir = TinyBridgeConfig::data_dir().join("keys");
        let ssh_config_path = home.join(".ssh/config");

        EnvironmentManager {
            environments: HashMap::new(),
            shell_sessions: Arc::new(RwLock::new(HashMap::new())),
            vm_manager: VmManager::new(),
            ssh_key_manager: SshKeyManager::new(&keys_dir),
            ssh_config_manager: SshConfigManager::new(&ssh_config_path),
            clipboard_sync_manager: ClipboardSyncManager::new(),
            assets_dir,
            boot_tiers: BootTierConfig::default(),
        }
    }

    #[cfg(test)]
    fn with_assets_dir(assets_dir: PathBuf) -> Self {
        let mut manager = Self::new();
        manager.assets_dir = assets_dir;
        manager
    }

    #[instrument(skip(self), fields(env_name = %name.as_ref().unwrap_or(&"default".to_string())))]
    pub async fn up(
        &mut self,
        name: Option<String>,
        _env_yaml_path: Option<String>,
    ) -> Result<serde_json::Value> {
        let boot_start = Instant::now();
        let env_name = name.unwrap_or_else(|| "default".to_string());

        tracing::debug!("Environment up requested");

        if self.environments.contains_key(&env_name) {
            let existing = &self.environments[&env_name];
            if let EnvironmentStatus::Running { .. } = existing.status {
                return Err(anyhow!("Environment already running"));
            }
        }

        let env_id = Uuid::new_v4();
        let resources = tinybridge_core::Resources {
            cpu: 2,
            memory_bytes: 4 * 1024_u64.pow(3),
            disk_bytes: 20 * 1024_u64.pow(3),
            gpu: None,
        };

        // Create VM via tinybridge-vz. `kernel`/`disk.raw` are required; `initrd`/`seed.iso`
        // are optional (a raw cloud image with no initrd/seed still boots, just with no
        // datasource-provided login - see README's "The actual fix" for how these four
        // files are produced today; there is no automated download/extraction pipeline
        // yet, so populating `self.assets_dir` is still a manual, documented step).
        let kernel_path = self.assets_dir.join("kernel");
        let disk_path = self.assets_dir.join("disk.raw");
        let initrd_path = self.assets_dir.join("initrd");
        let seed_image_path = self.assets_dir.join("seed.iso");

        // Fail fast with an actionable message instead of handing tinybridge-vmhost paths
        // that don't exist - previously this spawned the vmhost process regardless, which
        // then failed deep inside Virtualization.framework with an opaque VZErrorDomain
        // error that gave no hint the real problem was "no boot assets were ever placed
        // here."
        if !kernel_path.is_file() || !disk_path.is_file() {
            return Err(anyhow!(
                "missing boot assets in {}: need at least `kernel` and `disk.raw` (see \
                 README.md's \"The actual fix\" section for how to produce them from a \
                 real Ubuntu cloud image; `initrd` and `seed.iso` are optional but \
                 required for a usable login)",
                self.assets_dir.display()
            ));
        }

        let boot_assets = VmBootAssets {
            kernel_path: kernel_path.to_string_lossy().to_string(),
            disk_path: disk_path.to_string_lossy().to_string(),
            initrd_path: initrd_path
                .is_file()
                .then(|| initrd_path.to_string_lossy().to_string()),
            seed_image_path: seed_image_path
                .is_file()
                .then(|| seed_image_path.to_string_lossy().to_string()),
        };

        self.vm_manager
            .create_vm(env_id, env_name.clone(), boot_assets, resources.clone())
            .await?;

        // Create environment entry
        self.environments
            .entry(env_name.clone())
            .or_insert_with(|| Environment {
                id: env_id,
                name: env_name.clone(),
                version: "1.0.0".to_string(),
                description: Some("TinyBridge environment".to_string()),
                substrate: tinybridge_core::SubstrateConfig {
                    os: "ubuntu".to_string(),
                    version: Some("24.04".to_string()),
                    kernel: None,
                    arch: vec![tinybridge_core::Arch::Arm64],
                    display_mode: None,
                },
                resources,
                native_tools: vec![],
                status: EnvironmentStatus::Stopped,
                created_at: Utc::now(),
                started_at: None,
                ip_address: None,
                dds_configured: false,
                dds_configured_at: None,
                shell_capable: false,
                ssh_configured: false,
            });

        // Update environment status - starting
        if let Some(env) = self.environments.get_mut(&env_name) {
            env.status = EnvironmentStatus::Starting { progress_pct: 0 };
        }

        // Start the VM for real (issues a genuine vmhost.start JSON-RPC call, which drives
        // tinybridge_vz::VirtualMachine::start() -> Virtualization.framework). Any real
        // failure (missing entitlement, framework unavailable, invalid image, ...)
        // propagates from here rather than being swallowed.
        if let Err(e) = self.vm_manager.start_vm(env_id).await {
            // Without this, a failure here leaves the environment permanently stuck in
            // Starting (set above) -- not Running, so `down`/`destroy` refuse to touch it
            // (they require is_running()), and not terminal either (only Stopped/Error
            // are), so there is no way to ever remove or relaunch it. Mirror the same
            // Error transition used by the polling-loop failure paths below.
            let message = e.to_string();
            if let Some(env) = self.environments.get_mut(&env_name) {
                env.status = EnvironmentStatus::Error {
                    message: message.clone(),
                };
            }
            return Err(anyhow!("VM failed to start: {message}"));
        }

        // Poll the vmhost for the VM's real hypervisor-level state instead of fabricating
        // boot progress. This only proves the hypervisor itself reached "Running" - it does
        // not (yet) probe guest-level readiness (SSH, etc.), since that requires a real,
        // verified-bootable guest image pipeline that doesn't exist yet (see
        // scripts/build-rootfs-multi-tier.sh).
        const POLL_ATTEMPTS: u32 = 10;
        const POLL_INTERVAL_MS: u64 = 300;
        let mut last_status: Option<serde_json::Value> = None;
        let mut reached_running = false;

        for attempt in 0..POLL_ATTEMPTS {
            tokio::time::sleep(tokio::time::Duration::from_millis(POLL_INTERVAL_MS)).await;

            let progress_pct = (((attempt + 1) * 100 / POLL_ATTEMPTS) as u8).min(99);
            if let Some(env) = self.environments.get_mut(&env_name) {
                env.status = EnvironmentStatus::Starting { progress_pct };
            }

            match self.vm_manager.status_vm(env_id).await {
                Ok(status) => {
                    let state = status.get("state").and_then(|s| s.as_str()).unwrap_or("");
                    last_status = Some(status.clone());
                    if state == "Running" {
                        reached_running = true;
                        break;
                    }
                    if state == "Error" || state == "unavailable" {
                        let message = status
                            .get("error")
                            .and_then(|e| e.as_str())
                            .unwrap_or("VM entered an error state")
                            .to_string();
                        if let Some(env) = self.environments.get_mut(&env_name) {
                            env.status = EnvironmentStatus::Error {
                                message: message.clone(),
                            };
                        }
                        return Err(anyhow!("VM failed to start: {message}"));
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "status_vm query failed while polling boot state");
                }
            }
        }

        if !reached_running {
            let message = format!(
                "VM did not reach Running state within {}ms (last status: {})",
                POLL_ATTEMPTS as u64 * POLL_INTERVAL_MS,
                last_status
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "none".to_string())
            );
            if let Some(env) = self.environments.get_mut(&env_name) {
                env.status = EnvironmentStatus::Error {
                    message: message.clone(),
                };
            }
            return Err(anyhow!(message));
        }

        let boot_duration_ms = boot_start.elapsed().as_millis() as u64;

        // Determine which boot tier was achieved
        let tier_1_timeout = self
            .boot_tiers
            .timeout_for_tier(1)
            .unwrap_or_default()
            .as_millis() as u64;
        let tier_2_timeout = self
            .boot_tiers
            .timeout_for_tier(2)
            .unwrap_or_default()
            .as_millis() as u64;
        let tier_3_timeout = self
            .boot_tiers
            .timeout_for_tier(3)
            .unwrap_or_default()
            .as_millis() as u64;

        let boot_tier = if boot_duration_ms <= tier_1_timeout {
            1
        } else if boot_duration_ms <= tier_2_timeout {
            2
        } else if boot_duration_ms <= tier_3_timeout {
            3
        } else {
            4
        };

        // Update environment status - running. ip_address comes from the real status poll
        // above (tinybridge-vz's boot monitor observing the guest via the VZ NAT device) -
        // it is None until the guest is actually detected, never a fabricated placeholder.
        let real_ip_address = last_status
            .as_ref()
            .and_then(|s| s.get("ip_address"))
            .and_then(|ip| ip.as_str())
            .map(|s| s.to_string());
        if let Some(env) = self.environments.get_mut(&env_name) {
            env.status = EnvironmentStatus::Running { uptime_secs: 0 };
            env.started_at = Some(Utc::now());
            env.ip_address = real_ip_address.clone();
        }

        // Generate SSH key for this environment
        let mut ssh_configured = false;
        match self
            .ssh_key_manager
            .generate_key(env_id, &env_name, KeyType::Ed25519)
            .await
        {
            Ok(keypair) => {
                tracing::info!("SSH key generated: {}", keypair.fingerprint);

                // Create SSH config entry - only once the guest's real IP is known. This
                // used to hardcode "192.168.105.2" unconditionally, so every environment's
                // ~/.ssh/config entry pointed at the same fixed address regardless of the
                // VM's actual, real IP (`real_ip_address` above, resolved from the real DHCP
                // lease file) - silently wrong whenever that differs, and a real risk of
                // connecting to a stale/different VM that happens to hold that address.
                match Self::build_ssh_entry(
                    env_id,
                    &env_name,
                    &real_ip_address,
                    &keypair.private_key_path,
                ) {
                    Some(ssh_entry) => {
                        let hostname = ssh_entry.hostname.clone();
                        if let Err(e) = self.ssh_config_manager.add_entry(&ssh_entry) {
                            tracing::warn!("Failed to add SSH config entry: {}", e);
                        } else {
                            ssh_configured = true;
                            tracing::debug!(hostname, "SSH configuration registered");
                        }
                    }
                    None => {
                        tracing::warn!(
                            "Guest IP not yet resolved; skipping SSH config entry rather \
                             than writing one with a guessed address"
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to generate SSH key: {}", e);
            }
        }

        // Provision DDS configuration for this environment
        let mut dds_configured = false;
        let mut dds_configured_at = None;
        match self.provision_dds(&env_name, env_id).await {
            Ok(_) => {
                dds_configured = true;
                dds_configured_at = Some(Utc::now());
                tracing::info!("DDS configuration provisioned");
            }
            Err(e) => {
                tracing::warn!("Failed to provision DDS configuration: {}", e);
                tracing::info!("Note: Environment may still work, but shell access may be limited");
            }
        }

        // Now update the environment with the results
        if let Some(env) = self.environments.get_mut(&env_name) {
            env.ssh_configured = ssh_configured;
            env.dds_configured = dds_configured;
            env.dds_configured_at = dds_configured_at;
            env.shell_capable = dds_configured;
        }

        // Start clipboard sync for this environment
        self.clipboard_sync_manager
            .start_sync(env_id, "127.0.0.1".to_string(), 2222, "user".to_string())
            .await;

        // Record boot time metric (exported to OTel)
        crate::otel::record_boot_time(&env_name, boot_duration_ms, "success");

        tracing::info!(
            boot_time_ms = boot_duration_ms,
            boot_tier = boot_tier,
            tier_1_target_ms = tier_1_timeout,
            tier_2_target_ms = tier_2_timeout,
            tier_3_target_ms = tier_3_timeout,
            ip_address = "192.168.105.2",
            "Environment up complete"
        );

        let environment = &self.environments[&env_name];
        Ok(serde_json::to_value(UpResponse {
            id: environment.id.to_string(),
            name: environment.name.clone(),
            status: "running".to_string(),
            ip_address: Some("192.168.105.2".to_string()),
        })?)
    }

    #[instrument(skip(self), fields(env_name = %name.as_ref().unwrap_or(&"default".to_string()), force = force))]
    pub async fn down(&mut self, name: Option<String>, force: bool) -> Result<serde_json::Value> {
        let env_name = name.unwrap_or_else(|| "default".to_string());

        tracing::debug!("Environment down requested");

        // Check environment exists and get ID
        let env_id = {
            let env = self
                .environments
                .get(&env_name)
                .ok_or_else(|| anyhow!("Environment not found"))?;

            // A non-forced `down` only makes sense on a genuinely running
            // environment. But `destroy` calls this with force=true
            // specifically to clean up broken state -- an environment stuck
            // in Starting (a launch whose VM failed to start, see `up`
            // above) or already in Error is neither Running (so this check
            // would reject it) nor Stopped (so nothing else can remove it
            // either), which otherwise leaves it permanently stuck with no
            // escape hatch. Only enforce the running-check when not forcing.
            if !force && !env.status.is_running() {
                return Err(anyhow!("Environment not running"));
            }

            env.id
        };

        // Mark as stopping
        if let Some(env) = self.environments.get_mut(&env_name) {
            env.status = EnvironmentStatus::Stopping;
        }

        // Stop clipboard sync
        self.clipboard_sync_manager.stop_sync(env_id).await;

        // Remove SSH config entry
        if let Err(e) = self.ssh_config_manager.remove_entry(&env_name) {
            tracing::warn!("Failed to remove SSH config entry: {}", e);
        }

        // Archive SSH keys (don't delete, keep for recovery)
        if let Err(e) = self.ssh_key_manager.delete_key(env_id) {
            tracing::warn!("Failed to archive SSH keys: {}", e);
        }

        // Stop the actual VM
        if force {
            tracing::info!("Force stopping environment");
            self.vm_manager.force_stop_vm(env_id).await?;
        } else {
            tracing::info!("Gracefully stopping environment");
            self.vm_manager.stop_vm(env_id).await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Mark as stopped
        if let Some(env) = self.environments.get_mut(&env_name) {
            env.status = EnvironmentStatus::Stopped;
            env.started_at = None;
            env.ip_address = None;
        }

        // Clean up VM handle
        self.vm_manager.destroy_vm(env_id)?;

        tracing::info!("Environment down complete");

        let environment = &self.environments[&env_name];
        Ok(serde_json::to_value(DownResponse {
            name: environment.name.clone(),
            status: "stopped".to_string(),
        })?)
    }

    pub fn status(&self, name: Option<String>) -> Result<serde_json::Value> {
        let envs: Vec<EnvironmentSummary> = if let Some(n) = name {
            self.environments
                .get(&n)
                .map(|e| self.to_summary(e))
                .into_iter()
                .collect()
        } else {
            self.environments
                .values()
                .map(|e| self.to_summary(e))
                .collect()
        };

        Ok(serde_json::to_value(StatusResponse { environments: envs })?)
    }

    pub fn list(&self) -> Result<serde_json::Value> {
        let envs: Vec<EnvironmentSummary> = self
            .environments
            .values()
            .map(|e| self.to_summary(e))
            .collect();

        Ok(serde_json::to_value(ListResponse { environments: envs })?)
    }

    pub async fn shell(&self, name: Option<String>) -> Result<serde_json::Value> {
        let env_name = name.as_deref().unwrap_or("default");

        // Verify environment exists
        let env = self
            .environments
            .get(env_name)
            .ok_or_else(|| anyhow!("Environment '{}' not found", env_name))?;

        // Verify environment is running
        if !env.status.is_running() {
            return Err(anyhow!(
                "Environment '{}' is not running (status: {:?})",
                env_name,
                env.status
            ));
        }

        // Verify SSH is configured
        if !env.ssh_configured {
            return Err(anyhow!(
                "SSH not configured for environment '{}'. Try running: tinybridge repair {}",
                env_name,
                env_name
            ));
        }

        // Verify DDS is configured
        if !env.dds_configured {
            return Err(anyhow!(
                "DDS configuration missing for environment '{}'. Try running: tinybridge repair {}",
                env_name,
                env_name
            ));
        }

        // Create shell session
        let shell_id = Uuid::new_v4().to_string();
        let session = ShellSession {
            id: shell_id.clone(),
            env_id: env.id,
            created_at: Utc::now(),
        };

        {
            let mut sessions = self.shell_sessions.write().await;
            sessions.insert(shell_id.clone(), session);
        }

        tracing::info!(
            shell_id = %shell_id,
            environment = env_name,
            "Shell session created"
        );

        Ok(json!({
            "shell_id": shell_id,
            "shell": "bash",
            "environment": env_name,
            "status": "ready",
            "socket_path": TinyBridgeConfig::shell_socket_path(&shell_id),
        }))
    }

    async fn provision_dds(&self, env_name: &str, env_id: Uuid) -> Result<()> {
        tracing::info!(
            "Provisioning DDS configuration for environment '{}'",
            env_name
        );

        let dds_dir = TinyBridgeConfig::data_dir().join("dds").join(env_name);
        std::fs::create_dir_all(&dds_dir)?;

        let dds_config_path = dds_dir.join("dds_config.yaml");
        let dds_config = format!(
            r#"environment: {}
env_id: {}
configured_at: {}
multicast_enabled: true
domain_id: 0
"#,
            env_name,
            env_id,
            Utc::now().to_rfc3339()
        );

        std::fs::write(&dds_config_path, dds_config)?;
        tracing::debug!(
            config_path = %dds_config_path.display(),
            "DDS configuration written"
        );

        Ok(())
    }

    pub async fn repair(&mut self, name: Option<String>) -> Result<serde_json::Value> {
        let env_name = name.as_deref().unwrap_or("default");

        tracing::info!("Repairing environment '{}'", env_name);

        // Check environment exists and is running
        {
            let env = self
                .environments
                .get(env_name)
                .ok_or_else(|| anyhow!("Environment '{}' not found", env_name))?;

            if !env.status.is_running() {
                return Err(anyhow!(
                    "Cannot repair stopped environment. Start it first with: tinybridge up {}",
                    env_name
                ));
            }
        }

        // Re-establish SSH configuration if needed
        {
            let env = self
                .environments
                .get(env_name)
                .ok_or_else(|| anyhow!("Environment '{}' disappeared during repair", env_name))?;
            if !env.ssh_configured {
                tracing::info!("Re-establishing SSH configuration...");
                match self
                    .ssh_key_manager
                    .generate_key(env.id, env_name, KeyType::Ed25519)
                    .await
                {
                    Ok(keypair) => {
                        let ssh_entry = SshConfigEntry {
                            env_id: env.id,
                            alias: env_name.to_string(),
                            hostname: env
                                .ip_address
                                .clone()
                                .unwrap_or_else(|| "192.168.105.2".to_string()),
                            user: "user".to_string(),
                            port: 22,
                            identity_file: keypair.private_key_path,
                            options: Default::default(),
                        };

                        self.ssh_config_manager.add_entry(&ssh_entry)?;
                        tracing::info!("SSH configuration restored");
                    }
                    Err(e) => {
                        return Err(anyhow!("Failed to restore SSH configuration: {}", e));
                    }
                }
            }
        }

        // Re-provision DDS configuration if needed
        {
            let env = self
                .environments
                .get(env_name)
                .ok_or_else(|| anyhow!("Environment '{}' disappeared during repair", env_name))?;
            if !env.dds_configured {
                tracing::info!("Re-provisioning DDS configuration...");
                self.provision_dds(env_name, env.id).await?;
                tracing::info!("DDS configuration restored");
            }
        }

        // Update environment metadata
        if let Some(env) = self.environments.get_mut(env_name) {
            env.ssh_configured = true;
            env.dds_configured = true;
            env.dds_configured_at = Some(Utc::now());
            env.shell_capable = true;
        }

        // Validate repair was successful
        match self.validate_environment(env_name).await {
            Ok(validation) => {
                let env = self.environments.get(env_name).ok_or_else(|| {
                    anyhow!("Environment '{}' disappeared during repair", env_name)
                })?;
                tracing::info!("Environment validation result: {}", validation);
                Ok(json!({
                    "status": "repaired",
                    "environment": env_name,
                    "ssh_configured": env.ssh_configured,
                    "dds_configured": env.dds_configured,
                    "shell_capable": env.shell_capable,
                    "validation": validation,
                }))
            }
            Err(e) => {
                tracing::warn!("Validation after repair failed: {}", e);
                Ok(json!({
                    "status": "repaired_with_warnings",
                    "environment": env_name,
                    "warning": e.to_string(),
                }))
            }
        }
    }

    pub async fn validate_environment(&self, name: &str) -> Result<String> {
        let env = self
            .environments
            .get(name)
            .ok_or_else(|| anyhow!("Environment not found"))?;

        if !env.status.is_running() {
            return Err(anyhow!("Environment is not running"));
        }

        let mut checks = vec![];

        if env.status.is_running() {
            checks.push("✓ VM running");
        }

        if env.ssh_configured {
            checks.push("✓ SSH configured");
        } else {
            checks.push("✗ SSH not configured");
        }

        if env.dds_configured {
            checks.push("✓ DDS configured");
        } else {
            checks.push("✗ DDS not configured");
        }

        if env.shell_capable {
            checks.push("✓ Shell sessions available");
        } else {
            checks.push("✗ Shell sessions unavailable");
        }

        Ok(checks.join("\n"))
    }

    pub fn boot_tier_info(&self) -> Result<serde_json::Value> {
        let tiers: Vec<_> = (1..=4)
            .filter_map(|tier_num| {
                self.boot_tiers.tier(tier_num).map(|tier| {
                    json!({
                        "tier": tier.tier,
                        "name": tier.name,
                        "description": tier.description,
                        "timeout_ms": tier.timeout_ms,
                        "critical": tier.critical,
                        "start_type": format!("{:?}", tier.start_type),
                        "services": tier.services,
                    })
                })
            })
            .collect();

        Ok(json!({
            "strategy": self.boot_tiers.strategy,
            "tiers": tiers,
        }))
    }

    /// Builds the SSH config entry for a freshly-started environment, or `None` if the
    /// guest's real IP hasn't been resolved yet - never fabricates a hostname.
    fn build_ssh_entry(
        env_id: Uuid,
        env_name: &str,
        real_ip_address: &Option<String>,
        identity_file: &std::path::Path,
    ) -> Option<SshConfigEntry> {
        let hostname = real_ip_address.clone()?;
        Some(SshConfigEntry {
            env_id,
            alias: env_name.to_string(),
            hostname,
            user: "user".to_string(),
            port: 22,
            identity_file: identity_file.to_path_buf(),
            options: Default::default(),
        })
    }

    fn to_summary(&self, env: &Environment) -> EnvironmentSummary {
        let uptime_secs = match env.status {
            EnvironmentStatus::Running { uptime_secs } => Some(uptime_secs),
            _ => None,
        };

        EnvironmentSummary {
            id: env.id.to_string(),
            name: env.name.clone(),
            status: format!("{:?}", env.status).to_lowercase(),
            ip_address: env.ip_address.clone(),
            uptime_secs,
        }
    }
}

impl Default for EnvironmentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_ssh_entry_is_none_when_guest_ip_is_not_yet_resolved() {
        let entry = EnvironmentManager::build_ssh_entry(
            Uuid::new_v4(),
            "test-env",
            &None,
            std::path::Path::new("/tmp/id_ed25519"),
        );
        assert!(
            entry.is_none(),
            "must not fabricate a hostname when the guest IP is unknown"
        );
    }

    #[test]
    fn build_ssh_entry_uses_the_real_resolved_ip_not_a_hardcoded_placeholder() {
        let entry = EnvironmentManager::build_ssh_entry(
            Uuid::new_v4(),
            "test-env",
            &Some("192.168.64.3".to_string()),
            std::path::Path::new("/tmp/id_ed25519"),
        )
        .expect("a real IP was provided");

        assert_eq!(entry.hostname, "192.168.64.3");
        assert_ne!(
            entry.hostname, "192.168.105.2",
            "must not fall back to the old hardcoded placeholder IP"
        );
        assert_eq!(entry.alias, "test-env");
    }

    #[tokio::test]
    async fn up_fails_fast_with_actionable_error_when_boot_assets_are_missing() {
        let empty_dir = tempfile::tempdir().unwrap();
        let mut manager = EnvironmentManager::with_assets_dir(empty_dir.path().to_path_buf());

        let err = manager
            .up(Some("test-env".to_string()), None)
            .await
            .expect_err("up() must fail when kernel/disk.raw don't exist");

        let message = err.to_string();
        assert!(
            message.contains("missing boot assets"),
            "expected an actionable missing-assets error, got: {message}"
        );
        assert!(
            message.contains("kernel") && message.contains("disk.raw"),
            "error should name the required files, got: {message}"
        );
        // Must fail before ever touching the environments map or spawning a vmhost.
        assert!(manager.environments.is_empty());
    }

    #[tokio::test]
    async fn up_passes_asset_validation_once_required_files_exist() {
        let assets_dir = tempfile::tempdir().unwrap();
        std::fs::write(assets_dir.path().join("kernel"), b"fake-kernel").unwrap();
        std::fs::write(assets_dir.path().join("disk.raw"), b"fake-disk").unwrap();
        let mut manager = EnvironmentManager::with_assets_dir(assets_dir.path().to_path_buf());

        let err = manager
            .up(Some("test-env".to_string()), None)
            .await
            .expect_err(
                "up() still fails - there's no real tinybridge-vmhost binary in a test PATH",
            );

        // The point of this test: it must fail for a *different* reason than missing
        // assets (spawning tinybridge-vmhost, which isn't installed in the test
        // environment) - proving the fail-fast check itself no longer fires once
        // kernel/disk.raw are present, even without initrd/seed.iso.
        assert!(
            !err.to_string().contains("missing boot assets"),
            "asset validation should have passed, got: {err}"
        );
    }

    /// Real-hardware verification that `up()` - the actual `tinybridge launch`/daemon code
    /// path, not the standalone `vz_boot_test` example - drives a real
    /// Virtualization.framework VM to a real `Running` state using the
    /// initrd/seed_image plumbing added in this pass.
    ///
    /// Requires (not available in CI, hence `#[ignore]`):
    /// - `kernel`, `disk.raw`, `initrd` present under the real
    ///   `dirs::cache_dir()/TinyBridge/assets` (see README's "The actual fix" for how to
    ///   produce them from a real Ubuntu cloud image; `seed.iso` is optional - this test
    ///   only checks the hypervisor reaches `Running`, not a full guest login).
    /// - A `tinybridge-vmhost` binary on `PATH`, codesigned with
    ///   `crates/tinybridge-vmhost/tinybridge-vmhost.entitlements`, with
    ///   `libTinyBridgeVZBridge.dylib` alongside it (`@executable_path` rpath).
    ///
    /// Run manually: `cargo test -p tinybridge-daemon --release -- --ignored real_vm_boot`
    /// with `PATH="$(pwd)/target/release:$PATH"`.
    #[tokio::test]
    #[ignore = "requires real boot assets + a codesigned tinybridge-vmhost on PATH"]
    async fn up_drives_a_real_vm_to_running_state_via_the_daemon_code_path() {
        let mut manager = EnvironmentManager::new();

        let up_result = manager.up(Some("real-boot-verify".to_string()), None).await;
        // Always try to tear the VM down, even if `up()` failed partway through, so a
        // failed run doesn't leave a real vmhost process/socket behind.
        let down_result = manager
            .down(Some("real-boot-verify".to_string()), true)
            .await;

        let up_value = up_result.expect("up() should reach Running with real assets present");
        assert_eq!(
            up_value.get("status").and_then(|s| s.as_str()),
            Some("running"),
            "expected a running status, got: {up_value}"
        );
        down_result.expect("down(force=true) should always succeed as cleanup");
    }
}
