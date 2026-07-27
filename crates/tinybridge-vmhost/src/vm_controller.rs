use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tinybridge_vz::VmConfig;

#[derive(Clone)]
pub struct VmController {
    env_id: Arc<str>,
    config: Arc<VmConfig>,
}

impl VmController {
    pub fn new(env_id: String, config: VmConfig) -> Result<Self> {
        Ok(VmController {
            env_id: Arc::from(env_id),
            config: Arc::new(config),
        })
    }

    pub fn env_id(&self) -> &str {
        &self.env_id
    }

    pub fn show_window(&self) -> Result<()> {
        tracing::debug!(env_id = %self.env_id, "show_window requested");
        Ok(())
    }

    pub fn hide_window(&self) -> Result<()> {
        tracing::debug!(env_id = %self.env_id, "hide_window requested");
        Ok(())
    }

    pub async fn status(&self) -> Result<String> {
        let status = json!({
            "env_id": self.env_id.as_ref(),
            "status": "running",
            "config": {
                "cpu": self.config.cpu_count,
                "memory_gb": self.config.memory_bytes / (1024 * 1024 * 1024),
            }
        });

        Ok(status.to_string())
    }
}
