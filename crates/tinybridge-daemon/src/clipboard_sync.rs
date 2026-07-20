use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};
use uuid::Uuid;

use tinybridge_clipboard::{ClipboardBridge, LinuxClipboard};

/// Manages clipboard sync for a running environment
pub struct ClipboardSyncManager {
    active_syncs: Arc<RwLock<std::collections::HashMap<Uuid, JoinHandle<()>>>>,
}

impl ClipboardSyncManager {
    /// Create a new clipboard sync manager
    pub fn new() -> Self {
        Self {
            active_syncs: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Start clipboard sync for an environment
    ///
    /// # Arguments
    /// * `env_id` - Environment ID
    /// * `ssh_host` - SSH host for the VM
    /// * `ssh_port` - SSH port for the VM
    /// * `ssh_user` - SSH username
    pub async fn start_sync(
        &self,
        env_id: Uuid,
        ssh_host: String,
        ssh_port: u16,
        ssh_user: String,
    ) {
        debug!("Starting clipboard sync for environment {}", env_id);

        let linux_clipboard = LinuxClipboard::new(ssh_host, ssh_port, ssh_user);
        let bridge = ClipboardBridge::new(linux_clipboard, 1000); // 1s sync interval

        let handle = tokio::spawn(async move {
            info!("Clipboard sync thread started for {}", env_id);

            loop {
                if let Err(e) = bridge.sync_once().await {
                    debug!("Clipboard sync error: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        });

        let mut syncs = self.active_syncs.write().await;
        syncs.insert(env_id, handle);
    }

    /// Stop clipboard sync for an environment
    pub async fn stop_sync(&self, env_id: Uuid) {
        debug!("Stopping clipboard sync for environment {}", env_id);

        let mut syncs = self.active_syncs.write().await;
        if let Some(handle) = syncs.remove(&env_id) {
            handle.abort();
            info!("Clipboard sync stopped for {}", env_id);
        }
    }

    /// Check if sync is active for an environment
    pub async fn is_active(&self, env_id: Uuid) -> bool {
        let syncs = self.active_syncs.read().await;
        syncs.contains_key(&env_id)
    }

    /// Stop all active syncs
    pub async fn stop_all(&self) {
        let mut syncs = self.active_syncs.write().await;
        for (env_id, handle) in syncs.drain() {
            handle.abort();
            debug!("Aborted clipboard sync for {}", env_id);
        }
        info!("All clipboard sync tasks stopped");
    }
}

impl Default for ClipboardSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clipboard_sync_manager_creation() {
        let manager = ClipboardSyncManager::new();
        assert!(!manager.is_active(Uuid::new_v4()).await);
    }

    #[tokio::test]
    async fn test_stop_sync_inactive() {
        let manager = ClipboardSyncManager::new();
        let env_id = Uuid::new_v4();
        manager.stop_sync(env_id).await; // Should not panic
    }
}
