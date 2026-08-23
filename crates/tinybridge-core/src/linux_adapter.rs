//! Linux Platform Adapter - **UNIMPLEMENTED / PLANNED, NOT A REAL KVM/QEMU BACKEND.**
//!
//! Despite prior commit messages describing this as a "Phase 6.3 Implementation," this
//! module does not call `libvirt`, QEMU, or `/dev/kvm` in any way. Every method below
//! only mutates an in-memory `HashMap<String, LinuxVMMetadata>` - there is no real VM
//! anywhere. It is also **dead code today**: nothing outside this module and its own unit
//! tests constructs a `LinuxKVMAdapter` (verified by workspace-wide search), so it is not
//! reachable from the daemon, CLI, or any RPC path.
//!
//! It is being kept (rather than deleted) as clearly-labeled scaffolding for a genuine
//! future Linux implementation, since a real KVM/libvirt backend cannot be built or tested
//! from a macOS development sandbox - see the "Platform Support" section of the top-level
//! README for the honest, currently-supported platform (macOS only, via
//! `crates/tinybridge-vz`).

use crate::platform_abstraction::{
    NetworkMode, PlatformAdapter, PlatformInfo, StorageMount, VMResourceConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Linux VM metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxVMMetadata {
    /// VM ID (UUID)
    pub vm_id: String,
    /// VM name
    pub name: String,
    /// VM data directory (/var/lib/libvirt/qemu/{vm_id})
    pub vm_data_path: PathBuf,
    /// QEMU/KVM disk image path
    pub disk_image_path: PathBuf,
    /// CPU allocation
    pub cpu_cores: u32,
    /// Memory allocation in GB
    pub memory_gb: u32,
    /// GPU acceleration enabled
    pub gpu_enabled: bool,
    /// IOMMU support (for USB/GPU passthrough)
    pub iommu_support: bool,
    /// Network mode
    pub network_mode: String,
    /// Shared folders (via 9p)
    pub shared_folders: Vec<StorageMount>,
    /// USB devices
    pub usb_devices: Vec<String>,
    /// Running state
    pub is_running: bool,
    /// PID of QEMU process
    pub pid: Option<u32>,
}

impl LinuxVMMetadata {
    /// Create new VM metadata
    pub fn new(name: &str, config: &VMResourceConfig) -> Self {
        let vm_id = uuid::Uuid::new_v4().to_string();
        let vm_data_path = PathBuf::from(format!("/var/lib/libvirt/qemu/{}", vm_id));
        let disk_image_path = vm_data_path.join(format!("{}.qcow2", name));

        LinuxVMMetadata {
            vm_id,
            name: name.to_string(),
            vm_data_path,
            disk_image_path,
            cpu_cores: config.cpu_cores,
            memory_gb: config.memory_gb,
            gpu_enabled: config.gpu_enabled,
            iommu_support: true,             // Assume modern Linux
            network_mode: "NAT".to_string(), // safe-by-default; was "Bridged" (see SECURITY.md)
            shared_folders: Vec::new(),
            usb_devices: Vec::new(),
            is_running: false,
            pid: None,
        }
    }
}

/// Linux KVM/QEMU adapter
pub struct LinuxKVMAdapter {
    /// Platform info
    platform_info: PlatformInfo,
    /// VM registry
    vms: std::sync::Arc<std::sync::RwLock<HashMap<String, LinuxVMMetadata>>>,
}

impl LinuxKVMAdapter {
    /// Create new Linux adapter
    pub fn new(platform_info: PlatformInfo) -> Self {
        LinuxKVMAdapter {
            platform_info,
            vms: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Get VM metadata
    fn get_vm(&self, vm_id: &str) -> Result<LinuxVMMetadata, Box<dyn std::error::Error>> {
        let vms = self.vms.read().unwrap();
        vms.get(vm_id)
            .cloned()
            .ok_or_else(|| format!("VM not found: {}", vm_id).into())
    }

    /// Update VM metadata
    fn update_vm(&self, metadata: LinuxVMMetadata) -> Result<(), Box<dyn std::error::Error>> {
        let mut vms = self.vms.write().unwrap();
        vms.insert(metadata.vm_id.clone(), metadata);
        Ok(())
    }
}

impl PlatformAdapter for LinuxKVMAdapter {
    fn platform_info(&self) -> &PlatformInfo {
        &self.platform_info
    }

    fn create_vm(
        &self,
        name: &str,
        config: &VMResourceConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Validate inputs
        if config.cpu_cores == 0 || config.cpu_cores > 128 {
            return Err("CPU cores must be between 1 and 128".into());
        }
        if config.memory_gb == 0 || config.memory_gb > 2048 {
            return Err("Memory must be between 1GB and 2048GB".into());
        }

        let metadata = LinuxVMMetadata::new(name, config);
        let vm_id = metadata.vm_id.clone();

        // In a real implementation:
        // 1. Create VM data directory
        // 2. Create QCOW2 disk image via qemu-img
        // 3. Generate libvirt XML domain configuration
        // 4. Define VM via virsh or libvirt API
        eprintln!(
            "Linux: Creating KVM VM '{}' with {} CPU cores, {} GB RAM, GPU: {}",
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
        // virsh start {vm_name} or VIR_DOMAIN_START_NORMAL
        // Capture QEMU process PID
        eprintln!("Linux: Starting KVM VM '{}' via libvirt", metadata.name);

        metadata.is_running = true;
        metadata.pid = Some(12345); // Simulated PID
        self.update_vm(metadata)?;
        Ok(())
    }

    fn stop_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if !metadata.is_running {
            return Err("VM is not running".into());
        }

        // In a real implementation:
        // virsh destroy {vm_name} or VIR_DOMAIN_DESTROY
        eprintln!("Linux: Stopping KVM VM '{}'", metadata.name);

        metadata.is_running = false;
        metadata.pid = None;
        self.update_vm(metadata)?;
        Ok(())
    }

    fn suspend_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _metadata = self.get_vm(vm_id)?;

        // In a real implementation:
        // virsh suspend {vm_name} or VIR_DOMAIN_PMSUSPENDED
        eprintln!("Linux: Suspending KVM VM via libvirt");
        Ok(())
    }

    fn resume_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _metadata = self.get_vm(vm_id)?;

        // In a real implementation:
        // virsh resume {vm_name}
        eprintln!("Linux: Resuming KVM VM via libvirt");
        Ok(())
    }

    fn delete_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = self.get_vm(vm_id)?;

        if metadata.is_running {
            return Err("Cannot delete running VM. Stop it first.".into());
        }

        // In a real implementation:
        // virsh undefine {vm_name} --remove-all-storage
        eprintln!("Linux: Deleting KVM VM '{}'", metadata.name);

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
        // Modify libvirt domain XML with appropriate network interface
        eprintln!(
            "Linux: Configuring network for '{}' as {}",
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
        // Add 9p filesystem to libvirt domain XML
        // virsh edit domain to add: <filesystem type='mount'>...</filesystem>
        eprintln!(
            "Linux: Mounting '{:?}' to '{:?}' via 9p (read_only: {})",
            mount.host_path, mount.vm_path, mount.read_only
        );

        metadata.shared_folders.push(mount.clone());
        self.update_vm(metadata)?;
        Ok(())
    }

    fn enable_gpu(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if !metadata.iommu_support {
            return Err("IOMMU not available for GPU passthrough".into());
        }

        // In a real implementation:
        // Configure VFIO for GPU passthrough via libvirt
        eprintln!("Linux: Enabling GPU passthrough for '{}'", metadata.name);

        metadata.gpu_enabled = true;
        self.update_vm(metadata)?;
        Ok(())
    }

    fn clipboard_read(&self) -> Result<String, Box<dyn std::error::Error>> {
        // In a real implementation: Use D-Bus/systemd or xclip
        eprintln!("Linux: Reading clipboard via D-Bus");
        Ok("clipboard content".to_string())
    }

    fn clipboard_write(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation: Use D-Bus/systemd or xclip
        eprintln!("Linux: Writing to clipboard via D-Bus: {}", content);
        Ok(())
    }

    fn allocate_usb(&self, vm_id: &str, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if !metadata.iommu_support {
            return Err("IOMMU not available for USB passthrough".into());
        }

        // In a real implementation:
        // virsh attach-device domain usb-device.xml
        eprintln!(
            "Linux: Allocating USB device '{}' to VM '{}'",
            device_id, metadata.name
        );

        metadata.usb_devices.push(device_id.to_string());
        self.update_vm(metadata)?;
        Ok(())
    }

    fn release_usb(&self, vm_id: &str, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = self.get_vm(vm_id)?;

        if !metadata.usb_devices.contains(&device_id.to_string()) {
            return Err(format!("USB device not allocated: {}", device_id).into());
        }

        // In a real implementation:
        // virsh detach-device domain usb-device.xml
        eprintln!(
            "Linux: Releasing USB device '{}' from VM '{}'",
            device_id, metadata.name
        );

        metadata.usb_devices.retain(|d| d != device_id);
        self.update_vm(metadata)?;
        Ok(())
    }

    fn name(&self) -> &str {
        "linux-kvm"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_abstraction::PlatformCapabilities;
    use crate::platform_abstraction::{HostPlatform, HypervisorBackend};

    fn create_adapter() -> LinuxKVMAdapter {
        let platform_info = PlatformInfo {
            platform: HostPlatform::Linux,
            hypervisor: HypervisorBackend::KVM,
            capabilities: PlatformCapabilities::linux_kvm(),
            os_version: "Ubuntu 22.04".to_string(),
            arch: "x86_64".to_string(),
            available_memory_gb: 32,
            cpu_cores: 16,
        };
        LinuxKVMAdapter::new(platform_info)
    }

    #[test]
    fn test_adapter_name() {
        let adapter = create_adapter();
        assert_eq!(adapter.name(), "linux-kvm");
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
            cpu_cores: 256, // Too many
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
            cpu_cores: 8,
            memory_gb: 16,
            disk_gb: 100,
            gpu_enabled: true,
        };

        let metadata = LinuxVMMetadata::new("my-vm", &config);
        assert_eq!(metadata.name, "my-vm");
        assert_eq!(metadata.cpu_cores, 8);
        assert_eq!(metadata.memory_gb, 16);
        assert!(metadata.gpu_enabled);
        assert!(metadata.iommu_support);
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
    fn test_usb_passthrough() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: false,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();
        let result = adapter.allocate_usb(&vm_id, "usb-device-1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_usb_release() {
        let adapter = create_adapter();
        let config = VMResourceConfig {
            cpu_cores: 2,
            memory_gb: 4,
            disk_gb: 20,
            gpu_enabled: false,
        };

        let vm_id = adapter.create_vm("test-vm", &config).unwrap();
        adapter.allocate_usb(&vm_id, "usb-device-1").ok();
        let result = adapter.release_usb(&vm_id, "usb-device-1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_platform_info_access() {
        let adapter = create_adapter();
        let info = adapter.platform_info();
        assert_eq!(info.platform, HostPlatform::Linux);
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
}
