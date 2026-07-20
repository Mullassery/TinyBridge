use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tracing::instrument;
use uuid::Uuid;

use tinybridge_core::{
    DownResponse, Environment, EnvironmentStatus, EnvironmentSummary, ListResponse, StatusResponse,
    UpResponse,
};

use crate::boot_tiers::BootTierConfig;
use crate::vz::VmManager;

pub struct EnvironmentManager {
    environments: HashMap<String, Environment>,
    vm_manager: VmManager,
    assets_dir: PathBuf,
    boot_tiers: BootTierConfig,
}

impl EnvironmentManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let assets_dir = home.join(".tinybridge/assets");

        EnvironmentManager {
            environments: HashMap::new(),
            vm_manager: VmManager::new(),
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
        };

        // Create VM via tinybridge-vz
        let kernel_path = self.assets_dir.join("vmlinux");
        let disk_path = self.assets_dir.join("rootfs.img");

        self.vm_manager.create_vm(
            env_id,
            env_name.clone(),
            kernel_path.to_string_lossy().to_string(),
            disk_path.to_string_lossy().to_string(),
            resources.clone(),
        )?;

        let env = self
            .environments
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
                },
                resources,
                native_tools: vec![],
                status: EnvironmentStatus::Stopped,
                created_at: Utc::now(),
                started_at: None,
                ip_address: None,
            });

        env.status = EnvironmentStatus::Starting { progress_pct: 0 };

        // Start the VM
        self.vm_manager.start_vm(env.id)?;

        // Simulate boot progress while VM starts
        for pct in [25, 50, 75, 100] {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            env.status = EnvironmentStatus::Starting { progress_pct: pct };
        }

        let boot_duration_ms = boot_start.elapsed().as_millis() as u64;

        // Determine which boot tier was achieved
        let tier_1_timeout = self.boot_tiers.timeout_for_tier(1).unwrap_or_default().as_millis() as u64;
        let tier_2_timeout = self.boot_tiers.timeout_for_tier(2).unwrap_or_default().as_millis() as u64;
        let tier_3_timeout = self.boot_tiers.timeout_for_tier(3).unwrap_or_default().as_millis() as u64;

        let boot_tier = if boot_duration_ms <= tier_1_timeout {
            1
        } else if boot_duration_ms <= tier_2_timeout {
            2
        } else if boot_duration_ms <= tier_3_timeout {
            3
        } else {
            4
        };

        env.status = EnvironmentStatus::Running { uptime_secs: 0 };
        env.started_at = Some(Utc::now());
        env.ip_address = Some("192.168.105.2".to_string());

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

        Ok(serde_json::to_value(UpResponse {
            id: env.id.to_string(),
            name: env.name.clone(),
            status: "running".to_string(),
            ip_address: Some("192.168.105.2".to_string()),
        })?)
    }

    #[instrument(skip(self), fields(env_name = %name.as_ref().unwrap_or(&"default".to_string()), force = force))]
    pub async fn down(&mut self, name: Option<String>, force: bool) -> Result<serde_json::Value> {
        let env_name = name.unwrap_or_else(|| "default".to_string());

        tracing::debug!("Environment down requested");

        let env = self
            .environments
            .get_mut(&env_name)
            .ok_or_else(|| anyhow!("Environment not found"))?;

        if !env.status.is_running() {
            return Err(anyhow!("Environment not running"));
        }

        env.status = EnvironmentStatus::Stopping;

        // Stop the actual VM
        if force {
            tracing::info!("Force stopping environment");
            self.vm_manager.force_stop_vm(env.id)?;
        } else {
            tracing::info!("Gracefully stopping environment");
            self.vm_manager.stop_vm(env.id)?;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        env.status = EnvironmentStatus::Stopped;
        env.started_at = None;
        env.ip_address = None;

        // Clean up VM handle
        self.vm_manager.destroy_vm(env.id)?;

        tracing::info!("Environment down complete");

        Ok(serde_json::to_value(DownResponse {
            name: env.name.clone(),
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

    pub async fn shell(&self, _name: Option<String>) -> Result<serde_json::Value> {
        Ok(json!({"shell": "bash", "status": "connecting"}))
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
