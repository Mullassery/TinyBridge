//! Windows Platform Adapter - **UNIMPLEMENTED / PLANNED, NOT A REAL HYPER-V BACKEND.**
//!
//! Despite prior commit messages describing this as a "Phase 6.1 Implementation," this
//! module does not call any Hyper-V / Windows Hypervisor Platform (WHPX) API. Every method
//! below only mutates an in-memory `HashMap<String, WindowsVMMetadata>` - there is no real
//! VM anywhere. It is also **dead code today**: nothing outside this module and its own
//! unit tests constructs a `WindowsHyperVAdapter` (verified by workspace-wide search), so
//! it is not reachable from the daemon, CLI, or any RPC path.
//!
//! It is being kept (rather than deleted) as clearly-labeled scaffolding for a genuine
//! future Windows implementation, since a real Hyper-V/WHPX backend cannot be built or
//! tested from a macOS development sandbox (no Windows host with virtualization available)
//! - see the "Platform Support" section of the top-level README for the honest,
//! currently-supported platform (macOS only, via `crates/tinybridge-vz`).

use crate::platform_abstraction::{
    NetworkMode, PlatformAdapter, PlatformCapabilities, PlatformInfo, StorageMount, VMResourceConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Windows VM metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsVMMetadata {
    /// VM ID
    pub vm_id: String,
    /// VM name
    pub name: String,
    /// Hyper-V VM path
    pub hyperv_path: PathBuf,
    /// Virtual disk path
    pub vhd_path: PathBuf,
    /// CPU allocation
    pub cpu_cores: u32,
    /// Memory allocation in GB
    pub memory_gb: u32,
    /// GPU enabled
    pub gpu_enabled: bool,
    /// Network mode
    pub network_mode: String,
    /// Mounted shared folders
    pub shared_folders: Vec<StorageMount>,
    /// VM running state
    pub is_running: bool,
}

impl WindowsVMMetadata {
    /// Create new VM metadata
    pub fn new(name: &str, config: &VMResourceConfig) -> Self {
        let vm_id = uuid::Uuid::new_v4().to_string();
        let hyperv_path = PathBuf::from(format!("C:\\ProgramData\\Microsoft\\Windows\\Hyper-V\\{}", vm_id));
        let vhd_path = hyperv_path.join(format!("{}.vhdx", name));

        WindowsVMMetadata {
            vm_id,
            name: name.to_string(),
            hyperv_path,
            vhd_path,
            cpu_cores: config.cpu_cores,
            memory_gb: config.memory_gb,
            gpu_enabled: config.gpu_enabled,
            network_mode: "NAT".to_string(),
            shared_folders: Vec::new(),
            is_running: false,
        }
    }
}

/// Windows Hyper-V adapter
pub struct WindowsHyperVAdapter {
    /// Platform info
    platform_info: PlatformInfo,
    /// VM registry
    vms: std::sync::Arc<std::sync::RwLock<HashMap<String, WindowsVMMetadata>>>,
}

impl WindowsHyperVAdapter {
    /// Create new Windows adapter
    pub fn new(platform_info: PlatformInfo) -> Self {
        WindowsHyperVAdapter {
            platform_info,
            vms: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Get VM metadata
    fn get_vm(&self, vm_id: &str) -> Result<WindowsVMMetadata, Box<dyn std::error::Error>> {
        let vms = self.vms.read().unwrap();
        vms.get(vm_id)
            .cloned()
            .ok_or_else(|| format!("VM not found: {}", vm_id).into())
    }

    /// Update VM metadata
    fn update_vm(&self, metadata: WindowsVMMetadata) -> Result<(), Box<dyn std::error::Error>> {
        let mut vms = self.vms.write().unwrap();
        vms.insert(metadata.vm_id.clone(), metadata);
        Ok(())
    }
}

impl PlatformAdapter for WindowsHyperVAdapter {
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

        let metadata = WindowsVMMetadata::new(name, config);
        let vm_id = metadata.vm_id.clone();

        // In a real implementation, this would call Hyper-V APIs via PowerShell or WinRM
        // For now, we simulate the operation
        eprintln!("Windows: Creating Hyper-V VM '{}' with {} CPU cores, {} GB RAM",
                  name, config.cpu_cores, config.memory_gb);

        self.update_vm(metadata)?;
        Ok(vm_id)
    }

    fn start_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if metadata.is_running {
            return Err("VM is already running".into());
        }

        // In a real implementation: Invoke-Item -Path "$vmPath\*.vmcx"
        eprintln!("Windows: Starting Hyper-V VM '{}'", metadata.name);

        metadata.is_running = true;
        self.update_vm(metadata)?;
        Ok(())
    }

    fn stop_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if !metadata.is_running {
            return Err("VM is not running".into());
        }

        // In a real implementation: Stop-VM -Name $vmName -Force
        eprintln!("Windows: Stopping Hyper-V VM '{}'", metadata.name);

        metadata.is_running = false;
        self.update_vm(metadata)?;
        Ok(())
    }

    fn suspend_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = self.get_vm(vm_id)?;

        if !metadata.is_running {
            return Err("VM is not running".into());
        }

        // In a real implementation: Suspend-VM -Name $vmName
        eprintln!("Windows: Suspending Hyper-V VM '{}'", metadata.name);
        Ok(())
    }

    fn resume_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = self.get_vm(vm_id)?;

        if metadata.is_running {
            return Err("VM is already running".into());
        }

        // In a real implementation: Resume-VM -Name $vmName
        eprintln!("Windows: Resuming Hyper-V VM '{}'", metadata.name);
        Ok(())
    }

    fn delete_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = self.get_vm(vm_id)?;

        if metadata.is_running {
            return Err("Cannot delete running VM. Stop it first.".into());
        }

        // In a real implementation: Remove-VM -Name $vmName -Force
        eprintln!("Windows: Deleting Hyper-V VM '{}'", metadata.name);

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

        // In a real implementation: Add-VMNetworkAdapter -VMName $vmName -SwitchName $switchName
        eprintln!("Windows: Configuring network for '{}' as {}", metadata.name, mode_name);

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

        // In a real implementation: Add-VMHardDiskDrive or Add-VMSharedFolder
        eprintln!(
            "Windows: Mounting '{:?}' to '{:?}' (read_only: {})",
            mount.host_path, mount.vm_path, mount.read_only
        );

        metadata.shared_folders.push(mount.clone());
        self.update_vm(metadata)?;
        Ok(())
    }

    fn enable_gpu(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = self.get_vm(vm_id)?;

        // GPU acceleration not supported on Hyper-V for Linux VMs
        Err("GPU acceleration not available on Windows Hyper-V".into())
    }

    fn clipboard_read(&self) -> Result<String, Box<dyn std::error::Error>> {
        // In a real implementation: Use Windows clipboard APIs
        eprintln!("Windows: Reading clipboard");
        Ok("clipboard content".to_string())
    }

    fn clipboard_write(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation: Use Windows clipboard APIs
        eprintln!("Windows: Writing to clipboard: {}", content);
        Ok(())
    }

    fn allocate_usb(
        &self,
        vm_id: &str,
        device_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _metadata = self.get_vm(vm_id)?;

        // USB passthrough not supported on Hyper-V
        Err(format!(
            "USB passthrough not available on Windows Hyper-V (device: {})",
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
            "USB passthrough not available on Windows Hyper-V (device: {})",
            device_id
        )
        .into())
    }

    fn name(&self) -> &str {
        "windows-hyperv"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_abstraction::{HypervisorBackend, HostPlatform};

    fn create_adapter() -> WindowsHyperVAdapter {
        let platform_info = PlatformInfo {
            platform: HostPlatform::Windows,
            hypervisor: HypervisorBackend::HyperV,
            capabilities: PlatformCapabilities::windows_hyperv(),
            os_version: "Windows 11".to_string(),
            arch: "x86_64".to_string(),
            available_memory_gb: 16,
            cpu_cores: 8,
        };
        WindowsHyperVAdapter::new(platform_info)
    }

    #[test]
    fn test_adapter_name() {
        let adapter = create_adapter();
        assert_eq!(adapter.name(), "windows-hyperv");
    }

    #[test]
    fn test_create_vm() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 4,
            memory_gb: 8,
            disk_gb: 40,
            gpu_enabled: false,
        };

        let result = adapter.create_vm("test-vm", &config);
        assert!(result.is_ok());

        let vm_id = result.unwrap();
        assert!(!vm_id.is_empty());
    }

    #[test]
    fn test_create_vm_invalid_cpu() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 32, // Too many
            memory_gb: 8,
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
    fn test_vm_metadata() {
        let config = VMResourceConfig {
            cpu_cores: 4,
            memory_gb: 8,
            disk_gb: 40,
            gpu_enabled: true,
        };

        let metadata = WindowsVMMetadata::new("my-vm", &config);
        assert_eq!(metadata.name, "my-vm");
        assert_eq!(metadata.cpu_cores, 4);
        assert_eq!(metadata.memory_gb, 8);
        assert!(metadata.gpu_enabled);
        assert!(!metadata.is_running);
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
        let result = adapter.configure_network(&vm_id, &NetworkMode::NAT);
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

        // Create a temporary directory for testing
        let temp_dir = std::env::temp_dir();
        let mount = StorageMount {
            host_path: temp_dir.clone(),
            vm_path: PathBuf::from("/mnt/shared"),
            read_only: false,
        };

        let result = adapter.mount_storage(&vm_id, &mount);
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
    fn test_gpu_not_supported() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: true,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();
        let result = adapter.enable_gpu(&vm_id);
        assert!(result.is_err());
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
    fn test_delete_running_vm() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: false,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();
        adapter.start_vm(&vm_id).ok();

        let result = adapter.delete_vm(&vm_id);
        assert!(result.is_err()); // Can't delete running VM
    }

    #[test]
    fn test_platform_info_access() {
        let adapter = create_adapter();
        let info = adapter.platform_info();
        assert_eq!(info.platform, HostPlatform::Windows);
    }
}
