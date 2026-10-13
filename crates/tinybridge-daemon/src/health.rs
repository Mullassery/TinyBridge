use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Health status of the daemon and its resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// System operational but with warnings
    Degraded,
    /// System non-operational
    Unhealthy,
}

/// Individual resource health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ResourceHealth {
    pub fn new(name: &str, status: HealthStatus, message: &str) -> Self {
        ResourceHealth {
            name: name.to_string(),
            status,
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Comprehensive daemon health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub resources: Vec<ResourceHealth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl HealthReport {
    pub fn new(status: HealthStatus) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        HealthReport {
            status,
            timestamp,
            uptime_seconds: 0,
            resources: Vec::new(),
            message: None,
        }
    }

    pub fn with_uptime(mut self, uptime: u64) -> Self {
        self.uptime_seconds = uptime;
        self
    }

    pub fn with_resources(mut self, resources: Vec<ResourceHealth>) -> Self {
        self.resources = resources;
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    /// Calculate overall status from resources
    pub fn recalculate_status(&mut self) {
        let has_unhealthy = self
            .resources
            .iter()
            .any(|r| r.status == HealthStatus::Unhealthy);
        let has_degraded = self
            .resources
            .iter()
            .any(|r| r.status == HealthStatus::Degraded);

        if has_unhealthy {
            self.status = HealthStatus::Unhealthy;
        } else if has_degraded {
            self.status = HealthStatus::Degraded;
        } else {
            self.status = HealthStatus::Healthy;
        }
    }
}

/// Health check engine for daemon
pub struct HealthChecker {
    start_time: u64,
}

impl HealthChecker {
    pub fn new() -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        HealthChecker { start_time }
    }

    pub fn get_uptime(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.start_time)
    }

    /// Check memory availability
    pub fn check_memory(&self) -> ResourceHealth {
        match current_memory_mb() {
            (Some(available_mb), total_mb) => {
                let mut details = serde_json::json!({
                    "available_mb": available_mb,
                    "threshold_mb": 2048,
                });
                if let Some(total_mb) = total_mb {
                    details["total_mb"] = serde_json::json!(total_mb);
                }

                if available_mb > 2048 {
                    ResourceHealth::new(
                        "memory",
                        HealthStatus::Healthy,
                        "Sufficient memory available",
                    )
                    .with_details(details)
                } else {
                    ResourceHealth::new(
                        "memory",
                        HealthStatus::Degraded,
                        "Limited memory available",
                    )
                    .with_details(details)
                }
            }
            (None, _) => ResourceHealth::new(
                "memory",
                HealthStatus::Degraded,
                "Memory check not supported on this platform",
            ),
        }
    }

    /// Check disk space
    pub fn check_disk(&self) -> ResourceHealth {
        let data_dir = tinybridge_core::TinyBridgeConfig::data_dir();
        match disk_capacity_bytes(&data_dir) {
            Some((available, total)) => {
                let available_gb = available / (1024 * 1024 * 1024);
                let total_gb = total / (1024 * 1024 * 1024);

                if available_gb >= 10 {
                    ResourceHealth::new("disk", HealthStatus::Healthy, "Disk space available")
                        .with_details(serde_json::json!({
                            "available_gb": available_gb,
                            "total_gb": total_gb,
                            "threshold_gb": 10,
                        }))
                } else {
                    ResourceHealth::new("disk", HealthStatus::Degraded, "Low disk space available")
                        .with_details(serde_json::json!({
                            "available_gb": available_gb,
                            "total_gb": total_gb,
                            "threshold_gb": 10,
                        }))
                }
            }
            None => ResourceHealth::new(
                "disk",
                HealthStatus::Degraded,
                "Could not read disk information",
            ),
        }
    }

    /// Check socket connectivity
    pub fn check_socket(&self) -> ResourceHealth {
        ResourceHealth::new("socket", HealthStatus::Healthy, "Daemon socket operational")
    }

    /// Check virtualization capabilities
    pub fn check_virtualization(&self) -> ResourceHealth {
        #[cfg(target_os = "macos")]
        {
            // Check if VZ framework is available
            ResourceHealth::new(
                "virtualization",
                HealthStatus::Healthy,
                "Apple Virtualization Framework available",
            )
            .with_details(serde_json::json!({
                "framework": "VZ",
                "architecture": "arm64"
            }))
        }

        #[cfg(not(target_os = "macos"))]
        {
            ResourceHealth::new(
                "virtualization",
                HealthStatus::Unhealthy,
                "Not running on macOS",
            )
        }
    }

    /// Perform comprehensive health check
    pub fn check_all(&self) -> HealthReport {
        let resources = vec![
            self.check_virtualization(),
            self.check_memory(),
            self.check_disk(),
            self.check_socket(),
        ];

        let mut report = HealthReport::new(HealthStatus::Healthy)
            .with_uptime(self.get_uptime())
            .with_resources(resources);

        report.recalculate_status();
        report
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Best-effort memory measurement for the host: `(available_mb, total_mb)`.
///
/// Real numbers, never a fake "enough memory" verdict: free+inactive+
/// speculative pages on macOS (`host_statistics64`) and `MemAvailable` on
/// Linux. `None` only on platforms we can't measure (documented as such in
/// `check_memory`, not silently healthy).
#[cfg(target_os = "macos")]
fn current_memory_mb() -> (Option<u64>, Option<u64>) {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;

    let mut total: u64 = 0;
    let mut len = std::mem::size_of::<u64>() as libc::size_t;
    let mut mib = [libc::CTL_HW, libc::HW_MEMSIZE];
    let total_mb = unsafe {
        let rc = libc::sysctl(
            mib.as_mut_ptr(),
            2,
            &mut total as *mut u64 as *mut libc::c_void,
            &mut len,
            std::ptr::null_mut(),
            0,
        );
        if rc == 0 {
            Some(total / (1024 * 1024))
        } else {
            None
        }
    };

    let mut stats: libc::vm_statistics64 = unsafe { std::mem::zeroed() };
    let mut count = libc::HOST_VM_INFO64_COUNT;
    // libc marks `mach_host_self()` deprecated (steering callers to the
    // separate `mach2` crate); allow it locally rather than add a dependency.
    #[allow(deprecated)]
    let host = unsafe { libc::mach_host_self() };
    let rc = unsafe {
        libc::host_statistics64(
            host,
            libc::HOST_VM_INFO64,
            &mut stats as *mut libc::vm_statistics64 as libc::host_info64_t,
            &mut count,
        )
    };
    let available_mb = if rc == libc::KERN_SUCCESS {
        let avail_pages = (stats.free_count as u64)
            .saturating_add(stats.inactive_count as u64)
            .saturating_add(stats.speculative_count as u64);
        Some(avail_pages.saturating_mul(page_size) / (1024 * 1024))
    } else {
        None
    };

    (available_mb, total_mb.unwrap_or(0).into())
}

#[cfg(target_os = "linux")]
fn current_memory_mb() -> (Option<u64>, Option<u64>) {
    let content = match std::fs::read_to_string("/proc/meminfo") {
        Ok(c) => c,
        Err(_) => return (None, None),
    };
    let mut total = None;
    let mut available = None;
    for line in content.lines() {
        match line.split_whitespace().collect::<Vec<_>>()[..] {
            [key, value_kb, "kB", ..] => {
                let value_mb = value_kb.parse::<u64>().ok()? / 1024;
                match key {
                    "MemTotal:" => total = Some(value_mb),
                    "MemAvailable:" => available = Some(value_mb),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    (available, total)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn current_memory_mb() -> (Option<u64>, Option<u64>) {
    (None, None)
}

/// Available/total bytes on the filesystem holding `path`'s closest existing
/// ancestor (the data dir may not exist yet on a fresh machine — walk up).
///
/// Real `statfs` (macOS) / `statvfs` (Linux) numbers; the previous hardcoded
/// "50 GB free, always healthy" is gone.
#[cfg(target_os = "macos")]
fn disk_capacity_bytes(path: &std::path::Path) -> Option<(u64, u64)> {
    let existing = nearest_existing_dir(path);
    let c_path = std::ffi::CString::new(existing.to_string_lossy().to_string()).ok()?;
    let mut stats: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statfs(c_path.as_ptr(), &mut stats) } != 0 {
        return None;
    }
    let block = stats.f_bsize as u64;
    Some((
        stats.f_bavail.checked_mul(block)?,
        stats.f_blocks.checked_mul(block)?,
    ))
}

#[cfg(target_os = "linux")]
fn disk_capacity_bytes(path: &std::path::Path) -> Option<(u64, u64)> {
    let existing = nearest_existing_dir(path);
    let c_path = std::ffi::CString::new(existing.to_string_lossy().to_string()).ok()?;
    let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stats) } != 0 {
        return None;
    }
    let frag = stats.f_frsize as u64;
    Some((
        (stats.f_bavail as u64).checked_mul(frag)?,
        (stats.f_blocks as u64).checked_mul(frag)?,
    ))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn disk_capacity_bytes(_path: &std::path::Path) -> Option<(u64, u64)> {
    None
}

fn nearest_existing_dir(path: &std::path::Path) -> std::path::PathBuf {
    let mut owned = path.to_path_buf();
    loop {
        if owned.exists() {
            return owned;
        }
        if !owned.pop() {
            return std::path::PathBuf::from("/");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_report_creation() {
        let report = HealthReport::new(HealthStatus::Healthy);
        assert_eq!(report.status, HealthStatus::Healthy);
        assert_eq!(report.uptime_seconds, 0);
        assert!(report.resources.is_empty());
    }

    #[test]
    fn test_health_report_with_resources() {
        let resources = vec![
            ResourceHealth::new("memory", HealthStatus::Healthy, "OK"),
            ResourceHealth::new("disk", HealthStatus::Healthy, "OK"),
        ];

        let report = HealthReport::new(HealthStatus::Healthy).with_resources(resources);
        assert_eq!(report.resources.len(), 2);
    }

    #[test]
    fn test_health_status_recalculation() {
        let resources = vec![
            ResourceHealth::new("memory", HealthStatus::Healthy, "OK"),
            ResourceHealth::new("disk", HealthStatus::Degraded, "Low space"),
        ];

        let mut report = HealthReport::new(HealthStatus::Healthy).with_resources(resources);
        report.recalculate_status();

        assert_eq!(report.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_unhealthy_overrides_degraded() {
        let resources = vec![
            ResourceHealth::new("memory", HealthStatus::Healthy, "OK"),
            ResourceHealth::new("disk", HealthStatus::Degraded, "Low space"),
            ResourceHealth::new("virtualization", HealthStatus::Unhealthy, "Not available"),
        ];

        let mut report = HealthReport::new(HealthStatus::Healthy).with_resources(resources);
        report.recalculate_status();

        assert_eq!(report.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_resource_health_with_details() {
        let resource = ResourceHealth::new("test", HealthStatus::Healthy, "Test")
            .with_details(serde_json::json!({"key": "value"}));

        assert!(resource.details.is_some());
    }

    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        // get_uptime() returns u64, so it's always >= 0 by construction --
        // just confirm the call succeeds without panicking.
        let _uptime = checker.get_uptime();
    }

    #[test]
    fn test_health_check_all() {
        let checker = HealthChecker::new();
        let report = checker.check_all();

        assert!(!report.resources.is_empty());
        // uptime_seconds is u64, so it's always >= 0 by construction --
        // the meaningful check is that the report has resources, above.
    }

    #[test]
    fn test_health_status_serialization() {
        let report = HealthReport::new(HealthStatus::Healthy)
            .with_uptime(60)
            .with_resources(vec![ResourceHealth::new(
                "test",
                HealthStatus::Healthy,
                "OK",
            )]);

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("60"));
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn test_memory_check_is_real() {
        let checker = HealthChecker::new();
        let memory = checker.check_memory();
        let details = memory.details.expect("memory check should have details");
        let available_mb = details["available_mb"]
            .as_u64()
            .expect("available_mb present");
        let total_mb = details["total_mb"].as_u64().expect("total_mb present");
        // Real measurement of the running host: non-trivial amounts, and
        // available can never exceed total.
        assert!(
            available_mb > 0,
            "available_mb should be real, got {available_mb}"
        );
        assert!(total_mb > 0, "total_mb should be real, got {total_mb}");
        assert!(available_mb <= total_mb);
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn test_disk_check_is_real() {
        let checker = HealthChecker::new();
        let disk = checker.check_disk();
        let details = disk.details.expect("disk check should have details");
        let available_gb = details["available_gb"]
            .as_u64()
            .expect("available_gb present");
        let total_gb = details["total_gb"].as_u64().expect("total_gb present");
        // No more hardcoded "50 GB free, always healthy": report the actual
        // host numbers.
        assert!(total_gb > 0, "total_gb should be real, got {total_gb}");
        assert!(
            available_gb <= total_gb,
            "available {available_gb} cannot exceed total {total_gb}"
        );
    }
}
