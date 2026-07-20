use crate::{BinaryFormat, ExecutionTier};
use serde::{Deserialize, Serialize};

/// A routing rule that determines which tier to execute a workload on
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule name for logging
    pub name: String,

    /// Match criteria
    pub criteria: RoutingCriteria,

    /// Target execution tier
    pub tier: ExecutionTier,

    /// Priority (higher = evaluated first)
    #[serde(default)]
    pub priority: i32,
}

/// Criteria for matching a workload to a routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoutingCriteria {
    /// Match by binary format
    BinaryFormat { format: String },

    /// Match by command name
    Command { name: String },

    /// Match by file extension
    Extension { extension: String },

    /// Match by environment capability
    Capability { name: String },

    /// Match by CPU/GPU requirements
    Requirements { gpu: Option<bool> },

    /// Composite rule (AND logic)
    All { rules: Vec<RoutingCriteria> },

    /// Composite rule (OR logic)
    Any { rules: Vec<RoutingCriteria> },
}

/// Routing rules engine
pub struct RulesEngine {
    rules: Vec<RoutingRule>,
}

impl RulesEngine {
    /// Create a new rules engine
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    /// Add a routing rule
    pub fn add_rule(&mut self, rule: RoutingRule) {
        self.rules.push(rule);
        // Sort by priority (highest first)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Add multiple rules
    pub fn add_rules(&mut self, rules: Vec<RoutingRule>) {
        for rule in rules {
            self.add_rule(rule);
        }
    }

    /// Create default rules for common tools
    pub fn with_defaults() -> Self {
        let mut engine = Self::new();

        // Linux-only tools (force Linux tier)
        let linux_tools = vec![
            "python", "python3", "node", "npm", "pip", "docker", "docker-compose",
            "systemctl", "apt-get", "dpkg", "rpm", "gcc", "make", "cmake",
        ];

        for tool in linux_tools {
            engine.add_rule(RoutingRule {
                name: format!("linux_tool_{}", tool),
                criteria: RoutingCriteria::Command {
                    name: tool.to_string(),
                },
                tier: ExecutionTier::Linux,
                priority: 100,
            });
        }

        // GPU-accelerated tools (use remote if available)
        let gpu_tools = vec!["cuda", "torch", "tensorflow", "pytorch"];
        for tool in gpu_tools {
            engine.add_rule(RoutingRule {
                name: format!("gpu_tool_{}", tool),
                criteria: RoutingCriteria::Command {
                    name: tool.to_string(),
                },
                tier: ExecutionTier::Remote,
                priority: 90,
            });
        }

        // Native tools (force native)
        let native_tools = vec!["swift", "xcode-select", "xcrun", "swiftc"];
        for tool in native_tools {
            engine.add_rule(RoutingRule {
                name: format!("native_tool_{}", tool),
                criteria: RoutingCriteria::Command {
                    name: tool.to_string(),
                },
                tier: ExecutionTier::Native,
                priority: 100,
            });
        }

        // Script files (prefer native for shell scripts)
        engine.add_rule(RoutingRule {
            name: "shell_script".to_string(),
            criteria: RoutingCriteria::Extension {
                extension: "sh".to_string(),
            },
            tier: ExecutionTier::Native,
            priority: 50,
        });

        // Python scripts (prefer Linux)
        engine.add_rule(RoutingRule {
            name: "python_script".to_string(),
            criteria: RoutingCriteria::Extension {
                extension: "py".to_string(),
            },
            tier: ExecutionTier::Linux,
            priority: 50,
        });

        engine
    }

    /// Find the best matching rule for a workload
    pub fn find_tier(
        &self,
        command: &str,
        format: BinaryFormat,
        has_gpu: bool,
    ) -> ExecutionTier {
        // Check each rule in priority order
        for rule in &self.rules {
            if self.matches_criteria(&rule.criteria, command, format, has_gpu) {
                return rule.tier;
            }
        }

        // Default routing: native > linux > remote
        match format {
            BinaryFormat::MachO => ExecutionTier::Native,
            BinaryFormat::Elf => ExecutionTier::Linux,
            BinaryFormat::Script => ExecutionTier::Native,
            BinaryFormat::Unknown => ExecutionTier::Linux,
        }
    }

    fn matches_criteria(
        &self,
        criteria: &RoutingCriteria,
        command: &str,
        format: BinaryFormat,
        has_gpu: bool,
    ) -> bool {
        match criteria {
            RoutingCriteria::BinaryFormat { format: fmt } => {
                fmt == &format.to_string()
            }
            RoutingCriteria::Command { name } => {
                command.ends_with(name) || command == name
            }
            RoutingCriteria::Extension { extension } => {
                command.ends_with(&format!(".{}", extension))
            }
            RoutingCriteria::Capability { .. } => {
                // Capabilities checked at environment level
                false
            }
            RoutingCriteria::Requirements { gpu } => {
                gpu.is_none() || gpu == &Some(has_gpu)
            }
            RoutingCriteria::All { rules } => {
                rules.iter().all(|r| self.matches_criteria(r, command, format, has_gpu))
            }
            RoutingCriteria::Any { rules } => {
                rules.iter().any(|r| self.matches_criteria(r, command, format, has_gpu))
            }
        }
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules_engine_defaults() {
        let engine = RulesEngine::with_defaults();
        assert!(!engine.rules.is_empty());
    }

    #[test]
    fn test_python_routing() {
        let engine = RulesEngine::with_defaults();
        let tier = engine.find_tier("python", BinaryFormat::Elf, false);
        assert_eq!(tier, ExecutionTier::Linux);
    }

    #[test]
    fn test_swift_routing() {
        let engine = RulesEngine::with_defaults();
        let tier = engine.find_tier("swift", BinaryFormat::MachO, false);
        assert_eq!(tier, ExecutionTier::Native);
    }

    #[test]
    fn test_add_custom_rule() {
        let mut engine = RulesEngine::new();
        engine.add_rule(RoutingRule {
            name: "custom_tool".to_string(),
            criteria: RoutingCriteria::Command {
                name: "custom".to_string(),
            },
            tier: ExecutionTier::Remote,
            priority: 1000,
        });

        let tier = engine.find_tier("custom", BinaryFormat::Unknown, false);
        assert_eq!(tier, ExecutionTier::Remote);
    }
}
