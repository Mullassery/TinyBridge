/// env.yaml Configuration Parser
/// Phase 4.0.1: Config Foundation
///
/// Parses TinyBridge environment configuration from YAML files with validation
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name
    pub name: String,
    /// Profile name (development, production, custom)
    #[serde(default = "default_profile")]
    pub profile: String,
    /// Resource allocation
    #[serde(default)]
    pub resources: ResourceSpec,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Volume mounts
    #[serde(default)]
    pub volumes: Vec<VolumeMount>,
    /// Network configuration
    #[serde(default)]
    pub network: NetworkConfig,
    /// Ports to expose
    #[serde(default)]
    pub ports: Vec<PortMapping>,
    /// Custom configuration
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU cores
    #[serde(default = "default_cpus")]
    pub cpus: u32,
    /// Memory in GB
    #[serde(default = "default_memory")]
    pub memory: u32,
    /// Disk size in GB
    #[serde(default = "default_disk")]
    pub disk: u32,
    /// GPU support (true/false)
    #[serde(default)]
    pub gpu: bool,
}

impl Default for ResourceSpec {
    fn default() -> Self {
        ResourceSpec {
            cpus: 2,
            memory: 4,
            disk: 20,
            gpu: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub source: String,
    pub target: String,
    /// "ro" for read-only, "rw" for read-write
    #[serde(default = "default_mode")]
    pub mode: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bridge network (true) or isolated (false)
    #[serde(default = "default_bridge")]
    pub bridge: bool,
    /// Custom hostname
    pub hostname: Option<String>,
    /// DNS servers
    #[serde(default)]
    pub dns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub vm_port: u16,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    ParseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

impl EnvironmentConfig {
    /// Load configuration from env.yaml file
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    /// Parse configuration from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(yaml).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.name.is_empty() {
            return Err(ConfigError::MissingField("name".to_string()));
        }
        if self.resources.cpus == 0 || self.resources.cpus > 16 {
            return Err(ConfigError::ValidationError(
                "cpus must be between 1 and 16".to_string(),
            ));
        }
        if self.resources.memory == 0 || self.resources.memory > 64 {
            return Err(ConfigError::ValidationError(
                "memory must be between 1GB and 64GB".to_string(),
            ));
        }
        Ok(())
    }

    /// Apply overrides to configuration
    pub fn apply_overrides(&mut self, overrides: &ConfigOverrides) {
        if let Some(cpus) = overrides.cpus {
            self.resources.cpus = cpus;
        }
        if let Some(memory) = overrides.memory {
            self.resources.memory = memory;
        }
        if let Some(profile) = &overrides.profile {
            self.profile.clone_from(profile);
        }
        for (key, value) in &overrides.env_vars {
            self.env.insert(key.clone(), value.clone());
        }
    }
}

/// Runtime configuration overrides
#[derive(Debug, Default, Clone)]
pub struct ConfigOverrides {
    pub cpus: Option<u32>,
    pub memory: Option<u32>,
    pub profile: Option<String>,
    pub env_vars: HashMap<String, String>,
}

// Default values for optional fields
fn default_profile() -> String {
    "development".to_string()
}

fn default_cpus() -> u32 {
    2
}

fn default_memory() -> u32 {
    4
}

fn default_disk() -> u32 {
    20
}

fn default_mode() -> String {
    "rw".to_string()
}

fn default_bridge() -> bool {
    true
}

fn default_protocol() -> String {
    "tcp".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let yaml = r#"
name: my-dev-env
profile: development
resources:
  cpus: 4
  memory: 8
  disk: 40
  gpu: false
env:
  RUST_LOG: debug
volumes:
  - source: /home/user/projects
    target: /projects
    mode: rw
network:
  bridge: true
  hostname: dev-vm
  dns:
    - 1.1.1.1
    - 8.8.8.8
ports:
  - host_port: 8080
    vm_port: 8080
    protocol: tcp
"#;

        let config = EnvironmentConfig::from_yaml(yaml);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.name, "my-dev-env");
        assert_eq!(config.profile, "development");
        assert_eq!(config.resources.cpus, 4);
        assert_eq!(config.resources.memory, 8);
        assert_eq!(config.env.get("RUST_LOG"), Some(&"debug".to_string()));
    }

    #[test]
    fn test_validation() {
        let yaml = r#"
name: ""
profile: production
"#;

        let config = EnvironmentConfig::from_yaml(yaml).unwrap();
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_default_values() {
        let yaml = r#"
name: minimal-env
"#;

        let config = EnvironmentConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.profile, "development");
        assert_eq!(config.resources.cpus, 2);
        assert_eq!(config.resources.memory, 4);
    }

    #[test]
    fn test_config_overrides() {
        let yaml = r#"
name: test-env
resources:
  cpus: 2
  memory: 4
env:
  FOO: bar
"#;

        let mut config = EnvironmentConfig::from_yaml(yaml).unwrap();
        let mut overrides = ConfigOverrides {
            cpus: Some(8),
            ..Default::default()
        };
        overrides
            .env_vars
            .insert("FOO".to_string(), "baz".to_string());

        config.apply_overrides(&overrides);
        assert_eq!(config.resources.cpus, 8);
        assert_eq!(config.env.get("FOO"), Some(&"baz".to_string()));
    }
}
