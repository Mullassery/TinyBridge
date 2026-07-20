use crate::detector::BinaryDetector;
use crate::error::Result;
use crate::rules::RulesEngine;
use crate::{BinaryFormat, ExecutionTier};
use std::path::Path;

/// Routing decision with metadata
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// Target execution tier
    pub tier: ExecutionTier,

    /// Detected binary format
    pub format: BinaryFormat,

    /// Detected architecture
    pub architecture: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Reasoning for the decision
    pub reason: String,
}

/// Main router for workload execution decisions
pub struct Router {
    rules_engine: RulesEngine,
}

impl Router {
    /// Create a new router with default rules
    pub fn new() -> Self {
        Self {
            rules_engine: RulesEngine::with_defaults(),
        }
    }

    /// Create a router with custom rules
    pub fn with_rules(rules_engine: RulesEngine) -> Self {
        Self { rules_engine }
    }

    /// Determine where to execute a workload
    pub fn route(&self, path: impl AsRef<Path>) -> Result<RoutingDecision> {
        let path = path.as_ref();

        // Detect binary format
        let format = BinaryDetector::detect_format(path)?;

        // Detect architecture
        let architecture = BinaryDetector::detect_architecture(path)?;

        // Get command name from path
        let command = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Determine tier
        let tier = self.rules_engine.find_tier(command, format, false);

        // Build decision with reasoning
        let reason = match format {
            BinaryFormat::MachO => format!("Native macOS binary ({})", architecture),
            BinaryFormat::Elf => format!("Linux binary ({})", architecture),
            BinaryFormat::Script => "Shell script".to_string(),
            BinaryFormat::Unknown => "Unknown format, defaulting to Linux".to_string(),
        };

        Ok(RoutingDecision {
            tier,
            format,
            architecture,
            confidence: 0.95,
            reason,
        })
    }

    /// Route a command by name (without file detection)
    pub fn route_command(&self, command: &str) -> RoutingDecision {
        let tier = self
            .rules_engine
            .find_tier(command, BinaryFormat::Unknown, false);

        RoutingDecision {
            tier,
            format: BinaryFormat::Unknown,
            architecture: "unknown".to_string(),
            confidence: 0.7,
            reason: format!("Routed by command name: {}", command),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_route_script() {
        use std::io::Write;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"#!/bin/bash\necho hello").unwrap();
        file.flush().unwrap();

        let router = Router::new();
        let decision = router.route(file.path()).unwrap();

        assert_eq!(decision.format, BinaryFormat::Script);
        assert_eq!(decision.tier, ExecutionTier::Native);
    }

    #[test]
    fn test_route_command() {
        let router = Router::new();
        let decision = router.route_command("python");

        assert_eq!(decision.tier, ExecutionTier::Linux);
    }

    #[test]
    fn test_route_swift_command() {
        let router = Router::new();
        let decision = router.route_command("swift");

        assert_eq!(decision.tier, ExecutionTier::Native);
    }
}
