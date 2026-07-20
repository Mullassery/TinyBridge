use crate::config::VmConfig;
use crate::error::{Result, VzError};
use std::ffi::CString;
use std::ptr::null_mut;
use tinybridge_vz_sys::*;
use uuid::Uuid;

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
            state_callback: None,
            user_data: null_mut(),
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
}
