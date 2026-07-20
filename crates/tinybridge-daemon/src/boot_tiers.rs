use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct BootTier {
    pub tier: u8,
    pub name: String,
    pub description: String,
    pub timeout_ms: u64,
    pub critical: bool,
    pub start_delay_ms: u64,
    pub start_type: BootStartType,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BootStartType {
    Immediate,
    Idle,
    SocketActivation,
    OnDemand,
}

#[derive(Debug, Clone)]
pub struct BootTierConfig {
    pub strategy: String,
    pub tiers: HashMap<u8, BootTier>,
}

impl Default for BootTierConfig {
    fn default() -> Self {
        let mut tiers = HashMap::new();

        // Tier 1: Critical path (SSH ready)
        tiers.insert(
            1,
            BootTier {
                tier: 1,
                name: "critical".to_string(),
                description: "SSH ready (kernel, VirtioFS, sshd)".to_string(),
                timeout_ms: 2000,
                critical: true,
                start_delay_ms: 0,
                start_type: BootStartType::Immediate,
                services: vec!["sshd.service".to_string()],
            },
        );

        // Tier 2: Background services (DNS, logging, manager)
        tiers.insert(
            2,
            BootTier {
                tier: 2,
                name: "background".to_string(),
                description: "Background services (resolved, logind, journald)".to_string(),
                timeout_ms: 5000,
                critical: false,
                start_delay_ms: 1500,
                start_type: BootStartType::Idle,
                services: vec![
                    "systemd-resolved.service".to_string(),
                    "systemd-timesyncd.service".to_string(),
                    "systemd-logind.service".to_string(),
                ],
            },
        );

        // Tier 3: Eventual system services (caches, monitoring, tools)
        tiers.insert(
            3,
            BootTier {
                tier: 3,
                name: "eventual".to_string(),
                description: "Deferred services (package caches, monitoring, tools)".to_string(),
                timeout_ms: 120000,
                critical: false,
                start_delay_ms: 5000,
                start_type: BootStartType::Idle,
                services: vec![
                    "apt-daily.service".to_string(),
                    "apt-daily-upgrade.service".to_string(),
                ],
            },
        );

        // Tier 4: On-demand only
        tiers.insert(
            4,
            BootTier {
                tier: 4,
                name: "on-demand".to_string(),
                description: "Optional services (Bluetooth, printing, GUI)".to_string(),
                timeout_ms: 0, // No timeout, on-demand
                critical: false,
                start_delay_ms: 0,
                start_type: BootStartType::OnDemand,
                services: vec![
                    "bluetooth.service".to_string(),
                    "cups.service".to_string(),
                    "avahi-daemon.service".to_string(),
                ],
            },
        );

        BootTierConfig {
            strategy: "multi-tier-lazy".to_string(),
            tiers,
        }
    }
}

impl BootTierConfig {
    pub fn tier(&self, tier_num: u8) -> Option<&BootTier> {
        self.tiers.get(&tier_num)
    }

    pub fn is_critical_complete(&self, tier_num: u8) -> bool {
        self.tier(tier_num)
            .map(|t| !t.critical || t.tier <= 1)
            .unwrap_or(false)
    }

    pub fn timeout_for_tier(&self, tier_num: u8) -> Option<Duration> {
        self.tier(tier_num).map(|t| Duration::from_millis(t.timeout_ms))
    }

    pub fn wait_before_tier(&self, tier_num: u8) -> Option<Duration> {
        self.tier(tier_num).map(|t| Duration::from_millis(t.start_delay_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BootTierConfig::default();
        assert_eq!(config.strategy, "multi-tier-lazy");
        assert_eq!(config.tiers.len(), 4);
    }

    #[test]
    fn test_tier_1_critical() {
        let config = BootTierConfig::default();
        let tier1 = config.tier(1).unwrap();
        assert!(tier1.critical);
        assert_eq!(tier1.timeout_ms, 2000);
    }

    #[test]
    fn test_tier_2_idle() {
        let config = BootTierConfig::default();
        let tier2 = config.tier(2).unwrap();
        assert!(!tier2.critical);
        assert_eq!(tier2.start_delay_ms, 1500);
        matches!(tier2.start_type, BootStartType::Idle);
    }

    #[test]
    fn test_tier_3_eventual() {
        let config = BootTierConfig::default();
        let tier3 = config.tier(3).unwrap();
        assert!(!tier3.critical);
        assert_eq!(tier3.timeout_ms, 120000);
    }

    #[test]
    fn test_tier_4_ondemand() {
        let config = BootTierConfig::default();
        let tier4 = config.tier(4).unwrap();
        assert!(!tier4.critical);
        assert_eq!(tier4.start_type, BootStartType::OnDemand);
    }

    #[test]
    fn test_wait_times() {
        let config = BootTierConfig::default();
        assert_eq!(config.wait_before_tier(1).unwrap().as_millis(), 0);
        assert_eq!(config.wait_before_tier(2).unwrap().as_millis(), 1500);
        assert_eq!(config.wait_before_tier(3).unwrap().as_millis(), 5000);
    }
}
