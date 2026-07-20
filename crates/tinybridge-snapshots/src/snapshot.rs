use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Result, SnapshotError};

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Snapshot ID
    pub id: Uuid,

    /// Source environment ID
    pub env_id: Uuid,

    /// Human-readable name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Size in bytes
    pub size_bytes: u64,

    /// Checksum (SHA256)
    pub checksum: String,

    /// Whether this is a read-only snapshot
    pub read_only: bool,

    /// Parent snapshot ID (for incremental snapshots)
    pub parent_id: Option<Uuid>,

    /// Retention policy ("keep" or days from creation)
    pub retention: SnapshotRetention,
}

/// Snapshot retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotRetention {
    /// Keep indefinitely
    Keep,
    /// Delete after N days
    Days(u32),
}

impl SnapshotMetadata {
    /// Create new snapshot metadata
    pub fn new(env_id: Uuid, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            env_id,
            name,
            description: None,
            created_at: Utc::now(),
            size_bytes: 0,
            checksum: String::new(),
            read_only: false,
            parent_id: None,
            retention: SnapshotRetention::Keep,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    /// Mark as read-only
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Set parent snapshot
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set retention policy
    pub fn with_retention(mut self, retention: SnapshotRetention) -> Self {
        self.retention = retention;
        self
    }

    /// Check if snapshot is expired (for retention policies)
    pub fn is_expired(&self) -> bool {
        match &self.retention {
            SnapshotRetention::Keep => false,
            SnapshotRetention::Days(days) => {
                let age = (Utc::now() - self.created_at).num_days();
                age > *days as i64
            }
        }
    }
}

/// Snapshot manager for lifecycle operations
pub struct SnapshotManager {
    snapshots: std::collections::HashMap<Uuid, SnapshotMetadata>,
}

impl SnapshotManager {
    /// Create new snapshot manager
    pub fn new() -> Self {
        Self {
            snapshots: std::collections::HashMap::new(),
        }
    }

    /// Create a snapshot
    pub fn create_snapshot(&mut self, metadata: SnapshotMetadata) -> Result<()> {
        if self.snapshots.contains_key(&metadata.id) {
            return Err(SnapshotError::SnapshotExists(metadata.id.to_string()));
        }

        self.snapshots.insert(metadata.id, metadata);
        Ok(())
    }

    /// Get snapshot metadata
    pub fn get_snapshot(&self, id: Uuid) -> Option<SnapshotMetadata> {
        self.snapshots.get(&id).cloned()
    }

    /// List snapshots for an environment
    pub fn list_for_env(&self, env_id: Uuid) -> Vec<SnapshotMetadata> {
        self.snapshots
            .values()
            .filter(|s| s.env_id == env_id)
            .cloned()
            .collect()
    }

    /// List all snapshots
    pub fn list_all(&self) -> Vec<SnapshotMetadata> {
        self.snapshots.values().cloned().collect()
    }

    /// Delete a snapshot
    pub fn delete_snapshot(&mut self, id: Uuid) -> Result<SnapshotMetadata> {
        self.snapshots
            .remove(&id)
            .ok_or_else(|| SnapshotError::SnapshotNotFound(id.to_string()))
    }

    /// Update snapshot metadata
    pub fn update_snapshot(&mut self, metadata: SnapshotMetadata) -> Result<()> {
        if !self.snapshots.contains_key(&metadata.id) {
            return Err(SnapshotError::SnapshotNotFound(metadata.id.to_string()));
        }

        self.snapshots.insert(metadata.id, metadata);
        Ok(())
    }

    /// Delete expired snapshots
    pub fn cleanup_expired(&mut self) -> Vec<SnapshotMetadata> {
        let expired: Vec<_> = self
            .snapshots
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| *id)
            .collect();

        let mut deleted = Vec::new();
        for id in expired {
            if let Some(snapshot) = self.snapshots.remove(&id) {
                deleted.push(snapshot);
            }
        }

        deleted
    }

    /// Get total storage used by snapshots for an environment
    pub fn storage_for_env(&self, env_id: Uuid) -> u64 {
        self.snapshots
            .values()
            .filter(|s| s.env_id == env_id)
            .map(|s| s.size_bytes)
            .sum()
    }

    /// Count snapshots for an environment
    pub fn count_for_env(&self, env_id: Uuid) -> usize {
        self.snapshots
            .values()
            .filter(|s| s.env_id == env_id)
            .count()
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_snapshot() {
        let mut manager = SnapshotManager::new();
        let env_id = Uuid::new_v4();
        let metadata = SnapshotMetadata::new(env_id, "snapshot1".to_string());

        manager.create_snapshot(metadata.clone()).unwrap();

        let retrieved = manager.get_snapshot(metadata.id).unwrap();
        assert_eq!(retrieved.name, "snapshot1");
        assert_eq!(retrieved.env_id, env_id);
    }

    #[test]
    fn test_list_for_env() {
        let mut manager = SnapshotManager::new();
        let env_id = Uuid::new_v4();

        let s1 = SnapshotMetadata::new(env_id, "s1".to_string());
        let s2 = SnapshotMetadata::new(env_id, "s2".to_string());

        manager.create_snapshot(s1).unwrap();
        manager.create_snapshot(s2).unwrap();

        let list = manager.list_for_env(env_id);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_delete_snapshot() {
        let mut manager = SnapshotManager::new();
        let env_id = Uuid::new_v4();
        let metadata = SnapshotMetadata::new(env_id, "snapshot1".to_string());
        let id = metadata.id;

        manager.create_snapshot(metadata).unwrap();
        assert_eq!(manager.list_all().len(), 1);

        manager.delete_snapshot(id).unwrap();
        assert_eq!(manager.list_all().len(), 0);
    }

    #[test]
    fn test_storage_calculation() {
        let mut manager = SnapshotManager::new();
        let env_id = Uuid::new_v4();

        let mut s1 = SnapshotMetadata::new(env_id, "s1".to_string());
        s1.size_bytes = 1000;

        let mut s2 = SnapshotMetadata::new(env_id, "s2".to_string());
        s2.size_bytes = 2000;

        manager.create_snapshot(s1).unwrap();
        manager.create_snapshot(s2).unwrap();

        assert_eq!(manager.storage_for_env(env_id), 3000);
    }

    #[test]
    fn test_retention_policy() {
        let metadata = SnapshotMetadata::new(Uuid::new_v4(), "snap".to_string())
            .with_retention(SnapshotRetention::Days(1));

        assert!(!metadata.is_expired());

        let old = SnapshotMetadata {
            created_at: Utc::now() - chrono::Duration::days(2),
            ..metadata
        };

        assert!(old.is_expired());
    }

    #[test]
    fn test_duplicate_snapshot_fails() {
        let mut manager = SnapshotManager::new();
        let metadata = SnapshotMetadata::new(Uuid::new_v4(), "snap".to_string());

        manager.create_snapshot(metadata.clone()).unwrap();
        let result = manager.create_snapshot(metadata);

        assert!(result.is_err());
    }
}
