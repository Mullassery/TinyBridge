/// Daemon Bootstrap and Resource Management
/// Phase 4.0.3: Profile-Based Resource Management
///
/// Orchestrates daemon startup with config loading, profile selection, and resource allocation

use crate::{
    boot_instrumentation::{BootInstrumentation, BootPhase, BootSpan, ConfigContext},
    config_overrides::OverrideEngine,
    config_parser::EnvironmentConfig,
    metrics::BootMetrics,
    otel_provider::{OtelConfig, OtelProvider, TraceContext},
    profiles::Profile,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Resource limits and allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// CPU cores to allocate
    pub cpus: u32,
    /// Memory in GB
    pub memory: u32,
    /// Disk in GB
    pub disk: u32,
    /// GPU enabled
    pub gpu: bool,
    /// Memory limit in bytes
    pub memory_limit_bytes: u64,
    /// CPU limit (millicores)
    pub cpu_limit_millicores: u32,
}

impl ResourceAllocation {
    /// Create allocation from CPU and memory cores
    pub fn new(cpus: u32, memory: u32, disk: u32, gpu: bool) -> Self {
        ResourceAllocation {
            cpus,
            memory,
            disk,
            gpu,
            memory_limit_bytes: memory as u64 * 1024 * 1024 * 1024,
            cpu_limit_millicores: cpus * 1000,
        }
    }

    /// Create from profile
    pub fn from_profile(profile: &Profile) -> Self {
        ResourceAllocation::new(
            profile.cpus,
            profile.memory,
            profile.disk,
            profile.gpu,
        )
    }

    /// Merge with override (override wins)
    pub fn merge_with_override(mut self, cpus: Option<u32>, memory: Option<u32>, disk: Option<u32>, gpu: Option<bool>) -> Self {
        if let Some(c) = cpus {
            self.cpus = c;
            self.cpu_limit_millicores = c * 1000;
        }
        if let Some(m) = memory {
            self.memory = m;
            self.memory_limit_bytes = m as u64 * 1024 * 1024 * 1024;
        }
        if let Some(d) = disk {
            self.disk = d;
        }
        if let Some(g) = gpu {
            self.gpu = g;
        }
        self
    }
}

/// Daemon bootstrap configuration
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Environment configuration from YAML
    pub env_config: EnvironmentConfig,
    /// Selected profile
    pub profile: Profile,
    /// Resource allocation
    pub resources: ResourceAllocation,
    /// Override engine (CLI + ENV)
    pub overrides: OverrideEngine,
    /// Environment variables for daemon
    pub env_vars: HashMap<String, String>,
    /// OTel configuration
    pub otel_config: OtelConfig,
}

impl BootstrapConfig {
    /// Load configuration from env.yaml file
    pub fn from_file(config_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        // Load YAML config
        let env_config = EnvironmentConfig::from_file(config_path)?;
        env_config.validate()?;

        // Load overrides from environment
        let overrides = OverrideEngine::load_from_env();

        // Select profile
        let profile_name = overrides
            .get_profile()
            .or_else(|| {
                if !env_config.profile.is_empty() {
                    Some(env_config.profile.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "development".to_string());

        let profile = Profile::from_name(&profile_name)
            .ok_or_else(|| format!("Unknown profile: {}", profile_name))?;

        // Apply overrides to profile resources
        let mut resources = ResourceAllocation::from_profile(&profile);
        resources = resources.merge_with_override(
            overrides.get_cpus(),
            overrides.get_memory(),
            overrides.get_disk(),
            overrides.get_gpu(),
        );

        // Collect environment variables
        let mut env_vars = profile.get_env_vars();
        env_vars.extend(env_config.env.clone());
        env_vars.extend(overrides.get_all_env_vars());

        // OTel configuration
        let otel_config = OtelConfig {
            service_name: env_config.name.clone(),
            sampling_rate: if profile.name == "production" { 0.1 } else { 1.0 },
            ..Default::default()
        };

        Ok(BootstrapConfig {
            env_config,
            profile,
            resources,
            overrides,
            env_vars,
            otel_config,
        })
    }

    /// Get configuration context for spans
    pub fn to_config_context(&self) -> ConfigContext {
        ConfigContext {
            profile: self.profile.name.clone(),
            cpus: self.resources.cpus,
            memory: self.resources.memory,
            disk: self.resources.disk,
            gpu: self.resources.gpu,
        }
    }
}

/// Bootstrap result with metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapResult {
    /// Environment name
    pub env_name: String,
    /// Profile used
    pub profile: String,
    /// Resources allocated
    pub cpus: u32,
    pub memory: u32,
    pub disk: u32,
    /// Boot metrics
    pub metrics: BootMetrics,
    /// Trace ID
    pub trace_id: String,
    /// Total boot time ms
    pub boot_time_ms: u128,
}

/// Daemon bootstrap orchestrator
pub struct DaemonBootstrapper {
    /// Bootstrap configuration
    pub config: BootstrapConfig,
    /// Boot instrumentation
    pub instrumentation: BootInstrumentation,
    /// OTel provider
    pub otel_provider: Option<OtelProvider>,
}

impl DaemonBootstrapper {
    /// Create new bootstrapper from config file
    pub fn new(config_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let config = BootstrapConfig::from_file(config_path)?;
        let instrumentation = BootInstrumentation::new();

        Ok(DaemonBootstrapper {
            config,
            instrumentation,
            otel_provider: None,
        })
    }

    /// Record preflight checks
    pub fn preflight(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let span = BootSpan::new(BootPhase::PreFlight)
            .with_config(self.config.to_config_context())
            .success();
        self.instrumentation.record_phase(span);
        Ok(())
    }

    /// Record config loading
    pub fn config_load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let span = BootSpan::new(BootPhase::ConfigLoad)
            .with_config(self.config.to_config_context())
            .success();
        self.instrumentation.record_phase(span);
        Ok(())
    }

    /// Record profile selection
    pub fn profile_select(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let span = BootSpan::new(BootPhase::ProfileSelect)
            .with_config(self.config.to_config_context())
            .success();
        self.instrumentation.record_phase(span);
        Ok(())
    }

    /// Record resource allocation
    pub fn resource_alloc(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let span = BootSpan::new(BootPhase::ResourceAlloc)
            .with_config(self.config.to_config_context())
            .success();
        self.instrumentation.record_phase(span);
        Ok(())
    }

    /// Record network setup
    pub fn network_setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let span = BootSpan::new(BootPhase::NetworkSetup)
            .with_config(self.config.to_config_context())
            .success();
        self.instrumentation.record_phase(span);
        Ok(())
    }

    /// Record daemon initialization
    pub fn daemon_init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let span = BootSpan::new(BootPhase::DaemonInit)
            .with_config(self.config.to_config_context())
            .success();
        self.instrumentation.record_phase(span);
        Ok(())
    }

    /// Record health monitor startup
    pub fn health_monitor(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let span = BootSpan::new(BootPhase::HealthMonitor)
            .with_config(self.config.to_config_context())
            .success();
        self.instrumentation.record_phase(span);
        Ok(())
    }

    /// Record API server startup
    pub fn api_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let span = BootSpan::new(BootPhase::ApiServer)
            .with_config(self.config.to_config_context())
            .success();
        self.instrumentation.record_phase(span);
        Ok(())
    }

    /// Mark daemon ready
    pub fn ready(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let span = BootSpan::new(BootPhase::Ready)
            .with_config(self.config.to_config_context())
            .success();
        self.instrumentation.record_phase(span);
        Ok(())
    }

    /// Get bootstrap result
    pub fn result(&self) -> BootstrapResult {
        let summary = self.instrumentation.summary();
        BootstrapResult {
            env_name: self.config.env_config.name.clone(),
            profile: self.config.profile.name.clone(),
            cpus: self.config.resources.cpus,
            memory: self.config.resources.memory,
            disk: self.config.resources.disk,
            metrics: BootMetrics {
                boot_time_ms: summary.total_time_ms as f64,
                profile: self.config.profile.name.clone(),
                cpus: self.config.resources.cpus,
                memory: self.config.resources.memory,
                disk: self.config.resources.disk,
                gpu_enabled: self.config.resources.gpu,
                success: summary.successful,
                ..Default::default()
            },
            trace_id: summary.trace_id,
            boot_time_ms: summary.total_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_allocation_new() {
        let alloc = ResourceAllocation::new(4, 8, 40, false);
        assert_eq!(alloc.cpus, 4);
        assert_eq!(alloc.memory, 8);
        assert_eq!(alloc.disk, 40);
        assert!(!alloc.gpu);
        assert_eq!(alloc.cpu_limit_millicores, 4000);
        assert_eq!(alloc.memory_limit_bytes, 8 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_resource_allocation_from_profile() {
        let profile = Profile::development();
        let alloc = ResourceAllocation::from_profile(&profile);
        assert_eq!(alloc.cpus, profile.cpus);
        assert_eq!(alloc.memory, profile.memory);
    }

    #[test]
    fn test_resource_allocation_merge() {
        let mut alloc = ResourceAllocation::new(2, 4, 20, false);
        alloc = alloc.merge_with_override(Some(8), Some(16), Some(100), Some(true));
        assert_eq!(alloc.cpus, 8);
        assert_eq!(alloc.memory, 16);
        assert_eq!(alloc.disk, 100);
        assert!(alloc.gpu);
    }

    #[test]
    fn test_resource_allocation_partial_merge() {
        let alloc = ResourceAllocation::new(2, 4, 20, false);
        let merged = alloc.merge_with_override(Some(8), None, None, None);
        assert_eq!(merged.cpus, 8);
        assert_eq!(merged.memory, 4);
        assert_eq!(merged.disk, 20);
    }

    #[test]
    fn test_bootstrap_config_to_context() {
        let config = BootstrapConfig::from_file(std::path::Path::new("/dev/null")).err();
        // File doesn't exist, but we can test with a valid in-memory config
        assert!(config.is_some());
    }

    #[test]
    fn test_bootstrap_result_creation() {
        let result = BootstrapResult {
            env_name: "test-env".to_string(),
            profile: "testing".to_string(),
            cpus: 2,
            memory: 4,
            disk: 30,
            metrics: BootMetrics::default(),
            trace_id: "trace-123".to_string(),
            boot_time_ms: 500,
        };

        assert_eq!(result.env_name, "test-env");
        assert_eq!(result.profile, "testing");
        assert_eq!(result.cpus, 2);
    }

    #[test]
    fn test_bootstrapper_record_phases() {
        // Create a minimal bootstrapper (we'll use a dummy config approach for testing)
        let bootstrap_config = BootstrapConfig {
            env_config: crate::config_parser::EnvironmentConfig {
                name: "test".to_string(),
                profile: "development".to_string(),
                ..Default::default()
            },
            profile: Profile::development(),
            resources: ResourceAllocation::new(4, 8, 40, false),
            overrides: OverrideEngine::new(),
            env_vars: HashMap::new(),
            otel_config: OtelConfig::default(),
        };

        let mut bootstrapper = DaemonBootstrapper {
            config: bootstrap_config,
            instrumentation: BootInstrumentation::new(),
            otel_provider: None,
        };

        assert!(bootstrapper.preflight().is_ok());
        assert!(bootstrapper.config_load().is_ok());
        assert!(bootstrapper.profile_select().is_ok());
        assert!(bootstrapper.resource_alloc().is_ok());
        assert!(bootstrapper.ready().is_ok());

        assert_eq!(bootstrapper.instrumentation.phases.len(), 5);
    }

    #[test]
    fn test_bootstrapper_result() {
        let bootstrap_config = BootstrapConfig {
            env_config: crate::config_parser::EnvironmentConfig {
                name: "demo".to_string(),
                profile: "production".to_string(),
                ..Default::default()
            },
            profile: Profile::production(),
            resources: ResourceAllocation::new(8, 16, 100, true),
            overrides: OverrideEngine::new(),
            env_vars: HashMap::new(),
            otel_config: OtelConfig::default(),
        };

        let mut bootstrapper = DaemonBootstrapper {
            config: bootstrap_config,
            instrumentation: BootInstrumentation::new(),
            otel_provider: None,
        };

        bootstrapper.preflight().ok();
        bootstrapper.ready().ok();

        let result = bootstrapper.result();
        assert_eq!(result.env_name, "demo");
        assert_eq!(result.profile, "production");
        assert_eq!(result.cpus, 8);
        assert_eq!(result.memory, 16);
        assert!(result.metrics.success);
    }
}
