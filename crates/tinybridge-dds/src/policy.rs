use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// DDS policy enforcement levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DdsPolicyLevel {
    /// Platform-wide policy (highest priority)
    Platform = 3,
    /// Project-level policy
    Project = 2,
    /// Environment-level policy
    Environment = 1,
    /// User-level policy (lowest priority, ignored if higher level overrides)
    User = 0,
}

/// Reasons why DDS might be blocked
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DdsBlockReason {
    /// Security policy blocks DDS
    SecurityPolicy,
    /// Compliance requirement blocks DDS
    Compliance,
    /// Network isolation prevents DDS
    NetworkIsolation,
    /// Resource constraints prevent DDS
    ResourceConstraints,
    /// DDS not supported on this substrate
    UnsupportedSubstrate,
    /// Administrator requires approval
    RequiresApproval,
    /// Custom block reason
    Custom(String),
}

/// Access decision
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DdsAccessDecision {
    /// Access allowed
    Allow,
    /// Access blocked
    Block,
    /// Use default behavior (no explicit policy)
    Default,
}

/// Policy rule for DDS access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsPolicy {
    /// Unique ID
    pub id: Uuid,
    /// Environment this policy applies to
    pub env_id: Option<Uuid>,
    /// Feature this policy applies to (empty = all features)
    pub feature: String,
    /// Allow or block
    pub decision: DdsAccessDecision,
    /// Reason if blocked
    pub block_reason: Option<DdsBlockReason>,
    /// Policy level
    pub level: DdsPolicyLevel,
    /// Requires admin approval to override
    pub requires_admin_approval: bool,
    /// Compliance reference
    pub compliance_reference: Option<String>,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Description
    pub description: Option<String>,
}

impl DdsPolicy {
    /// Create a block policy
    pub fn block(level: DdsPolicyLevel, block_reason: DdsBlockReason) -> Self {
        Self {
            id: Uuid::new_v4(),
            env_id: None,
            feature: String::new(),
            decision: DdsAccessDecision::Block,
            block_reason: Some(block_reason),
            level,
            requires_admin_approval: false,
            compliance_reference: None,
            created_at: Utc::now(),
            description: None,
        }
    }

    /// Create an allow policy
    pub fn allow(level: DdsPolicyLevel) -> Self {
        Self {
            id: Uuid::new_v4(),
            env_id: None,
            feature: String::new(),
            decision: DdsAccessDecision::Allow,
            block_reason: None,
            level,
            requires_admin_approval: false,
            compliance_reference: None,
            created_at: Utc::now(),
            description: None,
        }
    }

    /// Set environment ID
    pub fn with_env_id(mut self, env_id: Uuid) -> Self {
        self.env_id = Some(env_id);
        self
    }

    /// Set feature
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.feature = feature.into();
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set compliance reference
    pub fn with_compliance_reference(mut self, reference: impl Into<String>) -> Self {
        self.compliance_reference = Some(reference.into());
        self
    }

    /// Mark as requiring admin approval
    pub fn requires_approval(mut self) -> Self {
        self.requires_admin_approval = true;
        self
    }
}

/// Result of access check
#[derive(Debug, Clone)]
pub struct DdsAccessResult {
    pub allowed: bool,
    pub policy_level: Option<DdsPolicyLevel>,
    pub block_reason: Option<DdsBlockReason>,
    pub requires_approval: bool,
    pub compliance_reference: Option<String>,
}

impl DdsAccessResult {
    /// Get user-friendly message
    pub fn user_message(&self) -> String {
        if self.allowed {
            "DDS feature is allowed".to_string()
        } else {
            let mut msg = "DDS feature is blocked".to_string();
            if let Some(reason) = &self.block_reason {
                msg.push_str(&format!(": {:?}", reason));
            }
            if let Some(comp) = &self.compliance_reference {
                msg.push_str(&format!(" ({})", comp));
            }
            msg
        }
    }
}

/// Admin override for temporary access
#[derive(Debug, Clone)]
pub struct DdsOverride {
    pub env_id: Uuid,
    pub feature: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub granted_by: Option<String>,
    pub reason: Option<String>,
}

/// DDS Policy Engine
pub struct DdsPolicyEngine {
    policies: Vec<DdsPolicy>,
    overrides: HashMap<(Uuid, String), DdsOverride>,
}

impl DdsPolicyEngine {
    /// Create new policy engine
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            overrides: HashMap::new(),
        }
    }

    /// Add a policy rule
    pub fn add_policy(&mut self, policy: DdsPolicy) {
        self.policies.push(policy);
        // Sort by level (higher level first)
        self.policies.sort_by(|a, b| b.level.cmp(&a.level));
    }

    /// Remove policy by ID
    pub fn remove_policy(&mut self, policy_id: Uuid) {
        self.policies.retain(|p| p.id != policy_id);
    }

    /// List all policies
    pub fn list_policies(&self) -> &[DdsPolicy] {
        &self.policies
    }

    /// Check access to a DDS feature
    pub fn check_access(&self, env_id: Uuid, feature: &str) -> DdsAccessResult {
        // Check for active override first
        if let Some(override_) = self.overrides.get(&(env_id, feature.to_string())) {
            if override_.expires_at > Utc::now() {
                return DdsAccessResult {
                    allowed: true,
                    policy_level: None,
                    block_reason: None,
                    requires_approval: false,
                    compliance_reference: None,
                };
            }
        }

        // Check policies in order (platform > project > environment > user)
        for policy in &self.policies {
            if policy.feature.is_empty() || policy.feature == feature {
                match policy.decision {
                    DdsAccessDecision::Allow => {
                        return DdsAccessResult {
                            allowed: true,
                            policy_level: Some(policy.level),
                            block_reason: None,
                            requires_approval: policy.requires_admin_approval,
                            compliance_reference: policy.compliance_reference.clone(),
                        }
                    }
                    DdsAccessDecision::Block => {
                        return DdsAccessResult {
                            allowed: false,
                            policy_level: Some(policy.level),
                            block_reason: policy.block_reason.clone(),
                            requires_approval: policy.requires_admin_approval,
                            compliance_reference: policy.compliance_reference.clone(),
                        }
                    }
                    DdsAccessDecision::Default => continue,
                }
            }
        }

        // Default: allow (DDS is opt-in, not opt-out)
        DdsAccessResult {
            allowed: true,
            policy_level: None,
            block_reason: None,
            requires_approval: false,
            compliance_reference: None,
        }
    }

    /// Grant temporary override
    pub fn grant_override(
        &mut self,
        env_id: Uuid,
        feature: impl Into<String>,
        duration_secs: u64,
        granted_by: Option<String>,
        reason: Option<String>,
    ) {
        let feature = feature.into();
        let now = Utc::now();
        self.overrides.insert(
            (env_id, feature.clone()),
            DdsOverride {
                env_id,
                feature,
                granted_at: now,
                expires_at: now + Duration::seconds(duration_secs as i64),
                granted_by,
                reason,
            },
        );
    }

    /// Revoke override
    pub fn revoke_override(&mut self, env_id: Uuid, feature: &str) {
        self.overrides.remove(&(env_id, feature.to_string()));
    }

    /// List active overrides
    pub fn list_active_overrides(&self) -> Vec<&DdsOverride> {
        let now = Utc::now();
        self.overrides
            .values()
            .filter(|o| o.expires_at > now)
            .collect()
    }
}

impl Default for DdsPolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_block() {
        let policy = DdsPolicy::block(DdsPolicyLevel::Platform, DdsBlockReason::SecurityPolicy);
        assert_eq!(policy.decision, DdsAccessDecision::Block);
    }

    #[test]
    fn test_policy_allow() {
        let policy = DdsPolicy::allow(DdsPolicyLevel::Environment);
        assert_eq!(policy.decision, DdsAccessDecision::Allow);
    }

    #[test]
    fn test_add_policy() {
        let mut engine = DdsPolicyEngine::new();
        let policy = DdsPolicy::allow(DdsPolicyLevel::Platform);
        engine.add_policy(policy);
        assert_eq!(engine.list_policies().len(), 1);
    }

    #[test]
    fn test_check_access_default_allow() {
        let engine = DdsPolicyEngine::new();
        let env_id = Uuid::new_v4();
        let result = engine.check_access(env_id, "discovery");
        assert!(result.allowed);
    }

    #[test]
    fn test_check_access_block() {
        let mut engine = DdsPolicyEngine::new();
        let policy = DdsPolicy::block(DdsPolicyLevel::Platform, DdsBlockReason::SecurityPolicy);
        engine.add_policy(policy);

        let env_id = Uuid::new_v4();
        let result = engine.check_access(env_id, "discovery");
        assert!(!result.allowed);
    }

    #[test]
    fn test_override_allows_access() {
        let mut engine = DdsPolicyEngine::new();
        let policy = DdsPolicy::block(DdsPolicyLevel::Platform, DdsBlockReason::SecurityPolicy);
        engine.add_policy(policy);

        let env_id = Uuid::new_v4();
        engine.grant_override(env_id, "discovery", 3600, None, None);

        let result = engine.check_access(env_id, "discovery");
        assert!(result.allowed);
    }

    #[test]
    fn test_override_expires() {
        let mut engine = DdsPolicyEngine::new();
        let policy = DdsPolicy::block(DdsPolicyLevel::Platform, DdsBlockReason::SecurityPolicy);
        engine.add_policy(policy);

        let env_id = Uuid::new_v4();
        // Grant override for 0 seconds (immediately expired)
        engine.grant_override(env_id, "discovery", 0, None, None);

        let result = engine.check_access(env_id, "discovery");
        assert!(!result.allowed);
    }

    #[test]
    fn test_user_message() {
        let result = DdsAccessResult {
            allowed: false,
            policy_level: Some(DdsPolicyLevel::Platform),
            block_reason: Some(DdsBlockReason::SecurityPolicy),
            requires_approval: false,
            compliance_reference: None,
        };
        let msg = result.user_message();
        assert!(msg.contains("blocked"));
    }
}
