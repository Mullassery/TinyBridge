use tinybridge_core::Resources;

pub struct VmConfig {
    pub cpu_count: u32,
    pub memory_bytes: u64,
    pub kernel_path: String,
    pub initrd_path: Option<String>,
    pub cmdline: String,
    pub disk_image_path: String,
    pub enable_rosetta: bool,
}

impl VmConfig {
    pub fn new(kernel_path: String, disk_image_path: String, resources: Resources) -> Self {
        VmConfig {
            cpu_count: resources.cpu,
            memory_bytes: resources.memory_bytes,
            kernel_path,
            initrd_path: None,
            cmdline: "root=/dev/vda1 rw console=hvc0 quiet".to_string(),
            disk_image_path,
            enable_rosetta: true,
        }
    }

    pub fn with_initrd(mut self, path: String) -> Self {
        self.initrd_path = Some(path);
        self
    }

    pub fn with_cmdline(mut self, cmdline: String) -> Self {
        self.cmdline = cmdline;
        self
    }

    pub fn with_rosetta(mut self, enabled: bool) -> Self {
        self.enable_rosetta = enabled;
        self
    }
}
