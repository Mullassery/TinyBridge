/// Multi-Tier Boot Stages
/// Phase 4.0.4: Boot Optimization
///
/// Define boot tiers with specific features and timing targets

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Boot tier levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BootTier {
    /// Tier 1: SSH tunnel ready (1.5s target)
    Ssh = 1,
    /// Tier 2: Daemon usable (5s target)
    Usable = 2,
    /// Tier 3: API server ready (30s target)
    Api = 3,
    /// Tier 4: Full features (120s target)
    Full = 4,
}

impl std::fmt::Display for BootTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootTier::Ssh => write!(f, "ssh"),
            BootTier::Usable => write!(f, "usable"),
            BootTier::Api => write!(f, "api"),
            BootTier::Full => write!(f, "full"),
        }
    }
}

impl BootTier {
    /// Get target time for this tier in milliseconds
    pub fn target_time_ms(&self) -> u64 {
        match self {
            BootTier::Ssh => 1500,
            BootTier::Usable => 5000,
            BootTier::Api => 30000,
            BootTier::Full => 120000,
        }
    }

    /// Get target as Duration
    pub fn target_duration(&self) -> Duration {
        Duration::from_millis(self.target_time_ms())
    }

    /// Check if boot time meets target
    pub fn meets_target(&self, actual_ms: u64) -> bool {
        actual_ms <= self.target_time_ms()
    }

    /// Get slack time (target - actual)
    pub fn slack_ms(&self, actual_ms: u64) -> i64 {
        self.target_time_ms() as i64 - actual_ms as i64
    }
}

/// Feature availability at each tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierFeatures {
    /// Features available at this tier
    pub features: Vec<String>,
    /// Services that must be running
    pub required_services: Vec<String>,
    /// Optional services (deferred)
    pub optional_services: Vec<String>,
    /// Modules loaded in memory
    pub loaded_modules: Vec<String>,
}

impl TierFeatures {
    /// Tier 1: SSH (1.5s)
    /// Only essential: config loading, networking setup
    pub fn ssh() -> Self {
        TierFeatures {
            features: vec![
                "config_load".to_string(),
                "profile_select".to_string(),
                "network_setup".to_string(),
                "ssh_tunnel".to_string(),
            ],
            required_services: vec![
                "config_service".to_string(),
                "network_service".to_string(),
            ],
            optional_services: vec![
                "health_monitor".to_string(),
                "metrics_service".to_string(),
                "api_server".to_string(),
            ],
            loaded_modules: vec![
                "config".to_string(),
                "network".to_string(),
                "ssh".to_string(),
            ],
        }
    }

    /// Tier 2: Usable (5s)
    /// Add health monitoring: basic operations possible
    pub fn usable() -> Self {
        let mut features = Self::ssh();
        features.features.extend(vec![
            "health_check".to_string(),
            "resource_alloc".to_string(),
            "basic_operations".to_string(),
        ]);
        features.required_services.push("health_monitor".to_string());
        features.loaded_modules.extend(vec![
            "health".to_string(),
            "resources".to_string(),
        ]);
        features
    }

    /// Tier 3: API (30s)
    /// Add API server: full control plane available
    pub fn api() -> Self {
        let mut features = Self::usable();
        features.features.extend(vec![
            "api_server".to_string(),
            "json_rpc".to_string(),
            "command_execution".to_string(),
        ]);
        features.required_services.push("api_server".to_string());
        features.loaded_modules.extend(vec![
            "api".to_string(),
            "rpc".to_string(),
        ]);
        features
    }

    /// Tier 4: Full (120s)
    /// All services: metrics, telemetry, advanced features
    pub fn full() -> Self {
        let mut features = Self::api();
        features.features.extend(vec![
            "metrics_export".to_string(),
            "otel_tracing".to_string(),
            "logging".to_string(),
            "caching".to_string(),
            "advanced_scheduling".to_string(),
        ]);
        features.required_services.extend(vec![
            "metrics_service".to_string(),
            "otel_service".to_string(),
            "logging_service".to_string(),
        ]);
        features.optional_services.clear();
        features.loaded_modules.extend(vec![
            "metrics".to_string(),
            "otel".to_string(),
            "logging".to_string(),
            "cache".to_string(),
            "scheduler".to_string(),
        ]);
        features
    }

    /// Get features for a tier
    pub fn for_tier(tier: BootTier) -> Self {
        match tier {
            BootTier::Ssh => Self::ssh(),
            BootTier::Usable => Self::usable(),
            BootTier::Api => Self::api(),
            BootTier::Full => Self::full(),
        }
    }

    /// Check if all required services are available
    pub fn has_required_services(&self, available: &[String]) -> bool {
        self.required_services.iter().all(|svc| available.contains(svc))
    }

    /// Get services that need to be deferred
    pub fn deferred_services(&self) -> Vec<String> {
        self.optional_services.clone()
    }
}

/// Boot readiness status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootReadiness {
    /// Current tier reached
    pub current_tier: BootTier,
    /// Time to reach this tier (ms)
    pub time_to_tier_ms: u64,
    /// Met target for this tier
    pub met_target: bool,
    /// Slack time (target - actual)
    pub slack_ms: i64,
    /// Features available
    pub features: TierFeatures,
    /// Services running
    pub running_services: Vec<String>,
    /// Startup timestamp (unix ms)
    pub startup_ts_ms: u64,
}

impl BootReadiness {
    /// Create readiness at a tier
    pub fn new(tier: BootTier, time_to_tier_ms: u64, startup_ts_ms: u64) -> Self {
        let met_target = tier.meets_target(time_to_tier_ms);
        let slack_ms = tier.slack_ms(time_to_tier_ms);
        let features = TierFeatures::for_tier(tier);
        let running_services = features.required_services.clone();

        BootReadiness {
            current_tier: tier,
            time_to_tier_ms,
            met_target,
            slack_ms,
            features,
            running_services,
            startup_ts_ms,
        }
    }

    /// Advance to next tier
    pub fn advance(self, next_tier: BootTier, time_to_tier_ms: u64) -> Self {
        let met_target = next_tier.meets_target(time_to_tier_ms);
        let slack_ms = next_tier.slack_ms(time_to_tier_ms);
        let features = TierFeatures::for_tier(next_tier);
        let mut running_services = self.running_services;

        // Add new required services
        for svc in &features.required_services {
            if !running_services.contains(svc) {
                running_services.push(svc.clone());
            }
        }

        BootReadiness {
            current_tier: next_tier,
            time_to_tier_ms,
            met_target,
            slack_ms,
            features,
            running_services,
            startup_ts_ms: self.startup_ts_ms,
        }
    }

    /// Check if feature is available
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.features.contains(&feature.to_string())
    }

    /// Check if service is running
    pub fn has_service(&self, service: &str) -> bool {
        self.running_services.contains(&service.to_string())
    }

    /// Get readiness percentage (0-100)
    pub fn readiness_percent(&self) -> u32 {
        match self.current_tier {
            BootTier::Ssh => 25,
            BootTier::Usable => 50,
            BootTier::Api => 75,
            BootTier::Full => 100,
        }
    }
}

/// Boot stage timeline
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BootTimeline {
    /// Tier 1 time
    pub tier1_ms: Option<u64>,
    /// Tier 2 time
    pub tier2_ms: Option<u64>,
    /// Tier 3 time
    pub tier3_ms: Option<u64>,
    /// Tier 4 time
    pub tier4_ms: Option<u64>,
}

impl BootTimeline {
    /// Create timeline with all tiers
    pub fn full(t1: u64, t2: u64, t3: u64, t4: u64) -> Self {
        BootTimeline {
            tier1_ms: Some(t1),
            tier2_ms: Some(t2),
            tier3_ms: Some(t3),
            tier4_ms: Some(t4),
        }
    }

    /// Check all tiers met targets
    pub fn all_targets_met(&self) -> bool {
        if let (Some(t1), Some(t2), Some(t3), Some(t4)) = (self.tier1_ms, self.tier2_ms, self.tier3_ms, self.tier4_ms) {
            BootTier::Ssh.meets_target(t1)
                && BootTier::Usable.meets_target(t2)
                && BootTier::Api.meets_target(t3)
                && BootTier::Full.meets_target(t4)
        } else {
            false
        }
    }

    /// Get summary stats
    pub fn summary(&self) -> BootTimelineSummary {
        BootTimelineSummary {
            tier1_met: self.tier1_ms.map(|t| BootTier::Ssh.meets_target(t)),
            tier2_met: self.tier2_ms.map(|t| BootTier::Usable.meets_target(t)),
            tier3_met: self.tier3_ms.map(|t| BootTier::Api.meets_target(t)),
            tier4_met: self.tier4_ms.map(|t| BootTier::Full.meets_target(t)),
            total_time_ms: self.tier4_ms,
        }
    }
}

/// Boot timeline summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootTimelineSummary {
    /// Tier 1 met target
    pub tier1_met: Option<bool>,
    /// Tier 2 met target
    pub tier2_met: Option<bool>,
    /// Tier 3 met target
    pub tier3_met: Option<bool>,
    /// Tier 4 met target
    pub tier4_met: Option<bool>,
    /// Total boot time
    pub total_time_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_tier_targets() {
        assert_eq!(BootTier::Ssh.target_time_ms(), 1500);
        assert_eq!(BootTier::Usable.target_time_ms(), 5000);
        assert_eq!(BootTier::Api.target_time_ms(), 30000);
        assert_eq!(BootTier::Full.target_time_ms(), 120000);
    }

    #[test]
    fn test_boot_tier_meets_target() {
        assert!(BootTier::Ssh.meets_target(1000));
        assert!(!BootTier::Ssh.meets_target(2000));
        assert!(BootTier::Usable.meets_target(5000));
        assert!(!BootTier::Usable.meets_target(6000));
    }

    #[test]
    fn test_boot_tier_slack() {
        assert_eq!(BootTier::Ssh.slack_ms(1000), 500);
        assert_eq!(BootTier::Ssh.slack_ms(1500), 0);
        assert_eq!(BootTier::Usable.slack_ms(4000), 1000);
    }

    #[test]
    fn test_tier_features_ssh() {
        let features = TierFeatures::ssh();
        assert_eq!(features.features.len(), 4);
        assert_eq!(features.required_services.len(), 2);
        assert_eq!(features.optional_services.len(), 3);
    }

    #[test]
    fn test_tier_features_usable() {
        let features = TierFeatures::usable();
        assert!(features.features.contains(&"health_check".to_string()));
        assert!(features.required_services.contains(&"health_monitor".to_string()));
    }

    #[test]
    fn test_tier_features_api() {
        let features = TierFeatures::api();
        assert!(features.features.contains(&"api_server".to_string()));
        assert!(features.required_services.contains(&"api_server".to_string()));
    }

    #[test]
    fn test_tier_features_full() {
        let features = TierFeatures::full();
        assert!(features.features.contains(&"metrics_export".to_string()));
        assert!(features.features.contains(&"otel_tracing".to_string()));
        assert!(features.optional_services.is_empty());
    }

    #[test]
    fn test_boot_readiness_creation() {
        let readiness = BootReadiness::new(BootTier::Ssh, 1200, 100);
        assert_eq!(readiness.current_tier, BootTier::Ssh);
        assert_eq!(readiness.time_to_tier_ms, 1200);
        assert!(readiness.met_target);
        assert_eq!(readiness.slack_ms, 300);
    }

    #[test]
    fn test_boot_readiness_feature_check() {
        let readiness = BootReadiness::new(BootTier::Api, 25000, 100);
        assert!(readiness.has_feature("api_server"));
        assert!(!readiness.has_feature("metrics_export"));
    }

    #[test]
    fn test_boot_readiness_advance() {
        let readiness = BootReadiness::new(BootTier::Ssh, 1200, 100);
        let advanced = readiness.advance(BootTier::Usable, 4500);
        assert_eq!(advanced.current_tier, BootTier::Usable);
        assert!(advanced.met_target);
    }

    #[test]
    fn test_boot_readiness_percent() {
        assert_eq!(BootReadiness::new(BootTier::Ssh, 1000, 0).readiness_percent(), 25);
        assert_eq!(BootReadiness::new(BootTier::Usable, 4000, 0).readiness_percent(), 50);
        assert_eq!(BootReadiness::new(BootTier::Api, 25000, 0).readiness_percent(), 75);
        assert_eq!(BootReadiness::new(BootTier::Full, 100000, 0).readiness_percent(), 100);
    }

    #[test]
    fn test_boot_timeline() {
        let timeline = BootTimeline::full(1200, 4500, 25000, 100000);
        assert_eq!(timeline.tier1_ms, Some(1200));
        assert_eq!(timeline.tier4_ms, Some(100000));
        assert!(timeline.all_targets_met());
    }

    #[test]
    fn test_boot_timeline_partial() {
        let timeline = BootTimeline {
            tier1_ms: Some(1200),
            tier2_ms: Some(4500),
            tier3_ms: None,
            tier4_ms: None,
        };
        assert!(!timeline.all_targets_met());
    }

    #[test]
    fn test_boot_timeline_summary() {
        let timeline = BootTimeline::full(1200, 4500, 25000, 100000);
        let summary = timeline.summary();
        assert_eq!(summary.tier1_met, Some(true));
        assert_eq!(summary.tier2_met, Some(true));
        assert_eq!(summary.total_time_ms, Some(100000));
    }
}
