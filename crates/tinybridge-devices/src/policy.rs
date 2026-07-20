use crate::device::DeviceType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Policy enforcement level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum PolicyLevel {
    /// Global platform policy (highest priority)
    Platform,
    /// Project-level policy
    Project,
    /// VM-level policy
    Vm,
    /// User-level policy (lowest priority)
    User,
}

impl std::fmt::Display for PolicyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyLevel::Platform => write!(f, "Platform"),
            PolicyLevel::Project => write!(f, "Project"),
            PolicyLevel::Vm => write!(f, "VM"),
            PolicyLevel::User => write!(f, "User"),
        }
    }
}

/// Device access decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    /// Device passthrough is allowed
    Allow,
    /// Device passthrough is blocked
    Block,
    /// Default behavior (allowed, but can be overridden)
    Default,
}

/// Reason for blocking device access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockReason {
    /// Security policy blocks this device class
    SecurityPolicy,
    /// Compliance requirement blocks this device
    Compliance,
    /// Data Loss Prevention policy blocks this device
    Dlp,
    /// Resource reserved for host/critical workloads
    ResourceGovernance,
    /// Device type not supported in this configuration
    UnsupportedDevice,
    /// Operational stability concern
    OperationalStability,
    /// Multi-tenant isolation policy
    MultiTenantIsolation,
    /// Custom reason
    Custom(String),
}

impl std::fmt::Display for BlockReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockReason::SecurityPolicy => write!(f, "Security policy blocks this device"),
            BlockReason::Compliance => write!(f, "Compliance requirement restricts access"),
            BlockReason::Dlp => write!(f, "Data Loss Prevention policy blocks this device"),
            BlockReason::ResourceGovernance => write!(f, "Resource reserved for critical use"),
            BlockReason::UnsupportedDevice => write!(f, "Device type not supported"),
            BlockReason::OperationalStability => write!(f, "Device access may impact stability"),
            BlockReason::MultiTenantIsolation => write!(f, "Multi-tenant isolation policy"),
            BlockReason::Custom(reason) => write!(f, "{}", reason),
        }
    }
}

/// Device passthrough policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule ID
    pub id: Uuid,

    /// Device type this rule applies to
    pub device_type: DeviceType,

    /// Access decision
    pub decision: AccessDecision,

    /// Reason for blocking (if blocked)
    pub block_reason: Option<BlockReason>,

    /// Policy level this rule applies to
    pub level: PolicyLevel,

    /// Whether admin approval is required to override this rule
    pub requires_admin_approval: bool,

    /// Additional context (compliance reference, ticket, etc.)
    pub compliance_reference: Option<String>,

    /// When this rule was created
    pub created_at: DateTime<Utc>,

    /// Admin who created this rule
    pub created_by: Option<String>,

    /// Description of this policy rule
    pub description: Option<String>,
}

impl PolicyRule {
    /// Create a new allow policy rule
    pub fn allow(device_type: DeviceType, level: PolicyLevel) -> Self {
        Self {
            id: Uuid::new_v4(),
            device_type,
            decision: AccessDecision::Allow,
            block_reason: None,
            level,
            requires_admin_approval: false,
            compliance_reference: None,
            created_at: Utc::now(),
            created_by: None,
            description: None,
        }
    }

    /// Create a new block policy rule
    pub fn block(device_type: DeviceType, level: PolicyLevel, reason: BlockReason) -> Self {
        Self {
            id: Uuid::new_v4(),
            device_type,
            decision: AccessDecision::Block,
            block_reason: Some(reason),
            level,
            requires_admin_approval: false,
            compliance_reference: None,
            created_at: Utc::now(),
            created_by: None,
            description: None,
        }
    }

    /// Add compliance reference
    pub fn with_compliance_reference(mut self, reference: String) -> Self {
        self.compliance_reference = Some(reference);
        self
    }

    /// Require admin approval to override
    pub fn requires_approval(mut self) -> Self {
        self.requires_admin_approval = true;
        self
    }

    /// Add description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add creator info
    pub fn created_by(mut self, admin: String) -> Self {
        self.created_by = Some(admin);
        self
    }
}

/// Policy audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAuditEvent {
    /// Event ID
    pub id: Uuid,

    /// Event type
    pub event_type: PolicyEventType,

    /// Device involved (if applicable)
    pub device_id: Option<Uuid>,

    /// Device type
    pub device_type: Option<DeviceType>,

    /// Policy rule applied
    pub policy_rule_id: Option<Uuid>,

    /// User attempting access
    pub user_id: Option<String>,

    /// Environment ID
    pub env_id: Option<Uuid>,

    /// Decision made
    pub decision: AccessDecision,

    /// Reason if blocked
    pub block_reason: Option<BlockReason>,

    /// Policy level that enforced the decision
    pub policy_level: Option<PolicyLevel>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Additional context
    pub context: Option<String>,
}

/// Type of policy event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEventType {
    /// Device attachment attempted
    AttachmentAttempted,
    /// Device attachment allowed
    AttachmentAllowed,
    /// Device attachment denied
    AttachmentDenied,
    /// Device detached
    Detached,
    /// Policy rule created
    PolicyCreated,
    /// Policy rule deleted
    PolicyDeleted,
    /// Override granted by admin
    OverrideGranted,
    /// Override denied
    OverrideDenied,
}

/// Device passthrough policy engine
pub struct PolicyEngine {
    /// All active policy rules
    rules: HashMap<Uuid, PolicyRule>,

    /// Rules indexed by device type for fast lookup
    device_type_rules: HashMap<DeviceType, Vec<Uuid>>,

    /// Audit log of all policy decisions
    audit_log: Vec<PolicyAuditEvent>,

    /// Admin overrides (temporary allow exceptions)
    overrides: HashMap<(Option<Uuid>, DeviceType), DateTime<Utc>>, // (env_id, device_type) -> expiry
}

impl PolicyEngine {
    /// Create a new policy engine with default allow-all rules
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            device_type_rules: HashMap::new(),
            audit_log: Vec::new(),
            overrides: HashMap::new(),
        }
    }

    /// Add a policy rule
    pub fn add_rule(&mut self, rule: PolicyRule) -> Uuid {
        let device_type = rule.device_type;
        let rule_id = rule.id;

        self.device_type_rules
            .entry(device_type)
            .or_default()
            .push(rule_id);

        self.rules.insert(rule_id, rule);
        rule_id
    }

    /// Remove a policy rule
    pub fn remove_rule(&mut self, rule_id: Uuid) -> Option<PolicyRule> {
        if let Some(rule) = self.rules.remove(&rule_id) {
            if let Some(rules) = self.device_type_rules.get_mut(&rule.device_type) {
                rules.retain(|id| id != &rule_id);
            }
            return Some(rule);
        }
        None
    }

    /// Check if device access is allowed
    pub fn check_access(
        &mut self,
        device_type: DeviceType,
        env_id: Option<Uuid>,
        user_id: Option<&str>,
    ) -> AccessResult {
        // Check for admin override
        if let Some(expiry) = self.overrides.get(&(env_id, device_type)) {
            if *expiry > Utc::now() {
                let event = PolicyAuditEvent {
                    id: Uuid::new_v4(),
                    event_type: PolicyEventType::AttachmentAllowed,
                    device_id: None,
                    device_type: Some(device_type),
                    policy_rule_id: None,
                    user_id: user_id.map(|u| u.to_string()),
                    env_id,
                    decision: AccessDecision::Allow,
                    block_reason: None,
                    policy_level: None,
                    timestamp: Utc::now(),
                    context: Some("Admin override in effect".to_string()),
                };
                self.audit_log.push(event);

                return AccessResult {
                    allowed: true,
                    reason: None,
                    policy_level: None,
                    requires_approval: false,
                    compliance_reference: None,
                };
            }
        }

        // Find applicable rules (highest priority first)
        let mut applicable_rules: Vec<_> = self
            .device_type_rules
            .get(&device_type)
            .map(|rule_ids| {
                rule_ids
                    .iter()
                    .filter_map(|id| self.rules.get(id))
                    .collect()
            })
            .unwrap_or_default();

        applicable_rules.sort_by_key(|rule| std::cmp::Reverse(rule.level));

        // Apply first matching rule
        if let Some(rule) = applicable_rules.first() {
            let decision = rule.decision;
            let allowed = decision == AccessDecision::Allow || decision == AccessDecision::Default;

            let event = PolicyAuditEvent {
                id: Uuid::new_v4(),
                event_type: if allowed {
                    PolicyEventType::AttachmentAllowed
                } else {
                    PolicyEventType::AttachmentDenied
                },
                device_id: None,
                device_type: Some(device_type),
                policy_rule_id: Some(rule.id),
                user_id: user_id.map(|u| u.to_string()),
                env_id,
                decision,
                block_reason: rule.block_reason.clone(),
                policy_level: Some(rule.level),
                timestamp: Utc::now(),
                context: rule.description.clone(),
            };
            self.audit_log.push(event);

            return AccessResult {
                allowed,
                reason: rule.block_reason.clone(),
                policy_level: Some(rule.level),
                requires_approval: rule.requires_admin_approval && !allowed,
                compliance_reference: rule.compliance_reference.clone(),
            };
        }

        // Default: allow (no policy restriction)
        let event = PolicyAuditEvent {
            id: Uuid::new_v4(),
            event_type: PolicyEventType::AttachmentAllowed,
            device_id: None,
            device_type: Some(device_type),
            policy_rule_id: None,
            user_id: user_id.map(|u| u.to_string()),
            env_id,
            decision: AccessDecision::Default,
            block_reason: None,
            policy_level: None,
            timestamp: Utc::now(),
            context: Some("No policy restriction".to_string()),
        };
        self.audit_log.push(event);

        AccessResult {
            allowed: true,
            reason: None,
            policy_level: None,
            requires_approval: false,
            compliance_reference: None,
        }
    }

    /// Grant temporary admin override
    pub fn grant_override(
        &mut self,
        env_id: Option<Uuid>,
        device_type: DeviceType,
        duration_secs: u64,
    ) {
        let expiry = Utc::now() + chrono::Duration::seconds(duration_secs as i64);
        self.overrides.insert((env_id, device_type), expiry);

        let event = PolicyAuditEvent {
            id: Uuid::new_v4(),
            event_type: PolicyEventType::OverrideGranted,
            device_id: None,
            device_type: Some(device_type),
            policy_rule_id: None,
            user_id: None,
            env_id,
            decision: AccessDecision::Allow,
            block_reason: None,
            policy_level: None,
            timestamp: Utc::now(),
            context: Some(format!("Override expires in {} seconds", duration_secs)),
        };
        self.audit_log.push(event);
    }

    /// Get audit log
    pub fn get_audit_log(&self) -> Vec<PolicyAuditEvent> {
        self.audit_log.clone()
    }

    /// Get audit log for specific device type
    pub fn get_audit_log_for_device(&self, device_type: DeviceType) -> Vec<PolicyAuditEvent> {
        self.audit_log
            .iter()
            .filter(|e| e.device_type == Some(device_type))
            .cloned()
            .collect()
    }

    /// Get audit log for specific environment
    pub fn get_audit_log_for_env(&self, env_id: Uuid) -> Vec<PolicyAuditEvent> {
        self.audit_log
            .iter()
            .filter(|e| e.env_id == Some(env_id))
            .cloned()
            .collect()
    }

    /// Export audit log for compliance reporting
    pub fn export_audit_log(&self) -> String {
        serde_json::to_string_pretty(&self.audit_log).unwrap_or_default()
    }

    /// Get all policy rules
    pub fn list_rules(&self) -> Vec<PolicyRule> {
        self.rules.values().cloned().collect()
    }

    /// Get rules for a specific device type
    pub fn get_rules_for_device(&self, device_type: DeviceType) -> Vec<PolicyRule> {
        self.device_type_rules
            .get(&device_type)
            .map(|rule_ids| {
                rule_ids
                    .iter()
                    .filter_map(|id| self.rules.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of access check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessResult {
    /// Whether access is allowed
    pub allowed: bool,

    /// Reason for blocking (if blocked)
    pub reason: Option<BlockReason>,

    /// Which policy level enforced this decision
    pub policy_level: Option<PolicyLevel>,

    /// Whether admin approval is required to override
    pub requires_approval: bool,

    /// Compliance reference (if applicable)
    pub compliance_reference: Option<String>,
}

impl AccessResult {
    /// Get user-facing message
    pub fn user_message(&self) -> String {
        if self.allowed {
            "Device passthrough is allowed.".to_string()
        } else {
            let mut msg = format!(
                "Device passthrough is blocked: {}",
                self.reason
                    .as_ref()
                    .map(|r| r.to_string())
                    .unwrap_or_default()
            );

            if let Some(level) = self.policy_level {
                msg.push_str(&format!(" (enforced by {} policy)", level));
            }

            if let Some(reference) = &self.compliance_reference {
                msg.push_str(&format!(" [{}]", reference));
            }

            if self.requires_approval {
                msg.push_str(" Contact your administrator to request an override.");
            }

            msg
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_allow_rule() {
        let rule = PolicyRule::allow(DeviceType::Serial, PolicyLevel::Platform);
        assert_eq!(rule.decision, AccessDecision::Allow);
    }

    #[test]
    fn test_policy_block_rule() {
        let rule = PolicyRule::block(DeviceType::Usb, PolicyLevel::Platform, BlockReason::Dlp);
        assert_eq!(rule.decision, AccessDecision::Block);
        assert!(rule.block_reason.is_some());
    }

    #[test]
    fn test_policy_engine_allow() {
        let mut engine = PolicyEngine::new();
        let rule = PolicyRule::allow(DeviceType::Serial, PolicyLevel::Platform);
        engine.add_rule(rule);

        let result = engine.check_access(DeviceType::Serial, None, None);
        assert!(result.allowed);
    }

    #[test]
    fn test_policy_engine_block() {
        let mut engine = PolicyEngine::new();
        let rule = PolicyRule::block(DeviceType::Usb, PolicyLevel::Platform, BlockReason::Dlp);
        engine.add_rule(rule);

        let result = engine.check_access(DeviceType::Usb, None, None);
        assert!(!result.allowed);
        assert!(result.reason.is_some());
    }

    #[test]
    fn test_admin_override() {
        let mut engine = PolicyEngine::new();
        let rule = PolicyRule::block(
            DeviceType::Camera,
            PolicyLevel::Platform,
            BlockReason::SecurityPolicy,
        );
        engine.add_rule(rule);

        // Initially blocked
        let result = engine.check_access(DeviceType::Camera, None, None);
        assert!(!result.allowed);

        // Grant override
        engine.grant_override(None, DeviceType::Camera, 3600);

        // Now allowed
        let result = engine.check_access(DeviceType::Camera, None, None);
        assert!(result.allowed);
    }

    #[test]
    fn test_audit_logging() {
        let mut engine = PolicyEngine::new();
        engine.check_access(DeviceType::Serial, None, Some("user1"));

        let events = engine.get_audit_log();
        assert!(!events.is_empty());
        assert_eq!(events[0].user_id, Some("user1".to_string()));
    }

    #[test]
    fn test_policy_hierarchy() {
        let mut engine = PolicyEngine::new();

        // Platform policy: block all USB
        let platform_rule =
            PolicyRule::block(DeviceType::Usb, PolicyLevel::Platform, BlockReason::Dlp);
        engine.add_rule(platform_rule);

        // VM-level override: allow this specific VM
        let vm_rule = PolicyRule::allow(DeviceType::Usb, PolicyLevel::Vm);
        engine.add_rule(vm_rule);

        let env_id = Some(Uuid::new_v4());
        let result = engine.check_access(DeviceType::Usb, env_id, None);

        // VM-level should win (higher priority than platform)
        assert!(result.allowed);
    }
}
