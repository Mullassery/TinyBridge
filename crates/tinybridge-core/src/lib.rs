pub mod boot_diagnostics;
pub mod boot_instrumentation;
pub mod boot_integration;
pub mod boot_recovery;
pub mod boot_stages;
pub mod config;
pub mod config_overrides;
pub mod config_parser;
pub mod daemon_bootstrap;
pub mod dds;
pub mod environment;
pub mod error;
pub mod ipc;
pub mod lazy_loader;
pub mod metrics;
pub mod migration;
pub mod otel_provider;
pub mod platform_abstraction;
pub mod platform_registry;
pub mod profiles;
pub mod resource_enforcer;
pub mod ssh_keys;
pub mod windows_adapter;

pub use boot_diagnostics::{BootDiagnosticReport, BootPerformanceMetrics, ComponentHealth, HealthCheckEngine, HealthReport, HealthStatus};
pub use boot_instrumentation::{BootInstrumentation, BootPhase, BootSpan, BootSummary, ConfigContext, SpanStatus};
pub use boot_integration::{BootIntegrationTester, BootReadinessValidator, BootScenario, BootTestResult};
pub use boot_recovery::{BootFailure, BootFailureType, BootRecoveryHandler, DegradationPolicy, RecoveryStrategy, RecoverySummary};
pub use platform_abstraction::{HostPlatform, HypervisorBackend, NetworkMode, PlatformAdapter, PlatformCapabilities, PlatformInfo, StorageMount, VMResourceConfig};
pub use platform_registry::{PlatformRegistry, RegistryStatus};
pub use windows_adapter::{WindowsHyperVAdapter, WindowsVMMetadata};
pub use boot_stages::{BootReadiness, BootTier, BootTimeline, BootTimelineSummary, TierFeatures};
pub use config::{DefaultResources, TinyBridgeConfig};
pub use config_overrides::{CliOverrides, EnvOverrides, OverrideEngine};
pub use config_parser::{ConfigError, ConfigOverrides, EnvironmentConfig, NetworkConfig, PortMapping, ResourceSpec, VolumeMount};
pub use daemon_bootstrap::{BootstrapConfig, BootstrapResult, DaemonBootstrapper, ResourceAllocation};
pub use lazy_loader::{Loadable, LazyLoadScheduler, LoadState, LoaderSummary};
pub use metrics::{BootMetrics, MetricType, MetricValue, MetricsRegistry, ResourceMetrics};
pub use otel_provider::{OtelConfig, OtelProvider, TraceContext};
pub use profiles::Profile;
pub use resource_enforcer::{CpuLimit, DiskLimit, MemoryLimit, NetworkLimit, ResourcePercentages, ResourcePolicy, ResourceUsage};
pub use dds::{
    DdsAuditEvent, DdsConfig, DdsEventType, DdsFeatures, DdsNetworkingConfig, DdsProfile,
    DdsSecurityConfig,
};
pub use environment::{
    Arch, EnvMetadata, EnvYaml, Environment, EnvironmentStatus, NativeSection, NativeToolSpec,
    Resources, SubstrateConfig,
};
pub use error::{CoreError, Result};
pub use ipc::{
    error_codes, methods, DownResponse, EnvironmentSummary, JsonRpcError, JsonRpcRequest,
    JsonRpcResponse, ListResponse, ShellResponse, StatusResponse, UpResponse,
};
