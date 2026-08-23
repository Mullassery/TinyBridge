/// Resource Enforcement and Limits
/// Phase 4.0.3: Profile-Based Resource Management
///
/// Apply and enforce resource limits for daemon processes
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resource limit types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitType {
    Hard,
    Soft,
}

/// CPU resource limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuLimit {
    /// Number of CPU cores
    pub cores: u32,
    /// CPU percentage cap (0-100)
    pub percent: u32,
    /// Millicores allocation
    pub millicores: u32,
}

impl CpuLimit {
    /// Create CPU limit from cores
    pub fn from_cores(cores: u32) -> Self {
        CpuLimit {
            cores,
            percent: 100,
            millicores: cores * 1000,
        }
    }

    /// Cap CPU percentage
    pub fn with_percent(mut self, percent: u32) -> Self {
        self.percent = percent.min(100);
        self
    }
}

/// Memory resource limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimit {
    /// Memory in bytes
    pub bytes: u64,
    /// Memory in GB
    pub gb: u32,
    /// Swap allowed in bytes
    pub swap_bytes: u64,
}

impl MemoryLimit {
    /// Create memory limit from GB
    pub fn from_gb(gb: u32) -> Self {
        let bytes = gb as u64 * 1024 * 1024 * 1024;
        MemoryLimit {
            bytes,
            gb,
            swap_bytes: bytes / 2, // Half of memory as swap by default
        }
    }

    /// With custom swap
    pub fn with_swap_gb(mut self, swap_gb: u32) -> Self {
        self.swap_bytes = swap_gb as u64 * 1024 * 1024 * 1024;
        self
    }

    /// Disable swap
    pub fn no_swap(mut self) -> Self {
        self.swap_bytes = 0;
        self
    }
}

/// Disk I/O limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskLimit {
    /// Max size in GB
    pub size_gb: u32,
    /// Read bandwidth in MB/s
    pub read_mbps: u32,
    /// Write bandwidth in MB/s
    pub write_mbps: u32,
    /// IOPS limit (operations per second)
    pub iops: u32,
}

impl DiskLimit {
    /// Create disk limit from size
    pub fn from_size_gb(size_gb: u32) -> Self {
        DiskLimit {
            size_gb,
            read_mbps: 200,  // Default: 200 MB/s
            write_mbps: 100, // Default: 100 MB/s
            iops: 5000,      // Default: 5000 IOPS
        }
    }

    /// Set read/write bandwidth
    pub fn with_bandwidth(mut self, read_mbps: u32, write_mbps: u32) -> Self {
        self.read_mbps = read_mbps;
        self.write_mbps = write_mbps;
        self
    }

    /// Set IOPS limit
    pub fn with_iops(mut self, iops: u32) -> Self {
        self.iops = iops;
        self
    }
}

/// Network resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLimit {
    /// Bandwidth in Mbps
    pub bandwidth_mbps: u32,
    /// Latency in milliseconds
    pub latency_ms: u32,
    /// Packet loss percentage (0-100)
    pub packet_loss_percent: u32,
    /// Connection limit
    pub max_connections: u32,
}

impl NetworkLimit {
    /// Create network limit
    pub fn new(bandwidth_mbps: u32) -> Self {
        NetworkLimit {
            bandwidth_mbps,
            latency_ms: 0,
            packet_loss_percent: 0,
            max_connections: 65535,
        }
    }

    /// Add latency simulation
    pub fn with_latency(mut self, latency_ms: u32) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    /// Add packet loss
    pub fn with_packet_loss(mut self, percent: u32) -> Self {
        self.packet_loss_percent = percent.min(100);
        self
    }

    /// Set connection limit
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }
}

/// Complete resource enforcement policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePolicy {
    /// CPU limits
    pub cpu: CpuLimit,
    /// Memory limits
    pub memory: MemoryLimit,
    /// Disk limits
    pub disk: DiskLimit,
    /// Network limits
    pub network: Option<NetworkLimit>,
    /// Process priority (nice value: -20 to 19)
    pub priority: i32,
    /// Enable OOM killer
    pub oom_killer: bool,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

impl ResourcePolicy {
    /// Create policy for development
    pub fn development() -> Self {
        ResourcePolicy {
            cpu: CpuLimit::from_cores(4),
            memory: MemoryLimit::from_gb(8).with_swap_gb(4),
            disk: DiskLimit::from_size_gb(40),
            network: Some(NetworkLimit::new(1000)),
            priority: 0,
            oom_killer: true,
            env_vars: HashMap::new(),
        }
    }

    /// Create policy for production
    pub fn production() -> Self {
        ResourcePolicy {
            cpu: CpuLimit::from_cores(8),
            memory: MemoryLimit::from_gb(16).no_swap(),
            disk: DiskLimit::from_size_gb(100)
                .with_bandwidth(500, 300)
                .with_iops(10000),
            network: Some(NetworkLimit::new(10000)),
            priority: -5,
            oom_killer: false,
            env_vars: HashMap::new(),
        }
    }

    /// Create policy for testing
    pub fn testing() -> Self {
        ResourcePolicy {
            cpu: CpuLimit::from_cores(2),
            memory: MemoryLimit::from_gb(4),
            disk: DiskLimit::from_size_gb(30),
            network: Some(NetworkLimit::new(500)),
            priority: 0,
            oom_killer: true,
            env_vars: HashMap::new(),
        }
    }

    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority.max(-20).min(19);
        self
    }

    /// Validate policy constraints
    pub fn validate(&self) -> Result<(), String> {
        if self.cpu.cores == 0 || self.cpu.cores > 128 {
            return Err("CPU cores must be between 1 and 128".to_string());
        }
        if self.memory.gb == 0 || self.memory.gb > 2048 {
            return Err("Memory must be between 1GB and 2048GB".to_string());
        }
        if self.disk.size_gb == 0 {
            return Err("Disk size must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Export as cgroup configuration
    pub fn to_cgroup_config(&self) -> String {
        let mut config = String::new();
        config.push_str(&format!("# CPU limits\n"));
        config.push_str(&format!("cpus={}\n", self.cpu.cores));
        config.push_str(&format!("cpu_percent={}\n", self.cpu.percent));
        config.push_str(&format!("\n# Memory limits\n"));
        config.push_str(&format!("memory_limit={}\n", self.memory.bytes));
        config.push_str(&format!("memory_swap_limit={}\n", self.memory.swap_bytes));
        config.push_str(&format!("\n# Disk limits\n"));
        config.push_str(&format!("disk_size={}\n", self.disk.size_gb));
        config.push_str(&format!("read_bw_max={}M\n", self.disk.read_mbps));
        config.push_str(&format!("write_bw_max={}M\n", self.disk.write_mbps));
        config.push_str(&format!("\n# Process priority\n"));
        config.push_str(&format!("nice={}\n", self.priority));
        config
    }
}

/// Resource monitor for tracking usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory used in bytes
    pub memory_bytes: u64,
    /// Disk used in bytes
    pub disk_bytes: u64,
    /// Network bytes in
    pub network_in_bytes: u64,
    /// Network bytes out
    pub network_out_bytes: u64,
    /// Number of processes
    pub process_count: u32,
    /// Open file descriptors
    pub open_fds: u32,
}

impl ResourceUsage {
    /// Check if usage within limits
    pub fn within_limits(&self, policy: &ResourcePolicy) -> bool {
        let cpu_ok = self.cpu_percent <= policy.cpu.percent as f64;
        let memory_ok = self.memory_bytes <= policy.memory.bytes;
        let disk_ok = self.disk_bytes <= (policy.disk.size_gb as u64 * 1024 * 1024 * 1024);
        cpu_ok && memory_ok && disk_ok
    }

    /// Get usage percentage for each resource
    pub fn to_percentages(&self, policy: &ResourcePolicy) -> ResourcePercentages {
        ResourcePercentages {
            cpu_percent: (self.cpu_percent / policy.cpu.percent as f64 * 100.0).min(100.0),
            memory_percent: (self.memory_bytes as f64 / policy.memory.bytes as f64 * 100.0)
                .min(100.0),
            disk_percent: (self.disk_bytes as f64
                / (policy.disk.size_gb as u64 * 1024 * 1024 * 1024) as f64
                * 100.0)
                .min(100.0),
        }
    }
}

/// Resource usage percentages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePercentages {
    /// CPU usage as percentage of limit
    pub cpu_percent: f64,
    /// Memory usage as percentage of limit
    pub memory_percent: f64,
    /// Disk usage as percentage of limit
    pub disk_percent: f64,
}

impl ResourcePercentages {
    /// Check if any resource is over-utilized (>90%)
    pub fn has_high_utilization(&self) -> bool {
        self.cpu_percent > 90.0 || self.memory_percent > 90.0 || self.disk_percent > 90.0
    }

    /// Check if any resource is critically over-utilized (>95%)
    pub fn has_critical_utilization(&self) -> bool {
        self.cpu_percent > 95.0 || self.memory_percent > 95.0 || self.disk_percent > 95.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_limit() {
        let cpu = CpuLimit::from_cores(4).with_percent(80);
        assert_eq!(cpu.cores, 4);
        assert_eq!(cpu.percent, 80);
        assert_eq!(cpu.millicores, 4000);
    }

    #[test]
    fn test_memory_limit() {
        let mem = MemoryLimit::from_gb(8);
        assert_eq!(mem.gb, 8);
        assert_eq!(mem.bytes, 8 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_memory_limit_no_swap() {
        let mem = MemoryLimit::from_gb(8).no_swap();
        assert_eq!(mem.swap_bytes, 0);
    }

    #[test]
    fn test_disk_limit() {
        let disk = DiskLimit::from_size_gb(100)
            .with_bandwidth(500, 300)
            .with_iops(10000);
        assert_eq!(disk.size_gb, 100);
        assert_eq!(disk.read_mbps, 500);
        assert_eq!(disk.write_mbps, 300);
        assert_eq!(disk.iops, 10000);
    }

    #[test]
    fn test_network_limit() {
        let net = NetworkLimit::new(1000)
            .with_latency(10)
            .with_packet_loss(1)
            .with_max_connections(10000);

        assert_eq!(net.bandwidth_mbps, 1000);
        assert_eq!(net.latency_ms, 10);
        assert_eq!(net.packet_loss_percent, 1);
        assert_eq!(net.max_connections, 10000);
    }

    #[test]
    fn test_resource_policy_development() {
        let policy = ResourcePolicy::development();
        assert_eq!(policy.cpu.cores, 4);
        assert_eq!(policy.memory.gb, 8);
        assert_eq!(policy.disk.size_gb, 40);
        assert!(policy.oom_killer);
    }

    #[test]
    fn test_resource_policy_production() {
        let policy = ResourcePolicy::production();
        assert_eq!(policy.cpu.cores, 8);
        assert_eq!(policy.memory.gb, 16);
        assert_eq!(policy.disk.size_gb, 100);
        assert!(!policy.oom_killer);
        assert_eq!(policy.priority, -5);
    }

    #[test]
    fn test_resource_policy_validate() {
        let policy = ResourcePolicy::development();
        assert!(policy.validate().is_ok());

        let mut bad_policy = ResourcePolicy::development();
        bad_policy.cpu.cores = 0;
        assert!(bad_policy.validate().is_err());
    }

    #[test]
    fn test_resource_policy_cgroup_config() {
        let policy = ResourcePolicy::development();
        let config = policy.to_cgroup_config();
        assert!(config.contains("cpus=4"));
        assert!(config.contains("memory_limit="));
        assert!(config.contains("disk_size=40"));
    }

    #[test]
    fn test_resource_usage_within_limits() {
        let policy = ResourcePolicy::development();
        let usage = ResourceUsage {
            cpu_percent: 50.0,
            memory_bytes: 4 * 1024 * 1024 * 1024,
            disk_bytes: 20 * 1024 * 1024 * 1024,
            ..Default::default()
        };

        assert!(usage.within_limits(&policy));
    }

    #[test]
    fn test_resource_usage_exceeds_limits() {
        let policy = ResourcePolicy::development();
        let usage = ResourceUsage {
            cpu_percent: 101.0,
            memory_bytes: 4 * 1024 * 1024 * 1024,
            disk_bytes: 20 * 1024 * 1024 * 1024,
            ..Default::default()
        };

        assert!(!usage.within_limits(&policy));
    }

    #[test]
    fn test_resource_percentages() {
        let policy = ResourcePolicy::development();
        let usage = ResourceUsage {
            cpu_percent: 80.0,
            memory_bytes: 4 * 1024 * 1024 * 1024,
            disk_bytes: 20 * 1024 * 1024 * 1024,
            ..Default::default()
        };

        let pct = usage.to_percentages(&policy);
        assert!(pct.cpu_percent > 0.0);
        assert!(pct.memory_percent > 0.0);
        assert!(pct.disk_percent > 0.0);
    }

    #[test]
    fn test_high_utilization_detection() {
        let pct = ResourcePercentages {
            cpu_percent: 95.0,
            memory_percent: 50.0,
            disk_percent: 50.0,
        };

        assert!(pct.has_high_utilization());
    }

    #[test]
    fn test_critical_utilization_detection() {
        let pct = ResourcePercentages {
            cpu_percent: 96.0,
            memory_percent: 50.0,
            disk_percent: 50.0,
        };

        assert!(pct.has_critical_utilization());
    }
}
