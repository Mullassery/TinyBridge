use crate::error::Result;
use crate::vm::VirtualMachine;

pub struct VirtioFS {
    host_path: String,
    mount_tag: String,
    read_only: bool,
}

impl VirtioFS {
    pub fn new(host_path: String, mount_tag: String) -> Self {
        VirtioFS {
            host_path,
            mount_tag,
            read_only: false,
        }
    }

    pub fn read_only(mut self, ro: bool) -> Self {
        self.read_only = ro;
        self
    }

    pub fn attach(&self, _vm: &VirtualMachine) -> Result<()> {
        // In Week 4, this will call tb_vm_add_virtiofs
        // For now, stub
        Ok(())
    }

    pub fn host_path(&self) -> &str {
        &self.host_path
    }

    pub fn mount_tag(&self) -> &str {
        &self.mount_tag
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtiofs_new() {
        let fs = VirtioFS::new("/home/user".to_string(), "home".to_string());
        assert_eq!(fs.host_path(), "/home/user");
        assert_eq!(fs.mount_tag(), "home");
        assert!(!fs.is_read_only());
    }

    #[test]
    fn test_virtiofs_read_only() {
        let fs = VirtioFS::new("/data".to_string(), "data".to_string()).read_only(true);
        assert!(fs.is_read_only());
    }
}
