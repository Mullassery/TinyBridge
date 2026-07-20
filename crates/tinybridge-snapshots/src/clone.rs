use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Result, SnapshotError};

/// Clone strategy for environments
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CloneStrategy {
    /// Copy-on-Write: shared base, write-isolated changes
    CopyOnWrite,
    /// Full copy: independent environment
    Full,
    /// Linked clone: shares with original, read-only base
    Linked,
}

impl std::fmt::Display for CloneStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloneStrategy::CopyOnWrite => write!(f, "copy-on-write"),
            CloneStrategy::Full => write!(f, "full"),
            CloneStrategy::Linked => write!(f, "linked"),
        }
    }
}

/// Clone metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneMetadata {
    /// Cloned environment ID
    pub clone_id: Uuid,

    /// Source environment ID
    pub source_env_id: Uuid,

    /// Clone name
    pub name: String,

    /// Strategy used
    pub strategy: CloneStrategy,

    /// Clone timestamp
    pub created_at: DateTime<Utc>,

    /// Size overhead for this clone (varies by strategy)
    pub size_bytes: u64,

    /// Base snapshot ID (if strategy is CoW or Linked)
    pub base_snapshot_id: Option<Uuid>,
}

impl CloneMetadata {
    /// Create new clone metadata
    pub fn new(clone_id: Uuid, source_env_id: Uuid, name: String, strategy: CloneStrategy) -> Self {
        Self {
            clone_id,
            source_env_id,
            name,
            strategy,
            created_at: Utc::now(),
            size_bytes: 0,
            base_snapshot_id: None,
        }
    }

    /// Set base snapshot (for CoW/Linked)
    pub fn with_base_snapshot(mut self, snapshot_id: Uuid) -> Self {
        self.base_snapshot_id = Some(snapshot_id);
        self
    }

    /// Set size overhead
    pub fn with_size(mut self, size_bytes: u64) -> Self {
        self.size_bytes = size_bytes;
        self
    }
}

/// Clone manager for managing environment clones
pub struct CloneManager {
    clones: std::collections::HashMap<Uuid, CloneMetadata>,
}

impl CloneManager {
    /// Create new clone manager
    pub fn new() -> Self {
        Self {
            clones: std::collections::HashMap::new(),
        }
    }

    /// Register a clone
    pub fn create_clone(&mut self, metadata: CloneMetadata) -> Result<()> {
        if self.clones.contains_key(&metadata.clone_id) {
            return Err(SnapshotError::SnapshotExists(metadata.clone_id.to_string()));
        }

        self.clones.insert(metadata.clone_id, metadata);
        Ok(())
    }

    /// Get clone metadata
    pub fn get_clone(&self, id: Uuid) -> Option<CloneMetadata> {
        self.clones.get(&id).cloned()
    }

    /// List clones of a source environment
    pub fn list_clones_of(&self, source_env_id: Uuid) -> Vec<CloneMetadata> {
        self.clones
            .values()
            .filter(|c| c.source_env_id == source_env_id)
            .cloned()
            .collect()
    }

    /// List all clones
    pub fn list_all(&self) -> Vec<CloneMetadata> {
        self.clones.values().cloned().collect()
    }

    /// Delete a clone
    pub fn delete_clone(&mut self, id: Uuid) -> Result<CloneMetadata> {
        self.clones
            .remove(&id)
            .ok_or_else(|| SnapshotError::SnapshotNotFound(id.to_string()))
    }

    /// Count clones of a source environment
    pub fn clone_count(&self, source_env_id: Uuid) -> usize {
        self.clones
            .values()
            .filter(|c| c.source_env_id == source_env_id)
            .count()
    }

    /// Total storage used by all clones of an environment
    pub fn clone_storage_for_env(&self, source_env_id: Uuid) -> u64 {
        self.clones
            .values()
            .filter(|c| c.source_env_id == source_env_id)
            .map(|c| c.size_bytes)
            .sum()
    }

    /// Get most recent clone of an environment
    pub fn latest_clone(&self, source_env_id: Uuid) -> Option<CloneMetadata> {
        self.clones
            .values()
            .filter(|c| c.source_env_id == source_env_id)
            .max_by_key(|c| c.created_at)
            .cloned()
    }
}

impl Default for CloneManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_clone() {
        let mut manager = CloneManager::new();
        let source_id = Uuid::new_v4();
        let clone_id = Uuid::new_v4();
        let metadata = CloneMetadata::new(clone_id, source_id, "clone1".to_string(), CloneStrategy::CopyOnWrite);

        manager.create_clone(metadata.clone()).unwrap();

        let retrieved = manager.get_clone(clone_id).unwrap();
        assert_eq!(retrieved.name, "clone1");
        assert_eq!(retrieved.strategy, CloneStrategy::CopyOnWrite);
    }

    #[test]
    fn test_list_clones_of() {
        let mut manager = CloneManager::new();
        let source_id = Uuid::new_v4();

        let c1 = CloneMetadata::new(Uuid::new_v4(), source_id, "c1".to_string(), CloneStrategy::Full);
        let c2 = CloneMetadata::new(Uuid::new_v4(), source_id, "c2".to_string(), CloneStrategy::CopyOnWrite);

        manager.create_clone(c1).unwrap();
        manager.create_clone(c2).unwrap();

        let clones = manager.list_clones_of(source_id);
        assert_eq!(clones.len(), 2);
    }

    #[test]
    fn test_delete_clone() {
        let mut manager = CloneManager::new();
        let clone_id = Uuid::new_v4();
        let metadata = CloneMetadata::new(clone_id, Uuid::new_v4(), "clone".to_string(), CloneStrategy::Full);

        manager.create_clone(metadata).unwrap();
        assert_eq!(manager.list_all().len(), 1);

        manager.delete_clone(clone_id).unwrap();
        assert_eq!(manager.list_all().len(), 0);
    }

    #[test]
    fn test_clone_storage_calculation() {
        let mut manager = CloneManager::new();
        let source_id = Uuid::new_v4();

        let mut c1 = CloneMetadata::new(Uuid::new_v4(), source_id, "c1".to_string(), CloneStrategy::CopyOnWrite);
        c1.size_bytes = 500;

        let mut c2 = CloneMetadata::new(Uuid::new_v4(), source_id, "c2".to_string(), CloneStrategy::Full);
        c2.size_bytes = 1500;

        manager.create_clone(c1).unwrap();
        manager.create_clone(c2).unwrap();

        assert_eq!(manager.clone_storage_for_env(source_id), 2000);
    }

    #[test]
    fn test_latest_clone() {
        let mut manager = CloneManager::new();
        let source_id = Uuid::new_v4();

        let c1 = CloneMetadata::new(Uuid::new_v4(), source_id, "c1".to_string(), CloneStrategy::Full);
        let c2 = CloneMetadata::new(Uuid::new_v4(), source_id, "c2".to_string(), CloneStrategy::CopyOnWrite);

        manager.create_clone(c1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.create_clone(c2.clone()).unwrap();

        let latest = manager.latest_clone(source_id).unwrap();
        assert_eq!(latest.name, "c2");
    }

    #[test]
    fn test_clone_strategy_display() {
        assert_eq!(CloneStrategy::CopyOnWrite.to_string(), "copy-on-write");
        assert_eq!(CloneStrategy::Full.to_string(), "full");
        assert_eq!(CloneStrategy::Linked.to_string(), "linked");
    }
}
