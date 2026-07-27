use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinyBridgeConfig {
    pub socket_path: PathBuf,
    pub data_dir: PathBuf,
    pub log_level: String,
    pub kernel_path: PathBuf,
    pub initrd_path: PathBuf,
    pub default_resources: DefaultResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultResources {
    pub cpu: u32,
    pub memory_gb: u32,
    pub disk_gb: u32,
}

impl TinyBridgeConfig {
    pub fn data_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("TinyBridge")
    }

    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("TinyBridge")
    }

    pub fn logs_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library/Logs/TinyBridge")
    }

    pub fn socket_path() -> PathBuf {
        Self::data_dir().join("tinybridge.sock")
    }

    pub fn shell_socket_path(shell_id: &str) -> PathBuf {
        Self::data_dir().join(format!("shells/{}.sock", shell_id))
    }

    pub fn shells_dir() -> PathBuf {
        Self::data_dir().join("shells")
    }

    pub fn vmhost_socket_path(&self, env_id: &str) -> PathBuf {
        self.data_dir.join(format!("vmhost/{}.sock", env_id))
    }

    pub fn vmhosts_dir(&self) -> PathBuf {
        self.data_dir.join("vmhost")
    }
}

impl Default for TinyBridgeConfig {
    fn default() -> Self {
        TinyBridgeConfig {
            socket_path: Self::socket_path(),
            data_dir: Self::data_dir(),
            log_level: "info".to_string(),
            kernel_path: Self::cache_dir().join("assets/vmlinux"),
            initrd_path: Self::cache_dir().join("assets/initrd"),
            default_resources: DefaultResources {
                cpu: 2,
                memory_gb: 4,
                disk_gb: 20,
            },
        }
    }
}
