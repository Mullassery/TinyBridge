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
    pub fn socket_path() -> PathBuf {
        if cfg!(unix) {
            PathBuf::from("/var/run/tinybridge.sock")
        } else {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(".tinybridge/tinybridge.sock")
        }
    }

    pub fn data_dir() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".tinybridge")
    }

    pub fn shell_socket_path(shell_id: &str) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(format!(".tinybridge/shells/{}.sock", shell_id))
    }

    pub fn shells_dir() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".tinybridge/shells")
    }
}

impl Default for TinyBridgeConfig {
    fn default() -> Self {
        TinyBridgeConfig {
            socket_path: Self::socket_path(),
            data_dir: Self::data_dir(),
            log_level: "info".to_string(),
            kernel_path: PathBuf::from("/usr/local/bin/tinybridge-kernel"),
            initrd_path: PathBuf::from("/usr/local/bin/tinybridge-initrd"),
            default_resources: DefaultResources {
                cpu: 2,
                memory_gb: 4,
                disk_gb: 20,
            },
        }
    }
}
