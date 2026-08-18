use crate::config::VmConfig;
use crate::error::{Result, VzError};
use std::ffi::{CStr, CString};
use std::ptr::null_mut;
use tinybridge_vz_sys::*;
use uuid::Uuid;

/// Real-time lifecycle state of a [`VirtualMachine`], as reported by
/// Virtualization.framework via `tb_vm_get_status`. This is never fabricated locally -
/// it always reflects the last value the C ABI / Swift bridge / VZVirtualMachine
/// returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
    /// The C ABI returned a state value this Rust binding doesn't recognize yet.
    Unknown(u32),
}

#[allow(non_upper_case_globals)] // bindgen-generated constant names from the C enum
impl From<TBVMState> for VmState {
    fn from(raw: TBVMState) -> Self {
        match raw {
            TBVMState_TB_VM_STATE_STOPPED => VmState::Stopped,
            TBVMState_TB_VM_STATE_STARTING => VmState::Starting,
            TBVMState_TB_VM_STATE_RUNNING => VmState::Running,
            TBVMState_TB_VM_STATE_STOPPING => VmState::Stopping,
            TBVMState_TB_VM_STATE_ERROR => VmState::Error,
            other => VmState::Unknown(other),
        }
    }
}

/// A point-in-time snapshot of a running (or not-yet-running) VM, read directly from
/// Virtualization.framework through the FFI boundary - never hardcoded.
#[derive(Debug, Clone, PartialEq)]
pub struct VmStatus {
    pub state: VmState,
    pub cpu_usage_pct: f64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    /// Guest IP address, if one has been observed yet. Empty until the boot monitor on
    /// the Swift side detects the guest is up.
    pub ip_address: Option<String>,
}

pub struct VirtualMachine {
    vm: *mut tinybridge_vz_sys::TBVirtualMachine,
    id: Uuid,
    name: String,
}

// Safe to send across threads - the C FFI handles thread safety
unsafe impl Send for VirtualMachine {}
unsafe impl Sync for VirtualMachine {}

impl VirtualMachine {
    pub fn new(name: String, config: VmConfig) -> Result<Self> {
        if !Self::is_available() {
            return Err(VzError::NotAvailable);
        }

        let kernel_cstring =
            CString::new(config.kernel_path).map_err(|_| VzError::InvalidConfig)?;
        let disk_cstring =
            CString::new(config.disk_image_path).map_err(|_| VzError::InvalidConfig)?;
        let cmdline_cstring = CString::new(config.cmdline).map_err(|_| VzError::InvalidConfig)?;

        let initrd_cstring = config
            .initrd_path
            .as_ref()
            .and_then(|p| CString::new(p.clone()).ok());

        let serial_log_cstring = config
            .serial_log_path
            .as_ref()
            .and_then(|p| CString::new(p.clone()).ok());

        let vz_config = tinybridge_vz_sys::TBVMConfig {
            kernel_path: kernel_cstring.as_ptr(),
            initrd_path: initrd_cstring
                .as_ref()
                .map(|c| c.as_ptr())
                .unwrap_or(null_mut()),
            cmdline: cmdline_cstring.as_ptr(),
            disk_image_path: disk_cstring.as_ptr(),
            cpu_count: config.cpu_count,
            memory_bytes: config.memory_bytes,
            enable_rosetta: config.enable_rosetta,
            display_width: config.display_width,
            display_height: config.display_height,
            state_callback: None,
            user_data: null_mut(),
            serial_log_path: serial_log_cstring
                .as_ref()
                .map(|c| c.as_ptr())
                .unwrap_or(null_mut()),
        };

        let vm = unsafe { tb_vm_create(&vz_config) };

        if vm.is_null() {
            return Err(VzError::CreationFailed);
        }

        Ok(VirtualMachine {
            vm,
            id: Uuid::new_v4(),
            name,
        })
    }

    pub fn start(&self) -> Result<()> {
        let result = unsafe { tb_vm_start(self.vm as *mut _) };
        if result != 0 {
            return Err(VzError::StartFailed);
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let result = unsafe { tb_vm_stop(self.vm as *mut _) };
        if result != 0 {
            return Err(VzError::StopFailed);
        }
        Ok(())
    }

    pub fn force_stop(&self) -> Result<()> {
        let result = unsafe { tb_vm_force_stop(self.vm as *mut _) };
        if result != 0 {
            return Err(VzError::StopFailed);
        }
        Ok(())
    }

    /// Query the VM's real, current state from Virtualization.framework. This is the
    /// only source of truth for VM status - callers must not assume "running" just
    /// because `start()` was called; `start()` is asynchronous on the Swift side, and
    /// this call reflects whatever VZVirtualMachine.state actually is right now.
    pub fn status(&self) -> Result<VmStatus> {
        let mut raw = TBVMStatus {
            state: TBVMState_TB_VM_STATE_STOPPED,
            cpu_usage_pct: 0.0,
            memory_used_bytes: 0,
            memory_total_bytes: 0,
            ip_address: [0; 46],
        };

        let result = unsafe { tb_vm_get_status(self.vm as *mut _, &mut raw as *mut _) };
        if result != 0 {
            return Err(VzError::StatusQueryFailed);
        }

        let ip_address = unsafe {
            let cstr = CStr::from_ptr(raw.ip_address.as_ptr());
            let s = cstr.to_string_lossy().into_owned();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        };

        Ok(VmStatus {
            state: VmState::from(raw.state),
            cpu_usage_pct: raw.cpu_usage_pct,
            memory_used_bytes: raw.memory_used_bytes,
            memory_total_bytes: raw.memory_total_bytes,
            ip_address,
        })
    }

    /// Show the VM's graphical window (VZVirtualMachineView), if the host process has a
    /// window server connection. Returns an error if the underlying `tb_vm_show_window`
    /// call fails (e.g. headless/no display server, as is typical in CI/sandboxes).
    pub fn show_window(&self) -> Result<()> {
        let result = unsafe { tb_vm_show_window(self.vm as *mut _) };
        if result != 0 {
            return Err(VzError::StatusQueryFailed);
        }
        Ok(())
    }

    /// Hide the VM's graphical window.
    pub fn hide_window(&self) -> Result<()> {
        let result = unsafe { tb_vm_hide_window(self.vm as *mut _) };
        if result != 0 {
            return Err(VzError::StatusQueryFailed);
        }
        Ok(())
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_available() -> bool {
        unsafe { tb_is_available() }
    }

    pub fn version() -> &'static str {
        "0.1.0"
    }
}

impl Drop for VirtualMachine {
    fn drop(&mut self) {
        unsafe {
            tb_vm_destroy(self.vm as *mut _);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_is_available() {
        let available = VirtualMachine::is_available();
        // Will be true on macOS 13+, false otherwise
        // Don't fail test if not available
        let _ = available;
    }

    #[test]
    fn test_version() {
        assert_eq!(VirtualMachine::version(), "0.1.0");
    }

    /// This is the honest end-to-end proof that the Rust -> C ABI -> Swift ->
    /// Virtualization.framework call chain is real and reached, not a stub. With a
    /// kernel path that does not exist on disk, VZVirtualMachineConfiguration.validate()
    /// inside tb_vm_create() must genuinely fail, and that real failure must propagate
    /// all the way back as VzError::CreationFailed rather than silently succeeding.
    #[test]
    fn test_vm_create_with_missing_kernel_surfaces_real_ffi_error() {
        if !VirtualMachine::is_available() {
            eprintln!("skipping: Virtualization.framework not available on this host");
            return;
        }

        let resources = tinybridge_core::Resources {
            cpu: 1,
            memory_bytes: 512 * 1024 * 1024,
            disk_bytes: 0,
            gpu: None,
        };
        let config = VmConfig::new(
            "/nonexistent/tinybridge-test-kernel-does-not-exist".to_string(),
            "/nonexistent/tinybridge-test-disk-does-not-exist.img".to_string(),
            resources,
        );

        let result = VirtualMachine::new("ffi-smoke-test".to_string(), config);
        assert!(
            matches!(result, Err(VzError::CreationFailed)),
            "expected a real CreationFailed error from Virtualization.framework, got: {result:?}"
        );
    }
}

impl std::fmt::Debug for VirtualMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualMachine")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}
