use anyhow::Result;
use tinybridge_core::{Environment, EnvironmentStatus, Resources};
use tinybridge_vz::{VirtualMachine, VmConfig};
use uuid::Uuid;

pub struct VmManager {
    vms: std::collections::HashMap<Uuid, VirtualMachine>,
}

impl VmManager {
    pub fn new() -> Self {
        VmManager {
            vms: std::collections::HashMap::new(),
        }
    }

    pub fn create_vm(
        &mut self,
        id: Uuid,
        name: String,
        kernel_path: String,
        disk_path: String,
        resources: Resources,
    ) -> Result<()> {
        if !VirtualMachine::is_available() {
            return Err(anyhow::anyhow!("VZ Framework not available"));
        }

        let config = VmConfig::new(kernel_path, disk_path, resources).with_cmdline(
            "root=/dev/vda1 rw console=hvc0 quiet systemd.unified_cgroup_hierarchy=1".to_string(),
        );

        let vm = VirtualMachine::new(name, config)?;
        self.vms.insert(id, vm);
        Ok(())
    }

    pub fn start_vm(&self, id: Uuid) -> Result<()> {
        let vm = self
            .vms
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("VM not found"))?;
        vm.start().map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn stop_vm(&self, id: Uuid) -> Result<()> {
        let vm = self
            .vms
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("VM not found"))?;
        vm.stop().map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn force_stop_vm(&self, id: Uuid) -> Result<()> {
        let vm = self
            .vms
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("VM not found"))?;
        vm.force_stop().map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn destroy_vm(&mut self, id: Uuid) -> Result<()> {
        self.vms.remove(&id);
        Ok(())
    }

    pub fn vm_exists(&self, id: Uuid) -> bool {
        self.vms.contains_key(&id)
    }
}

impl Default for VmManager {
    fn default() -> Self {
        Self::new()
    }
}
