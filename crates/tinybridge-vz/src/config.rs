use tinybridge_core::Resources;

#[derive(Clone)]
pub struct VmConfig {
    pub cpu_count: u32,
    pub memory_bytes: u64,
    pub kernel_path: String,
    pub initrd_path: Option<String>,
    pub cmdline: String,
    pub disk_image_path: String,
    pub enable_rosetta: bool,
    pub display_width: u32,
    pub display_height: u32,
    pub gpu_enabled: bool,
    pub gpu_memory_gb: Option<u32>,
}

impl VmConfig {
    pub fn new(kernel_path: String, disk_image_path: String, resources: Resources) -> Self {
        let (gpu_enabled, gpu_memory_gb) = if let Some(gpu_config) = &resources.gpu {
            (gpu_config.enabled, gpu_config.memory_gb)
        } else {
            (true, None) // GPU enabled by default
        };

        VmConfig {
            cpu_count: resources.cpu,
            memory_bytes: resources.memory_bytes,
            kernel_path,
            initrd_path: None,
            cmdline: "root=/dev/vda1 rw console=hvc0 quiet".to_string(),
            disk_image_path,
            enable_rosetta: true,
            display_width: 1920,
            display_height: 1080,
            gpu_enabled,
            gpu_memory_gb,
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

    pub fn with_display(mut self, width: u32, height: u32) -> Self {
        self.display_width = width;
        self.display_height = height;
        self
    }
}
