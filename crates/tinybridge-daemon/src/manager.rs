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
    UpResponse, TinyBridgeConfig,
};
use tinybridge_ssh::{KeyType, SshConfigEntry, SshConfigManager, SshKeyManager};

use crate::boot_tiers::BootTierConfig;
use crate::clipboard_sync::ClipboardSyncManager;
use crate::vz::VmManager;

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

        // Create VM via tinybridge-vz
        let kernel_path = self.assets_dir.join("vmlinux");
        let disk_path = self.assets_dir.join("rootfs.img");

        self.vm_manager
            .create_vm(
                env_id,
                env_name.clone(),
                kernel_path.to_string_lossy().to_string(),
                disk_path.to_string_lossy().to_string(),
                resources.clone(),
            )
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

        // Start the VM
        self.vm_manager.start_vm(env_id).await?;

        // Simulate boot progress while VM starts
        for pct in [25, 50, 75, 100] {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if let Some(env) = self.environments.get_mut(&env_name) {
                env.status = EnvironmentStatus::Starting { progress_pct: pct };
            }
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

        // Update environment status - running
        if let Some(env) = self.environments.get_mut(&env_name) {
            env.status = EnvironmentStatus::Running { uptime_secs: 0 };
            env.started_at = Some(Utc::now());
            env.ip_address = Some("192.168.105.2".to_string());
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

                // Create SSH config entry
                let ssh_entry = SshConfigEntry {
                    env_id,
                    alias: env_name.clone(),
                    hostname: "192.168.105.2".to_string(),
                    user: "user".to_string(),
                    port: 22,
                    identity_file: keypair.private_key_path.clone(),
                    options: Default::default(),
                };

                if let Err(e) = self.ssh_config_manager.add_entry(&ssh_entry) {
                    tracing::warn!("Failed to add SSH config entry: {}", e);
                } else {
                    ssh_configured = true;
                    tracing::debug!("SSH configuration registered");
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

            if !env.status.is_running() {
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
                env_name, env_name
            ));
        }

        // Verify DDS is configured
        if !env.dds_configured {
            return Err(anyhow!(
                "DDS configuration missing for environment '{}'. Try running: tinybridge repair {}",
                env_name, env_name
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
        tracing::info!("Provisioning DDS configuration for environment '{}'", env_name);

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
            let env = self.environments.get(env_name).unwrap();
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
                            hostname: env.ip_address.clone().unwrap_or_else(|| "192.168.105.2".to_string()),
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
            let env = self.environments.get(env_name).unwrap();
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
                let env = self.environments.get(env_name).unwrap();
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
