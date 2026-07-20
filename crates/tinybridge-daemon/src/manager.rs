use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use tinybridge_core::{
    DownResponse, Environment, EnvironmentStatus, EnvironmentSummary, ListResponse, StatusResponse,
    UpResponse,
};

pub struct EnvironmentManager {
    environments: HashMap<String, Environment>,
}

impl EnvironmentManager {
    pub fn new() -> Self {
        EnvironmentManager {
            environments: HashMap::new(),
        }
    }

    pub async fn up(
        &mut self,
        name: Option<String>,
        _env_yaml_path: Option<String>,
    ) -> Result<serde_json::Value> {
        let env_name = name.unwrap_or_else(|| "default".to_string());

        if self.environments.contains_key(&env_name) {
            let existing = &self.environments[&env_name];
            if let EnvironmentStatus::Running { .. } = existing.status {
                return Err(anyhow!("Environment already running"));
            }
        }

        let env = self
            .environments
            .entry(env_name.clone())
            .or_insert_with(|| Environment {
                id: Uuid::new_v4(),
                name: env_name.clone(),
                version: "1.0.0".to_string(),
                description: Some("Stub environment".to_string()),
                substrate: tinybridge_core::SubstrateConfig {
                    os: "ubuntu-24.04".to_string(),
                    kernel: None,
                    arch: vec![tinybridge_core::Arch::Arm64],
                },
                resources: tinybridge_core::Resources {
                    cpu: 2,
                    memory_bytes: 4 * 1024_u64.pow(3),
                    disk_bytes: 20 * 1024_u64.pow(3),
                },
                native_tools: vec![],
                status: EnvironmentStatus::Stopped,
                created_at: Utc::now(),
                started_at: None,
                ip_address: None,
            });

        env.status = EnvironmentStatus::Starting { progress_pct: 0 };

        // Simulate progress
        for pct in [25, 50, 75, 100] {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            env.status = EnvironmentStatus::Starting { progress_pct: pct };
        }

        env.status = EnvironmentStatus::Running { uptime_secs: 0 };
        env.started_at = Some(Utc::now());
        env.ip_address = Some("192.168.105.2".to_string());

        Ok(serde_json::to_value(UpResponse {
            id: env.id.to_string(),
            name: env.name.clone(),
            status: "running".to_string(),
            ip_address: Some("192.168.105.2".to_string()),
        })?)
    }

    pub async fn down(&mut self, name: Option<String>, _force: bool) -> Result<serde_json::Value> {
        let env_name = name.unwrap_or_else(|| "default".to_string());

        let env = self
            .environments
            .get_mut(&env_name)
            .ok_or_else(|| anyhow!("Environment not found"))?;

        if !env.status.is_running() {
            return Err(anyhow!("Environment not running"));
        }

        env.status = EnvironmentStatus::Stopping;
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        env.status = EnvironmentStatus::Stopped;
        env.started_at = None;
        env.ip_address = None;

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
