/// Boot Error Recovery & Resilience
/// Phase 5: Production Hardening
///
/// Error recovery strategies, retry logic, and graceful degradation

use crate::boot_stages::BootTier;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Recovery strategy for boot failures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Retry failed operation
    Retry,
    /// Skip optional component, continue
    Skip,
    /// Downgrade to lower tier
    Downgrade,
    /// Abort boot entirely
    Abort,
}

impl std::fmt::Display for RecoveryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryStrategy::Retry => write!(f, "retry"),
            RecoveryStrategy::Skip => write!(f, "skip"),
            RecoveryStrategy::Downgrade => write!(f, "downgrade"),
            RecoveryStrategy::Abort => write!(f, "abort"),
        }
    }
}

/// Boot failure type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BootFailureType {
    /// Timeout waiting for tier
    Timeout {
        tier: BootTier,
        target_ms: u64,
        actual_ms: u64,
    },
    /// Component initialization failed
    ComponentFailure {
        component: String,
        error: String,
    },
    /// Dependency not available
    DependencyFailure {
        component: String,
        dependency: String,
    },
    /// Resource exhaustion
    ResourceExhaustion {
        resource: String,
        available: u64,
        required: u64,
    },
    /// Configuration error
    ConfigError {
        error: String,
    },
    /// Network error
    NetworkError {
        error: String,
    },
}

impl std::fmt::Display for BootFailureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootFailureType::Timeout { tier, actual_ms, target_ms } => {
                write!(f, "Timeout on {:?}: {}ms > {}ms target", tier, actual_ms, target_ms)
            }
            BootFailureType::ComponentFailure { component, error } => {
                write!(f, "Component {} failed: {}", component, error)
            }
            BootFailureType::DependencyFailure { component, dependency } => {
                write!(f, "Component {} depends on {}", component, dependency)
            }
            BootFailureType::ResourceExhaustion { resource, available, required } => {
                write!(f, "{} exhausted: {} available, {} required", resource, available, required)
            }
            BootFailureType::ConfigError { error } => write!(f, "Config error: {}", error),
            BootFailureType::NetworkError { error } => write!(f, "Network error: {}", error),
        }
    }
}

/// Boot failure record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootFailure {
    /// Failure type
    pub failure_type: BootFailureType,
    /// Tier where failure occurred
    pub tier: BootTier,
    /// When it occurred (ms since boot start)
    pub time_ms: u64,
    /// Retry count so far
    pub retry_count: u32,
    /// Max retries allowed
    pub max_retries: u32,
}

impl BootFailure {
    /// Create new boot failure
    pub fn new(failure_type: BootFailureType, tier: BootTier, time_ms: u64) -> Self {
        BootFailure {
            failure_type,
            tier,
            time_ms,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// Determine recovery strategy for this failure
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match &self.failure_type {
            // Timeouts on critical tier → downgrade
            BootFailureType::Timeout { tier, .. } if *tier == BootTier::Ssh => {
                RecoveryStrategy::Abort
            }
            // Timeout on non-critical tier → skip optional services
            BootFailureType::Timeout { .. } => RecoveryStrategy::Skip,
            // Component failure → retry once
            BootFailureType::ComponentFailure { component, .. } => {
                if self.retry_count < self.max_retries && !component.contains("critical") {
                    RecoveryStrategy::Retry
                } else {
                    RecoveryStrategy::Skip
                }
            }
            // Dependency failure → downgrade
            BootFailureType::DependencyFailure { .. } => RecoveryStrategy::Downgrade,
            // Resource exhaustion → downgrade
            BootFailureType::ResourceExhaustion { .. } => RecoveryStrategy::Downgrade,
            // Config/network errors → retry
            BootFailureType::ConfigError { .. } | BootFailureType::NetworkError { .. } => {
                if self.retry_count < self.max_retries {
                    RecoveryStrategy::Retry
                } else {
                    RecoveryStrategy::Abort
                }
            }
        }
    }

    /// Check if can retry
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Suggest next action
    pub fn next_action(&self) -> String {
        match self.recovery_strategy() {
            RecoveryStrategy::Retry => format!("Retry (attempt {})", self.retry_count + 1),
            RecoveryStrategy::Skip => "Skip optional services".to_string(),
            RecoveryStrategy::Downgrade => "Downgrade to previous tier".to_string(),
            RecoveryStrategy::Abort => "Abort boot".to_string(),
        }
    }
}

/// Boot recovery handler
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BootRecoveryHandler {
    /// Failures encountered
    pub failures: Vec<BootFailure>,
    /// Current recovery state
    pub current_tier: Option<BootTier>,
    /// Recovery attempts
    pub recovery_attempts: u32,
}

impl BootRecoveryHandler {
    /// Create new recovery handler
    pub fn new() -> Self {
        BootRecoveryHandler {
            failures: Vec::new(),
            current_tier: None,
            recovery_attempts: 0,
        }
    }

    /// Record a failure
    pub fn record_failure(&mut self, failure: BootFailure) {
        self.failures.push(failure);
    }

    /// Get last failure
    pub fn last_failure(&self) -> Option<&BootFailure> {
        self.failures.last()
    }

    /// Get failures at tier
    pub fn failures_at_tier(&self, tier: BootTier) -> Vec<&BootFailure> {
        self.failures.iter().filter(|f| f.tier == tier).collect()
    }

    /// Count failures
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Get recovery summary
    pub fn summary(&self) -> RecoverySummary {
        let critical_failures = self
            .failures
            .iter()
            .filter(|f| {
                matches!(
                    f.failure_type,
                    BootFailureType::Timeout { tier: BootTier::Ssh, .. }
                        | BootFailureType::ConfigError { .. }
                )
            })
            .count();

        let recovered_failures = self
            .failures
            .iter()
            .filter(|f| {
                matches!(
                    f.recovery_strategy(),
                    RecoveryStrategy::Retry | RecoveryStrategy::Skip
                )
            })
            .count();

        RecoverySummary {
            total_failures: self.failures.len(),
            critical_failures,
            recovered_failures,
            recovery_attempts: self.recovery_attempts,
            is_recoverable: critical_failures == 0,
        }
    }

    /// Check if recoverable
    pub fn is_recoverable(&self) -> bool {
        self.summary().is_recoverable
    }
}

/// Recovery summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySummary {
    /// Total failures
    pub total_failures: usize,
    /// Critical failures (unrecoverable)
    pub critical_failures: usize,
    /// Failures that were recovered
    pub recovered_failures: usize,
    /// Number of recovery attempts made
    pub recovery_attempts: u32,
    /// Whether boot is recoverable
    pub is_recoverable: bool,
}

/// Graceful degradation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationPolicy {
    /// Allow downgrade to lower tier
    pub allow_downgrade: bool,
    /// Allow skipping optional components
    pub allow_skip: bool,
    /// Min tier required for operation (can't go lower)
    pub min_tier: BootTier,
    /// Max retries per failure
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for DegradationPolicy {
    fn default() -> Self {
        DegradationPolicy {
            allow_downgrade: true,
            allow_skip: true,
            min_tier: BootTier::Ssh,
            max_retries: 3,
            retry_delay_ms: 500,
        }
    }
}

impl DegradationPolicy {
    /// Production policy: strict, minimal retries
    pub fn production() -> Self {
        DegradationPolicy {
            allow_downgrade: false,
            allow_skip: false,
            min_tier: BootTier::Usable,
            max_retries: 1,
            retry_delay_ms: 1000,
        }
    }

    /// Development policy: lenient, many retries
    pub fn development() -> Self {
        DegradationPolicy {
            allow_downgrade: true,
            allow_skip: true,
            min_tier: BootTier::Ssh,
            max_retries: 5,
            retry_delay_ms: 200,
        }
    }

    /// Validate policy
    pub fn validate(&self) -> Result<(), String> {
        if self.max_retries == 0 {
            return Err("max_retries must be > 0".to_string());
        }
        Ok(())
    }

    /// Get retry delay as Duration
    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.retry_delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_strategy_timeout_ssh() {
        let failure = BootFailure::new(
            BootFailureType::Timeout {
                tier: BootTier::Ssh,
                target_ms: 1500,
                actual_ms: 2000,
            },
            BootTier::Ssh,
            2000,
        );
        assert_eq!(failure.recovery_strategy(), RecoveryStrategy::Abort);
    }

    #[test]
    fn test_recovery_strategy_timeout_api() {
        let failure = BootFailure::new(
            BootFailureType::Timeout {
                tier: BootTier::Api,
                target_ms: 30000,
                actual_ms: 35000,
            },
            BootTier::Api,
            35000,
        );
        assert_eq!(failure.recovery_strategy(), RecoveryStrategy::Skip);
    }

    #[test]
    fn test_recovery_strategy_component_failure() {
        let failure = BootFailure::new(
            BootFailureType::ComponentFailure {
                component: "health_monitor".to_string(),
                error: "connection refused".to_string(),
            },
            BootTier::Usable,
            5000,
        );
        assert_eq!(failure.recovery_strategy(), RecoveryStrategy::Retry);
    }

    #[test]
    fn test_recovery_strategy_dependency_failure() {
        let failure = BootFailure::new(
            BootFailureType::DependencyFailure {
                component: "api_server".to_string(),
                dependency: "health_monitor".to_string(),
            },
            BootTier::Api,
            30000,
        );
        assert_eq!(failure.recovery_strategy(), RecoveryStrategy::Downgrade);
    }

    #[test]
    fn test_boot_failure_can_retry() {
        let mut failure = BootFailure::new(
            BootFailureType::ComponentFailure {
                component: "test".to_string(),
                error: "error".to_string(),
            },
            BootTier::Usable,
            1000,
        );
        assert!(failure.can_retry());
        failure.increment_retry();
        failure.increment_retry();
        failure.increment_retry();
        assert!(!failure.can_retry());
    }

    #[test]
    fn test_boot_failure_next_action() {
        let failure = BootFailure::new(
            BootFailureType::ComponentFailure {
                component: "test".to_string(),
                error: "error".to_string(),
            },
            BootTier::Usable,
            1000,
        );
        assert!(failure.next_action().contains("Retry"));
    }

    #[test]
    fn test_boot_recovery_handler() {
        let mut handler = BootRecoveryHandler::new();
        let failure = BootFailure::new(
            BootFailureType::ComponentFailure {
                component: "test".to_string(),
                error: "error".to_string(),
            },
            BootTier::Usable,
            1000,
        );
        handler.record_failure(failure);
        assert_eq!(handler.failure_count(), 1);
        assert!(handler.last_failure().is_some());
    }

    #[test]
    fn test_boot_recovery_handler_summary() {
        let mut handler = BootRecoveryHandler::new();
        handler.record_failure(BootFailure::new(
            BootFailureType::ComponentFailure {
                component: "test".to_string(),
                error: "error".to_string(),
            },
            BootTier::Usable,
            1000,
        ));
        let summary = handler.summary();
        assert_eq!(summary.total_failures, 1);
        assert!(summary.is_recoverable);
    }

    #[test]
    fn test_degradation_policy_production() {
        let policy = DegradationPolicy::production();
        assert!(!policy.allow_downgrade);
        assert!(!policy.allow_skip);
        assert_eq!(policy.max_retries, 1);
    }

    #[test]
    fn test_degradation_policy_development() {
        let policy = DegradationPolicy::development();
        assert!(policy.allow_downgrade);
        assert!(policy.allow_skip);
        assert_eq!(policy.max_retries, 5);
    }

    #[test]
    fn test_degradation_policy_validate() {
        let mut policy = DegradationPolicy::default();
        assert!(policy.validate().is_ok());
        policy.max_retries = 0;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_boot_failure_type_display() {
        let failure = BootFailureType::Timeout {
            tier: BootTier::Usable,
            target_ms: 5000,
            actual_ms: 6000,
        };
        assert!(failure.to_string().contains("Timeout"));
    }

    #[test]
    fn test_recovery_strategy_display() {
        assert_eq!(RecoveryStrategy::Retry.to_string(), "retry");
        assert_eq!(RecoveryStrategy::Skip.to_string(), "skip");
        assert_eq!(RecoveryStrategy::Downgrade.to_string(), "downgrade");
        assert_eq!(RecoveryStrategy::Abort.to_string(), "abort");
    }

    #[test]
    fn test_failures_at_tier() {
        let mut handler = BootRecoveryHandler::new();
        handler.record_failure(BootFailure::new(
            BootFailureType::ComponentFailure {
                component: "test1".to_string(),
                error: "error".to_string(),
            },
            BootTier::Usable,
            1000,
        ));
        handler.record_failure(BootFailure::new(
            BootFailureType::ComponentFailure {
                component: "test2".to_string(),
                error: "error".to_string(),
            },
            BootTier::Api,
            30000,
        ));
        assert_eq!(handler.failures_at_tier(BootTier::Usable).len(), 1);
        assert_eq!(handler.failures_at_tier(BootTier::Api).len(), 1);
    }

    #[test]
    fn test_critical_failures_detection() {
        let mut handler = BootRecoveryHandler::new();
        handler.record_failure(BootFailure::new(
            BootFailureType::Timeout {
                tier: BootTier::Ssh,
                target_ms: 1500,
                actual_ms: 2000,
            },
            BootTier::Ssh,
            2000,
        ));
        let summary = handler.summary();
        assert_eq!(summary.critical_failures, 1);
        assert!(!summary.is_recoverable);
    }
}
