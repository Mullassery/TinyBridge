use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{Result, TunnelError};

/// Type of SSH tunnel
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TunnelType {
    /// Local port forwarding: local_port → remote_host:remote_port
    LocalForward,
    /// Remote port forwarding: remote_port → local_host:local_port
    RemoteForward,
    /// SOCKS proxy: local_port → dynamic routing through SSH
    SocksProxy,
}

impl std::fmt::Display for TunnelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelType::LocalForward => write!(f, "local-forward"),
            TunnelType::RemoteForward => write!(f, "remote-forward"),
            TunnelType::SocksProxy => write!(f, "socks"),
        }
    }
}

/// Tunnel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    /// Environment ID to tunnel to
    pub env_id: Uuid,

    /// Tunnel type
    pub tunnel_type: TunnelType,

    /// Local port (bind side for LocalForward/SocksProxy)
    pub local_port: u16,

    /// Remote host (target side for LocalForward/RemoteForward)
    pub remote_host: String,

    /// Remote port (target side for LocalForward/RemoteForward)
    pub remote_port: u16,

    /// SSH host address
    pub ssh_host: String,

    /// SSH port
    pub ssh_port: u16,

    /// SSH username
    pub ssh_user: String,
}

/// Active tunnel instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tunnel {
    /// Tunnel ID
    pub id: Uuid,

    /// Configuration
    pub config: TunnelConfig,

    /// Status
    pub status: TunnelStatus,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Bytes transferred (cumulative)
    pub bytes_sent: u64,
    pub bytes_received: u64,

    /// Connection count (for SOCKS)
    pub connection_count: u64,
}

/// Tunnel status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TunnelStatus {
    /// Tunnel is active
    Active,
    /// Tunnel is inactive/closed
    Inactive,
    /// Tunnel failed
    Failed,
}

impl Tunnel {
    /// Create a new tunnel
    pub fn new(config: TunnelConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            status: TunnelStatus::Inactive,
            created_at: Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
            connection_count: 0,
        }
    }

    /// Get tunnel description
    pub fn description(&self) -> String {
        match self.config.tunnel_type {
            TunnelType::LocalForward => {
                format!(
                    "localhost:{} → {}:{}",
                    self.config.local_port, self.config.remote_host, self.config.remote_port
                )
            }
            TunnelType::RemoteForward => {
                format!(
                    "{}:{} → localhost:{}",
                    self.config.remote_host, self.config.remote_port, self.config.local_port
                )
            }
            TunnelType::SocksProxy => {
                format!("SOCKS5 proxy on localhost:{}", self.config.local_port)
            }
        }
    }
}

/// Tunnel manager for lifecycle operations
pub struct TunnelManager {
    tunnels: HashMap<Uuid, Tunnel>,
}

impl TunnelManager {
    /// Create a new tunnel manager
    pub fn new() -> Self {
        Self {
            tunnels: HashMap::new(),
        }
    }

    /// Create a new tunnel
    pub fn create_tunnel(&mut self, config: TunnelConfig) -> Result<Tunnel> {
        // Validate configuration
        if config.local_port == 0 || config.remote_port == 0 {
            return Err(TunnelError::InvalidConfig(
                "Port numbers must be non-zero".to_string(),
            ));
        }

        if config.ssh_host.is_empty() || config.ssh_user.is_empty() {
            return Err(TunnelError::InvalidConfig(
                "SSH host and user must be specified".to_string(),
            ));
        }

        // Check for port conflicts
        for tunnel in self.tunnels.values() {
            if tunnel.config.local_port == config.local_port
                && tunnel.status == TunnelStatus::Active
            {
                return Err(TunnelError::PortBindError(format!(
                    "Port {} already in use",
                    config.local_port
                )));
            }
        }

        let tunnel = Tunnel::new(config);
        self.tunnels.insert(tunnel.id, tunnel.clone());

        Ok(tunnel)
    }

    /// List all tunnels
    pub fn list_tunnels(&self) -> Vec<Tunnel> {
        self.tunnels.values().cloned().collect()
    }

    /// Get tunnel by ID
    pub fn get_tunnel(&self, id: Uuid) -> Result<Tunnel> {
        self.tunnels
            .get(&id)
            .cloned()
            .ok_or_else(|| TunnelError::TunnelNotFound(id.to_string()))
    }

    /// Update tunnel status
    pub fn set_status(&mut self, id: Uuid, status: TunnelStatus) -> Result<()> {
        self.tunnels
            .get_mut(&id)
            .ok_or_else(|| TunnelError::TunnelNotFound(id.to_string()))?
            .status = status;
        Ok(())
    }

    /// Remove a tunnel
    pub fn remove_tunnel(&mut self, id: Uuid) -> Result<Tunnel> {
        self.tunnels
            .remove(&id)
            .ok_or_else(|| TunnelError::TunnelNotFound(id.to_string()))
    }

    /// List active tunnels for an environment
    pub fn list_active_for_env(&self, env_id: Uuid) -> Vec<Tunnel> {
        self.tunnels
            .values()
            .filter(|t| t.config.env_id == env_id && t.status == TunnelStatus::Active)
            .cloned()
            .collect()
    }

    /// Stop all tunnels for an environment
    pub fn stop_all_for_env(&mut self, env_id: Uuid) -> Vec<Tunnel> {
        let mut stopped = Vec::new();
        for tunnel in self.tunnels.values_mut() {
            if tunnel.config.env_id == env_id && tunnel.status == TunnelStatus::Active {
                tunnel.status = TunnelStatus::Inactive;
                stopped.push(tunnel.clone());
            }
        }
        stopped
    }
}

impl Default for TunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_creation() {
        let config = TunnelConfig {
            env_id: Uuid::new_v4(),
            tunnel_type: TunnelType::LocalForward,
            local_port: 8000,
            remote_host: "localhost".to_string(),
            remote_port: 3000,
            ssh_host: "127.0.0.1".to_string(),
            ssh_port: 22,
            ssh_user: "user".to_string(),
        };

        let tunnel = Tunnel::new(config);
        assert_eq!(tunnel.status, TunnelStatus::Inactive);
        assert!(tunnel.description().contains("8000"));
    }

    #[test]
    fn test_tunnel_manager() {
        let mut manager = TunnelManager::new();
        let config = TunnelConfig {
            env_id: Uuid::new_v4(),
            tunnel_type: TunnelType::SocksProxy,
            local_port: 9050,
            remote_host: "unused".to_string(),
            remote_port: 1080,
            ssh_host: "127.0.0.1".to_string(),
            ssh_port: 22,
            ssh_user: "user".to_string(),
        };

        let tunnel = manager.create_tunnel(config).unwrap();
        assert_eq!(manager.list_tunnels().len(), 1);

        manager.set_status(tunnel.id, TunnelStatus::Active).unwrap();
        let updated = manager.get_tunnel(tunnel.id).unwrap();
        assert_eq!(updated.status, TunnelStatus::Active);
    }

    #[test]
    fn test_port_conflict_detection() {
        let mut manager = TunnelManager::new();
        let env_id = Uuid::new_v4();

        let config1 = TunnelConfig {
            env_id,
            tunnel_type: TunnelType::LocalForward,
            local_port: 8000,
            remote_host: "localhost".to_string(),
            remote_port: 3000,
            ssh_host: "127.0.0.1".to_string(),
            ssh_port: 22,
            ssh_user: "user".to_string(),
        };

        let tunnel1 = manager.create_tunnel(config1).unwrap();
        manager
            .set_status(tunnel1.id, TunnelStatus::Active)
            .unwrap();

        let config2 = TunnelConfig {
            env_id,
            tunnel_type: TunnelType::LocalForward,
            local_port: 8000, // Same port!
            remote_host: "localhost".to_string(),
            remote_port: 3001,
            ssh_host: "127.0.0.1".to_string(),
            ssh_port: 22,
            ssh_user: "user".to_string(),
        };

        let result = manager.create_tunnel(config2);
        assert!(result.is_err());
    }
}
