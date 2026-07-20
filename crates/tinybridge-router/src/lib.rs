pub mod detector;
pub mod error;
pub mod rules;
pub mod profiler;
pub mod router;

pub use detector::BinaryFormat;
pub use error::RouterError;
pub use router::{Router, RoutingDecision};
pub use rules::{RoutingRule, RulesEngine};

use serde::{Deserialize, Serialize};

/// Execution tier for workload routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionTier {
    /// Native macOS execution (fast, no VM overhead)
    Native,
    /// Linux substrate execution (full Linux compatibility)
    Linux,
    /// Remote GPU/cluster execution (heavy compute)
    Remote,
}

impl std::fmt::Display for ExecutionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionTier::Native => write!(f, "native"),
            ExecutionTier::Linux => write!(f, "linux"),
            ExecutionTier::Remote => write!(f, "remote"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_tier_display() {
        assert_eq!(ExecutionTier::Native.to_string(), "native");
        assert_eq!(ExecutionTier::Linux.to_string(), "linux");
        assert_eq!(ExecutionTier::Remote.to_string(), "remote");
    }
}
