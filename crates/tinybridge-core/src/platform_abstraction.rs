/// Platform Abstraction Layer
/// Phase 6: Cross-Platform Compatibility
///
/// Common interface for Windows, macOS, Linux, and future platforms
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Supported host platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostPlatform {
    /// Windows (Hyper-V, WHPX)
    Windows,
    /// macOS (Apple Virtualization Framework)
    MacOS,
    /// Linux (KVM/QEMU)
    Linux,
    /// ChromeOS
    ChromeOS,
    /// Cloud-hosted
    Cloud,
}

impl std::fmt::Display for HostPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostPlatform::Windows => write!(f, "windows"),
            HostPlatform::MacOS => write!(f, "macos"),
            HostPlatform::Linux => write!(f, "linux"),
            HostPlatform::ChromeOS => write!(f, "chromeos"),
            HostPlatform::Cloud => write!(f, "cloud"),
        }
    }
}

/// Hypervisor backend technologies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypervisorBackend {
    /// Hyper-V (Windows native)
    HyperV,
    /// Windows Hypervisor Platform (WHPX)
    WHPX,
    /// Apple Virtualization Framework
    AppleVirtualization,
    /// Hypervisor Framework (macOS)
    HypervisorFramework,
    /// KVM (Linux)
    KVM,
    /// QEMU (all platforms)
    QEMU,
    /// Xen (Linux)
    Xen,
}

impl std::fmt::Display for HypervisorBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HypervisorBackend::HyperV => write!(f, "hyper-v"),
            HypervisorBackend::WHPX => write!(f, "whpx"),
            HypervisorBackend::AppleVirtualization => write!(f, "apple-virtualization"),
            HypervisorBackend::HypervisorFramework => write!(f, "hypervisor-framework"),
            HypervisorBackend::KVM => write!(f, "kvm"),
            HypervisorBackend::QEMU => write!(f, "qemu"),
            HypervisorBackend::Xen => write!(f, "xen"),
        }
    }
}

/// Platform capability features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// VM lifecycle (create, start, suspend, resume)
    pub vm_lifecycle: bool,
    /// Snapshot support
    pub snapshots: bool,
    /// Shared folders
    pub shared_folders: bool,
    /// Clipboard integration
    pub clipboard: bool,
    /// Drag and drop
    pub drag_drop: bool,
    /// GPU acceleration
    pub gpu_acceleration: bool,
    /// USB passthrough
    pub usb_passthrough: bool,
    /// Audio support
    pub audio: bool,
    /// Microphone support
    pub microphone: bool,
    /// Camera support
    pub camera: bool,
    /// Multi-monitor support
    pub multi_monitor: bool,
    /// Network bridging
    pub network_bridge: bool,
    /// VPN passthrough
    pub vpn_passthrough: bool,
    /// Touchscreen support
    pub touchscreen: bool,
    /// Printing support
    pub printing: bool,
}

impl PlatformCapabilities {
    /// Capabilities for Windows with Hyper-V
    pub fn windows_hyperv() -> Self {
        PlatformCapabilities {
            vm_lifecycle: true,
            snapshots: true,
            shared_folders: true,
            clipboard: true,
            drag_drop: true,
            gpu_acceleration: false,
            usb_passthrough: false,
            audio: true,
            microphone: true,
            camera: false,
            multi_monitor: true,
            network_bridge: true,
            vpn_passthrough: false,
            touchscreen: false,
            printing: true,
        }
    }

    /// Capabilities for macOS with Apple Virtualization
    pub fn macos_apple() -> Self {
        PlatformCapabilities {
            vm_lifecycle: true,
            snapshots: false,
            shared_folders: true,
            clipboard: true,
            drag_drop: true,
            gpu_acceleration: true,
            usb_passthrough: false,
            audio: true,
            microphone: true,
            camera: true,
            multi_monitor: false,
            network_bridge: false,
            vpn_passthrough: false,
            touchscreen: false,
            printing: true,
        }
    }

    /// Capabilities for Linux with KVM
    pub fn linux_kvm() -> Self {
        PlatformCapabilities {
            vm_lifecycle: true,
            snapshots: true,
            shared_folders: true,
            clipboard: false,
            drag_drop: false,
            gpu_acceleration: true,
            usb_passthrough: true,
            audio: true,
            microphone: true,
            camera: false,
            multi_monitor: true,
            network_bridge: true,
            vpn_passthrough: true,
            touchscreen: false,
            printing: true,
        }
    }

    /// Count enabled capabilities
    pub fn enabled_count(&self) -> usize {
        vec![
            self.vm_lifecycle,
            self.snapshots,
            self.shared_folders,
            self.clipboard,
            self.drag_drop,
            self.gpu_acceleration,
            self.usb_passthrough,
            self.audio,
            self.microphone,
            self.camera,
            self.multi_monitor,
            self.network_bridge,
            self.vpn_passthrough,
            self.touchscreen,
            self.printing,
        ]
        .iter()
        .filter(|&&c| c)
        .count()
    }
}

/// Platform detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Detected platform
    pub platform: HostPlatform,
    /// Detected hypervisor backend
    pub hypervisor: HypervisorBackend,
    /// Platform capabilities
    pub capabilities: PlatformCapabilities,
    /// OS version
    pub os_version: String,
    /// Architecture (x86_64, arm64, etc.)
    pub arch: String,
    /// Available memory in GB
    pub available_memory_gb: u32,
    /// CPU core count
    pub cpu_cores: u32,
}

impl PlatformInfo {
    /// Detect current platform
    pub fn detect() -> Result<Self, Box<dyn std::error::Error>> {
        // This is a stub - actual implementation would use platform-specific detection
        #[cfg(target_os = "windows")]
        {
            Ok(PlatformInfo {
                platform: HostPlatform::Windows,
                hypervisor: HypervisorBackend::HyperV,
                capabilities: PlatformCapabilities::windows_hyperv(),
                os_version: "Windows 11".to_string(),
                arch: "x86_64".to_string(),
                available_memory_gb: 16,
                cpu_cores: 8,
            })
        }

        #[cfg(target_os = "macos")]
        {
            Ok(PlatformInfo {
                platform: HostPlatform::MacOS,
                hypervisor: HypervisorBackend::AppleVirtualization,
                capabilities: PlatformCapabilities::macos_apple(),
                os_version: "macOS Sonoma".to_string(),
                arch: "aarch64".to_string(),
                available_memory_gb: 16,
                cpu_cores: 10,
            })
        }

        #[cfg(target_os = "linux")]
        {
            Ok(PlatformInfo {
                platform: HostPlatform::Linux,
                hypervisor: HypervisorBackend::KVM,
                capabilities: PlatformCapabilities::linux_kvm(),
                os_version: "Ubuntu 22.04".to_string(),
                arch: "x86_64".to_string(),
                available_memory_gb: 32,
                cpu_cores: 16,
            })
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err("Unsupported platform".into())
        }
    }
}

/// VM resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMResourceConfig {
    /// CPU cores to allocate
    pub cpu_cores: u32,
    /// Memory in GB
    pub memory_gb: u32,
    /// Disk space in GB
    pub disk_gb: u32,
    /// Enable GPU acceleration
    pub gpu_enabled: bool,
}

/// Networking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMode {
    /// NAT (Network Address Translation)
    NAT,
    /// Bridged mode
    Bridged,
    /// Host-only mode
    HostOnly,
    /// Custom configuration
    Custom(HashMap<String, String>),
}

/// Storage mount point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMount {
    /// Host path
    pub host_path: PathBuf,
    /// VM path
    pub vm_path: PathBuf,
    /// Read-only
    pub read_only: bool,
}

/// Platform adapter trait - defines interface all platforms must implement
pub trait PlatformAdapter: Send + Sync {
    /// Get platform info
    fn platform_info(&self) -> &PlatformInfo;

    /// Create VM instance
    fn create_vm(
        &self,
        name: &str,
        config: &VMResourceConfig,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Start VM
    fn start_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Stop VM
    fn stop_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Suspend VM
    fn suspend_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Resume VM
    fn resume_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Delete VM
    fn delete_vm(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Configure networking
    fn configure_network(
        &self,
        vm_id: &str,
        mode: &NetworkMode,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Mount storage
    fn mount_storage(
        &self,
        vm_id: &str,
        mount: &StorageMount,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Enable GPU acceleration
    fn enable_gpu(&self, vm_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Access clipboard
    fn clipboard_read(&self) -> Result<String, Box<dyn std::error::Error>>;

    /// Write to clipboard
    fn clipboard_write(&self, content: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Allocate USB device
    fn allocate_usb(&self, vm_id: &str, device_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Release USB device
    fn release_usb(&self, vm_id: &str, device_id: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Name of this adapter
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_platform_display() {
        assert_eq!(HostPlatform::Windows.to_string(), "windows");
        assert_eq!(HostPlatform::MacOS.to_string(), "macos");
        assert_eq!(HostPlatform::Linux.to_string(), "linux");
    }

    #[test]
    fn test_hypervisor_backend_display() {
        assert_eq!(HypervisorBackend::HyperV.to_string(), "hyper-v");
        assert_eq!(
            HypervisorBackend::AppleVirtualization.to_string(),
            "apple-virtualization"
        );
        assert_eq!(HypervisorBackend::KVM.to_string(), "kvm");
    }

    #[test]
    fn test_platform_capabilities_windows() {
        let caps = PlatformCapabilities::windows_hyperv();
        assert!(caps.vm_lifecycle);
        assert!(caps.clipboard);
        assert!(!caps.gpu_acceleration);
        assert_eq!(caps.enabled_count(), 10);
    }

    #[test]
    fn test_platform_capabilities_macos() {
        let caps = PlatformCapabilities::macos_apple();
        assert!(caps.vm_lifecycle);
        assert!(caps.gpu_acceleration);
        assert!(!caps.snapshots);
        assert_eq!(caps.enabled_count(), 9);
    }

    #[test]
    fn test_platform_capabilities_linux() {
        let caps = PlatformCapabilities::linux_kvm();
        assert!(caps.vm_lifecycle);
        assert!(caps.gpu_acceleration);
        assert!(caps.usb_passthrough);
        assert_eq!(caps.enabled_count(), 11);
    }

    #[test]
    fn test_vm_resource_config() {
        let config = VMResourceConfig {
            cpu_cores: 4,
            memory_gb: 8,
            disk_gb: 40,
            gpu_enabled: false,
        };
        assert_eq!(config.cpu_cores, 4);
        assert_eq!(config.memory_gb, 8);
    }

    #[test]
    fn test_storage_mount() {
        let mount = StorageMount {
            host_path: PathBuf::from("/home/user/projects"),
            vm_path: PathBuf::from("/projects"),
            read_only: false,
        };
        assert!(!mount.read_only);
    }

    #[test]
    fn test_network_mode_nat() {
        let mode = NetworkMode::NAT;
        assert!(matches!(mode, NetworkMode::NAT));
    }

    #[test]
    fn test_platform_info_detection() {
        if let Ok(info) = PlatformInfo::detect() {
            assert!(!info.os_version.is_empty());
            assert!(!info.arch.is_empty());
            assert!(info.cpu_cores > 0);
            assert!(info.available_memory_gb > 0);
        }
    }

    #[test]
    fn test_platform_capabilities_enabled_count() {
        let mut caps = PlatformCapabilities {
            vm_lifecycle: true,
            snapshots: false,
            shared_folders: true,
            clipboard: false,
            drag_drop: false,
            gpu_acceleration: false,
            usb_passthrough: false,
            audio: false,
            microphone: false,
            camera: false,
            multi_monitor: false,
            network_bridge: false,
            vpn_passthrough: false,
            touchscreen: false,
            printing: false,
        };
        assert_eq!(caps.enabled_count(), 2);
        caps.gpu_acceleration = true;
        assert_eq!(caps.enabled_count(), 3);
    }
}
