/// Platform Adapter Registry
/// Phase 6: Cross-Platform Compatibility
///
/// Dynamic registration and selection of platform adapters
use crate::platform_abstraction::{HostPlatform, PlatformAdapter, PlatformInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Platform adapter registry
///
/// Note on lock poisoning: every `RwLock` access below recovers via
/// `unwrap_or_else(|poisoned| poisoned.into_inner())` instead of `.unwrap()`. A panic
/// inside one adapter call while holding this lock (e.g. a future real adapter
/// implementation panicking on an unexpected FFI/OS error) must not permanently poison
/// the registry and take every *other* adapter/VM down with it - the `HashMap` itself is
/// never left in a torn state by any operation here (each insert/remove is a single
/// atomic map operation), so recovering a poisoned guard is safe.
pub struct PlatformRegistry {
    /// Registered adapters
    adapters: RwLock<HashMap<String, Arc<dyn PlatformAdapter>>>,
    /// Current platform info
    current_platform: PlatformInfo,
}

impl PlatformRegistry {
    /// Create new registry
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let platform_info = PlatformInfo::detect()?;

        Ok(PlatformRegistry {
            adapters: RwLock::new(HashMap::new()),
            current_platform: platform_info,
        })
    }

    /// Register an adapter
    pub fn register(&self, name: impl Into<String>, adapter: Arc<dyn PlatformAdapter>) {
        let mut adapters = self
            .adapters
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        adapters.insert(name.into(), adapter);
    }

    /// Get adapter by name
    pub fn get_adapter(&self, name: &str) -> Option<Arc<dyn PlatformAdapter>> {
        let adapters = self
            .adapters
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        adapters.get(name).cloned()
    }

    /// Get default adapter for current platform
    pub fn get_default_adapter(&self) -> Option<Arc<dyn PlatformAdapter>> {
        let adapter_name = match self.current_platform.platform {
            HostPlatform::Windows => "windows",
            HostPlatform::MacOS => "macos",
            HostPlatform::Linux => "linux",
            _ => return None,
        };
        self.get_adapter(adapter_name)
    }

    /// List all registered adapters
    pub fn list_adapters(&self) -> Vec<String> {
        let adapters = self
            .adapters
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        adapters.keys().cloned().collect()
    }

    /// Get current platform info
    pub fn platform_info(&self) -> &PlatformInfo {
        &self.current_platform
    }

    /// Check if adapter is registered
    pub fn has_adapter(&self, name: &str) -> bool {
        let adapters = self
            .adapters
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        adapters.contains_key(name)
    }

    /// Get adapter count
    pub fn adapter_count(&self) -> usize {
        let adapters = self
            .adapters
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        adapters.len()
    }
}

impl Default for PlatformRegistry {
    fn default() -> Self {
        // Create with default platform detection
        // If detection fails, create with minimal config
        PlatformRegistry::new().unwrap_or_else(|_| PlatformRegistry {
            adapters: RwLock::new(HashMap::new()),
            current_platform: PlatformInfo {
                platform: HostPlatform::Linux,
                hypervisor: crate::platform_abstraction::HypervisorBackend::QEMU,
                capabilities: crate::platform_abstraction::PlatformCapabilities::linux_kvm(),
                os_version: "Unknown".to_string(),
                arch: "unknown".to_string(),
                available_memory_gb: 4,
                cpu_cores: 2,
            },
        })
    }
}

/// Adapter registration summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatus {
    /// Current platform
    pub platform: String,
    /// Number of adapters registered
    pub adapter_count: usize,
    /// Adapter names
    pub adapter_names: Vec<String>,
    /// Has default adapter
    pub has_default: bool,
}

impl PlatformRegistry {
    /// Get registry status
    pub fn status(&self) -> RegistryStatus {
        let adapters = self
            .adapters
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let adapter_names: Vec<String> = adapters.keys().cloned().collect();

        let default_adapter_name = match self.current_platform.platform {
            HostPlatform::Windows => "windows",
            HostPlatform::MacOS => "macos",
            HostPlatform::Linux => "linux",
            _ => "",
        };

        RegistryStatus {
            platform: self.current_platform.platform.to_string(),
            adapter_count: adapters.len(),
            adapter_names,
            has_default: adapters.contains_key(default_adapter_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock adapter for testing
    struct MockAdapter {
        name: String,
    }

    impl MockAdapter {
        fn new(name: &str) -> Self {
            MockAdapter {
                name: name.to_string(),
            }
        }
    }

    impl PlatformAdapter for MockAdapter {
        fn platform_info(&self) -> &PlatformInfo {
            panic!("Not implemented for mock")
        }

        fn create_vm(
            &self,
            _name: &str,
            _config: &crate::platform_abstraction::VMResourceConfig,
        ) -> Result<String, Box<dyn std::error::Error>> {
            Ok("mock-vm-id".to_string())
        }

        fn start_vm(&self, _vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn stop_vm(&self, _vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn suspend_vm(&self, _vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn resume_vm(&self, _vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn delete_vm(&self, _vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn configure_network(
            &self,
            _vm_id: &str,
            _mode: &crate::platform_abstraction::NetworkMode,
        ) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn mount_storage(
            &self,
            _vm_id: &str,
            _mount: &crate::platform_abstraction::StorageMount,
        ) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn enable_gpu(&self, _vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn clipboard_read(&self) -> Result<String, Box<dyn std::error::Error>> {
            Ok("test content".to_string())
        }

        fn clipboard_write(&self, _content: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn allocate_usb(
            &self,
            _vm_id: &str,
            _device_id: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn release_usb(
            &self,
            _vm_id: &str,
            _device_id: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_platform_registry_creation() {
        let registry = PlatformRegistry::new();
        assert!(registry.is_ok());
    }

    #[test]
    fn test_platform_registry_register() {
        let registry = PlatformRegistry::new().unwrap();
        let adapter = Arc::new(MockAdapter::new("test"));
        registry.register("test", adapter);
        assert!(registry.has_adapter("test"));
    }

    #[test]
    fn test_platform_registry_get_adapter() {
        let registry = PlatformRegistry::new().unwrap();
        let adapter = Arc::new(MockAdapter::new("test"));
        registry.register("test", adapter.clone());

        let retrieved = registry.get_adapter("test");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_platform_registry_list_adapters() {
        let registry = PlatformRegistry::new().unwrap();
        registry.register("adapter1", Arc::new(MockAdapter::new("adapter1")));
        registry.register("adapter2", Arc::new(MockAdapter::new("adapter2")));

        let adapters = registry.list_adapters();
        assert_eq!(adapters.len(), 2);
        assert!(adapters.contains(&"adapter1".to_string()));
        assert!(adapters.contains(&"adapter2".to_string()));
    }

    #[test]
    fn test_platform_registry_adapter_count() {
        let registry = PlatformRegistry::new().unwrap();
        registry.register("adapter1", Arc::new(MockAdapter::new("adapter1")));
        registry.register("adapter2", Arc::new(MockAdapter::new("adapter2")));

        assert_eq!(registry.adapter_count(), 2);
    }

    #[test]
    fn test_platform_registry_status() {
        let registry = PlatformRegistry::new().unwrap();
        registry.register("test", Arc::new(MockAdapter::new("test")));

        let status = registry.status();
        assert_eq!(status.adapter_count, 1);
        assert!(status.adapter_names.contains(&"test".to_string()));
    }

    #[test]
    fn test_platform_registry_default() {
        let registry = PlatformRegistry::default();
        assert_eq!(registry.adapter_count(), 0);
    }

    #[test]
    fn test_platform_info_access() {
        let registry = PlatformRegistry::new().unwrap();
        let info = registry.platform_info();
        assert!(!info.os_version.is_empty());
    }

    /// Before the fix, `register`/`get_adapter`/`has_adapter`/`adapter_count`/`status` all
    /// called `.read().unwrap()` / `.write().unwrap()` directly, which panics with
    /// "PoisonError" if any holder of the lock ever panicked while holding it - one
    /// adapter's bug would then permanently wedge every other adapter behind a poisoned
    /// lock. This test proves the registry recovers instead.
    #[test]
    fn test_survives_poisoned_lock() {
        let registry = Arc::new(PlatformRegistry::new().unwrap());
        registry.register("test", Arc::new(MockAdapter::new("test")));

        // Poison the internal RwLock by panicking on another thread while holding a write
        // guard on it directly - simulating what happens if a real adapter implementation
        // panics mid-mutation.
        let registry_for_thread = Arc::clone(&registry);
        let join_result = std::thread::spawn(move || {
            let _guard = registry_for_thread.adapters.write().unwrap();
            panic!("simulated panic while holding the write lock");
        })
        .join();
        assert!(
            join_result.is_err(),
            "background thread should have panicked, poisoning the lock"
        );

        // Every call below would itself panic here (propagating the poison) before the fix.
        assert!(
            registry.has_adapter("test"),
            "registry must recover from a poisoned lock instead of panicking"
        );
        assert_eq!(registry.adapter_count(), 1);
        assert!(registry.get_adapter("test").is_some());
        assert_eq!(registry.list_adapters(), vec!["test".to_string()]);
        let status = registry.status();
        assert_eq!(status.adapter_count, 1);

        // The registry must still be fully usable afterwards, not just readable.
        registry.register("second", Arc::new(MockAdapter::new("second")));
        assert_eq!(registry.adapter_count(), 2);
    }
}
