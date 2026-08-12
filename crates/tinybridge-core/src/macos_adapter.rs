//! macOS Platform Adapter - **NOT the real macOS backend, and not wired to anything.**
//!
//! Despite prior commit messages describing this as a "Phase 6.2 Implementation," this
//! module does not call Virtualization.framework or any other macOS API. Every method
//! below only mutates an in-memory `HashMap<String, MacOSVMMetadata>` - there is no real
//! VM anywhere. It is also **dead code today**: nothing outside this module and its own
//! unit tests constructs a `MacOSAppleAdapter` (verified by workspace-wide search), so it
//! is not reachable from the daemon, CLI, or any RPC path.
//!
//! **The real, working macOS Virtualization.framework integration lives in
//! `crates/tinybridge-vz` (`VirtualMachine`, backed by real FFI into
//! `swift/Sources/TinyBridgeVZBridge`) and is driven by `crates/tinybridge-vmhost`'s
//! `VmController` - not by this module.** This file is kept only as clearly-labeled
//! scaffolding matching the (also unimplemented) Windows/Linux adapters in this same
//! directory, for a possible future unification of the platform-abstraction trait with
//! the real backend; today it should be treated as dead code, not a second macOS backend.

use crate::platform_abstraction::{
    NetworkMode, PlatformAdapter, PlatformCapabilities, PlatformInfo, StorageMount, VMResourceConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// macOS VM metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacOSVMMetadata {
    /// VM ID (UUID)
    pub vm_id: String,
    /// VM name
    pub name: String,
    /// VM bundle path (~/.tinybridge/vms/{vm_id})
    pub vm_bundle_path: PathBuf,
    /// Root disk image path
    pub disk_image_path: PathBuf,
    /// CPU allocation
    pub cpu_cores: u32,
    /// Memory allocation in GB
    pub memory_gb: u32,
    /// GPU acceleration enabled
    pub gpu_enabled: bool,
    /// Metal graphics support
    pub metal_support: bool,
    /// Network mode
    pub network_mode: String,
    /// Shared directories
    pub shared_folders: Vec<StorageMount>,
    /// Running state
    pub is_running: bool,
    /// Suspended state
    pub is_suspended: bool,
}

impl MacOSVMMetadata {
    /// Create new VM metadata
    pub fn new(name: &str, config: &VMResourceConfig) -> Self {
        let vm_id = uuid::Uuid::new_v4().to_string();
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let vm_bundle_path = home_dir.join(".tinybridge/vms").join(&vm_id);
        let disk_image_path = vm_bundle_path.join(format!("{}.img", name));

        MacOSVMMetadata {
            vm_id,
            name: name.to_string(),
            vm_bundle_path,
            disk_image_path,
            cpu_cores: config.cpu_cores,
            memory_gb: config.memory_gb,
            gpu_enabled: config.gpu_enabled,
            metal_support: true, // Apple Silicon default
            network_mode: "NAT".to_string(), // safe-by-default; was "Bridged" (see SECURITY.md)
            shared_folders: Vec::new(),
            is_running: false,
            is_suspended: false,
        }
    }
}

/// macOS Apple Virtualization adapter
pub struct MacOSAppleAdapter {
    /// Platform info
    platform_info: PlatformInfo,
    /// VM registry
    vms: std::sync::Arc<std::sync::RwLock<HashMap<String, MacOSVMMetadata>>>,
}

impl MacOSAppleAdapter {
    /// Create new macOS adapter
    pub fn new(platform_info: PlatformInfo) -> Self {
        MacOSAppleAdapter {
            platform_info,
            vms: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Get VM metadata
    fn get_vm(&self, vm_id: &str) -> Result<MacOSVMMetadata, Box<dyn std::error::Error>> {
        let vms = self.vms.read().unwrap();
        vms.get(vm_id)
            .cloned()
            .ok_or_else(|| format!("VM not found: {}", vm_id).into())
    }

    /// Update VM metadata
    fn update_vm(&self, metadata: MacOSVMMetadata) -> Result<(), Box<dyn std::error::Error>> {
        let mut vms = self.vms.write().unwrap();
        vms.insert(metadata.vm_id.clone(), metadata);
        Ok(())
    }
}

impl PlatformAdapter for MacOSAppleAdapter {
    fn platform_info(&self) -> &PlatformInfo {
        &self.platform_info
    }

    fn create_vm(
        &self,
        name: &str,
        config: &VMResourceConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Validate inputs
        if config.cpu_cores == 0 || config.cpu_cores > 16 {
            return Err("CPU cores must be between 1 and 16".into());
        }
        if config.memory_gb == 0 || config.memory_gb > 64 {
            return Err("Memory must be between 1GB and 64GB".into());
        }

        let metadata = MacOSVMMetadata::new(name, config);
        let vm_id = metadata.vm_id.clone();

        // Create VM bundle directory
        std::fs::create_dir_all(&metadata.vm_bundle_path)
            .map_err(|e| format!("Failed to create VM bundle: {}", e))?;

        // In a real implementation, this would:
        // 1. Use VZVirtualMachine APIs
        // 2. Create disk image via hdiutil
        // 3. Configure CPU/memory/GPU via VZVirtualMachineConfiguration
        eprintln!(
            "macOS: Creating Apple Virtualization VM '{}' with {} CPU cores, {} GB RAM, GPU: {}",
            name, config.cpu_cores, config.memory_gb, config.gpu_enabled
        );

        self.update_vm(metadata)?;
        Ok(vm_id)
    }

    fn start_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if metadata.is_running {
            return Err("VM is already running".into());
        }

        // In a real implementation:
        // VZVirtualMachine.start() with completion handler
        eprintln!("macOS: Starting Apple Virtualization VM '{}'", metadata.name);

        metadata.is_running = true;
        metadata.is_suspended = false;
        self.update_vm(metadata)?;
        Ok(())
    }

    fn stop_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if !metadata.is_running && !metadata.is_suspended {
            return Err("VM is not running".into());
        }

        // In a real implementation: VZVirtualMachine.stop()
        eprintln!("macOS: Stopping Apple Virtualization VM '{}'", metadata.name);

        metadata.is_running = false;
        metadata.is_suspended = false;
        self.update_vm(metadata)?;
        Ok(())
    }

    fn suspend_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if !metadata.is_running {
            return Err("VM is not running".into());
        }

        // In a real implementation: VZVirtualMachine.pause()
        eprintln!("macOS: Suspending Apple Virtualization VM '{}'", metadata.name);

        metadata.is_running = false;
        metadata.is_suspended = true;
        self.update_vm(metadata)?;
        Ok(())
    }

    fn resume_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if !metadata.is_suspended {
            return Err("VM is not suspended".into());
        }

        // In a real implementation: VZVirtualMachine.resume()
        eprintln!("macOS: Resuming Apple Virtualization VM '{}'", metadata.name);

        metadata.is_suspended = false;
        metadata.is_running = true;
        self.update_vm(metadata)?;
        Ok(())
    }

    fn delete_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = self.get_vm(vm_id)?;

        if metadata.is_running || metadata.is_suspended {
            return Err("Cannot delete running or suspended VM. Stop it first.".into());
        }

        // Clean up VM bundle directory
        if metadata.vm_bundle_path.exists() {
            std::fs::remove_dir_all(&metadata.vm_bundle_path)
                .map_err(|e| format!("Failed to delete VM bundle: {}", e))?;
        }

        eprintln!("macOS: Deleting Apple Virtualization VM '{}'", metadata.name);

        let mut vms = self.vms.write().unwrap();
        vms.remove(vm_id);
        Ok(())
    }

    fn configure_network(
        &self,
        vm_id: &str,
        mode: &NetworkMode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        let mode_name = match mode {
            NetworkMode::NAT => "NAT",
            NetworkMode::Bridged => "Bridged",
            NetworkMode::HostOnly => "Internal",
            NetworkMode::Custom(_) => "Custom",
        };

        // In a real implementation:
        // Configure VZNetworkDeviceAttachment based on mode
        eprintln!(
            "macOS: Configuring network for '{}' as {}",
            metadata.name, mode_name
        );

        metadata.network_mode = mode_name.to_string();
        self.update_vm(metadata)?;
        Ok(())
    }

    fn mount_storage(
        &self,
        vm_id: &str,
        mount: &StorageMount,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        // Validate paths exist
        if !mount.host_path.exists() {
            return Err(format!("Host path does not exist: {:?}", mount.host_path).into());
        }

        // In a real implementation:
        // Use VZSharedDirectory and VZDirectoryShare APIs
        eprintln!(
            "macOS: Mounting '{:?}' to '{:?}' (read_only: {})",
            mount.host_path, mount.vm_path, mount.read_only
        );

        metadata.shared_folders.push(mount.clone());
        self.update_vm(metadata)?;
        Ok(())
    }

    fn enable_gpu(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if !metadata.metal_support {
            return Err("Metal graphics not supported on this machine".into());
        }

        // In a real implementation:
        // Configure VZMacGraphicsDeviceConfiguration with Metal support
        eprintln!("macOS: Enabling GPU acceleration with Metal for '{}'", metadata.name);

        metadata.gpu_enabled = true;
        self.update_vm(metadata)?;
        Ok(())
    }

    fn clipboard_read(&self) -> Result<String, Box<dyn std::error::Error>> {
        // In a real implementation: Use NSPasteboard
        eprintln!("macOS: Reading clipboard via NSPasteboard");
        Ok("clipboard content".to_string())
    }

    fn clipboard_write(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation: Use NSPasteboard
        eprintln!("macOS: Writing to clipboard via NSPasteboard: {}", content);
        Ok(())
    }

    fn allocate_usb(
        &self,
        vm_id: &str,
        device_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _metadata = self.get_vm(vm_id)?;

        // USB passthrough not supported on macOS virtualization
        Err(format!(
            "USB passthrough not available on macOS (device: {})",
            device_id
        )
        .into())
    }

    fn release_usb(
        &self,
        vm_id: &str,
        device_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _metadata = self.get_vm(vm_id)?;
        Err(format!(
            "USB passthrough not available on macOS (device: {})",
            device_id
        )
        .into())
    }

    fn name(&self) -> &str {
        "macos-apple"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_abstraction::{HypervisorBackend, HostPlatform};

    fn create_adapter() -> MacOSAppleAdapter {
        let platform_info = PlatformInfo {
            platform: HostPlatform::MacOS,
            hypervisor: HypervisorBackend::AppleVirtualization,
            capabilities: PlatformCapabilities::macos_apple(),
            os_version: "macOS Sonoma".to_string(),
            arch: "aarch64".to_string(),
            available_memory_gb: 16,
            cpu_cores: 10,
        };
        MacOSAppleAdapter::new(platform_info)
    }

    #[test]
    fn test_adapter_name() {
        let adapter = create_adapter();
        assert_eq!(adapter.name(), "macos-apple");
    }

    #[test]
    fn test_create_vm() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 4,
            memory_gb: 8,
            disk_gb: 40,
            gpu_enabled: true,
        };

        let result = adapter.create_vm("test-vm", &config);
        assert!(result.is_ok());

        let vm_id = result.unwrap();
        assert!(!vm_id.is_empty());
    }

    #[test]
    fn test_create_vm_invalid_memory() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 4,
            memory_gb: 128, // Too much
            disk_gb: 40,
            gpu_enabled: false,
        };

        let result = adapter.create_vm("test-vm", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_start_stop_vm() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: false,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();
        assert!(adapter.start_vm(&vm_id).is_ok());
        assert!(adapter.stop_vm(&vm_id).is_ok());
    }

    #[test]
    fn test_suspend_resume_vm() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: false,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();
        adapter.start_vm(&vm_id).ok();
        assert!(adapter.suspend_vm(&vm_id).is_ok());
        assert!(adapter.resume_vm(&vm_id).is_ok());
    }

    #[test]
    fn test_vm_metadata() {
        let config = VMResourceConfig {
            cpu_cores: 4,
            memory_gb: 8,
            disk_gb: 40,
            gpu_enabled: true,
        };

        let metadata = MacOSVMMetadata::new("my-vm", &config);
        assert_eq!(metadata.name, "my-vm");
        assert_eq!(metadata.cpu_cores, 4);
        assert_eq!(metadata.memory_gb, 8);
        assert!(metadata.gpu_enabled);
        assert!(metadata.metal_support);
    }

    #[test]
    fn test_enable_gpu() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: false,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();
        let result = adapter.enable_gpu(&vm_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clipboard_operations() {
        let adapter = create_adapter();

        let read_result = adapter.clipboard_read();
        assert!(read_result.is_ok());

        let write_result = adapter.clipboard_write("test content");
        assert!(write_result.is_ok());
    }

    #[test]
    fn test_usb_not_supported() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: false,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();
        let result = adapter.allocate_usb(&vm_id, "usb-device-1");
        assert!(result.is_err());
    }

    #[test]
    fn test_platform_info_access() {
        let adapter = create_adapter();
        let info = adapter.platform_info();
        assert_eq!(info.platform, HostPlatform::MacOS);
    }

    #[test]
    fn test_configure_network() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: false,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();
        let result = adapter.configure_network(&vm_id, &NetworkMode::Bridged);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mount_storage() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: false,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();

        let temp_dir = std::env::temp_dir();
        let mount = StorageMount {
            host_path: temp_dir.clone(),
            vm_path: PathBuf::from("/mnt/shared"),
            read_only: true,
        };

        let result = adapter.mount_storage(&vm_id, &mount);
        assert!(result.is_ok());
    }
}
