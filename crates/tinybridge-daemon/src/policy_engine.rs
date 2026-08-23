/// Policy engine for device access control
///
/// Implements hierarchical policy evaluation:
/// Platform > Project > VM > User
///
/// Each level can override the level above it.
use crate::device_manager::DeviceType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Policy access decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    /// Explicitly allow
    Allow,
    /// Explicitly deny
    Deny,
    /// Not specified (inherit from higher level)
    Inherit,
}

/// Policy rule for a device type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePolicy {
    /// Device type this policy applies to
    pub device_type: DeviceType,
    /// Allow/Deny/Inherit
    pub decision: AccessDecision,
    /// Optional reason (e.g., "security policy", "maintenance")
    pub reason: Option<String>,
    /// Optional whitelist of specific device IDs
    pub whitelist: Option<HashSet<String>>,
    /// Optional blacklist of specific device IDs
    pub blacklist: Option<HashSet<String>>,
}

impl DevicePolicy {
    pub fn new(device_type: DeviceType, decision: AccessDecision) -> Self {
        DevicePolicy {
            device_type,
            decision,
            reason: None,
            whitelist: None,
            blacklist: None,
        }
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn with_whitelist(mut self, devices: HashSet<String>) -> Self {
        self.whitelist = Some(devices);
        self
    }

    pub fn with_blacklist(mut self, devices: HashSet<String>) -> Self {
        self.blacklist = Some(devices);
        self
    }

    /// Check if a specific device ID matches this policy
    pub fn matches_device(&self, device_id: &str) -> bool {
        if let Some(whitelist) = &self.whitelist {
            return whitelist.contains(device_id);
        }
        if let Some(blacklist) = &self.blacklist {
            return !blacklist.contains(device_id);
        }
        true
    }
}

/// Policy level in the hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyLevel {
    /// Unique policy level ID
    pub id: String,
    /// Policies by device type
    pub policies: HashMap<String, DevicePolicy>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
}

impl PolicyLevel {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        PolicyLevel {
            id: Uuid::new_v4().to_string(),
            policies: HashMap::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Set policy for a device type
    pub fn set_policy(&mut self, device_type: DeviceType, policy: DevicePolicy) {
        self.policies.insert(device_type.to_string(), policy);
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    /// Get policy for a device type
    pub fn get_policy(&self, device_type: DeviceType) -> Option<&DevicePolicy> {
        self.policies.get(&device_type.to_string())
    }
}

impl Default for PolicyLevel {
    fn default() -> Self {
        Self::new()
    }
}

/// Request context for policy evaluation
#[derive(Debug, Clone)]
pub struct PolicyContext {
    /// Environment requesting access
    pub environment: String,
    /// Project name (if applicable)
    pub project: Option<String>,
    /// User ID (if applicable)
    pub user_id: Option<String>,
    /// Device ID being requested
    pub device_id: String,
    /// Device type
    pub device_type: DeviceType,
}

impl PolicyContext {
    pub fn new(environment: String, device_id: String, device_type: DeviceType) -> Self {
        PolicyContext {
            environment,
            project: None,
            user_id: None,
            device_id,
            device_type,
        }
    }

    pub fn with_project(mut self, project: String) -> Self {
        self.project = Some(project);
        self
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

/// Policy evaluation result with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Allow or Deny
    pub allowed: bool,
    /// Why the decision was made
    pub reason: String,
    /// Which policy level determined this
    pub policy_level: String,
    /// Audit trail
    pub decision_path: Vec<String>,
}

impl PolicyDecision {
    pub fn allow(reason: &str, policy_level: &str) -> Self {
        PolicyDecision {
            allowed: true,
            reason: reason.to_string(),
            policy_level: policy_level.to_string(),
            decision_path: vec![],
        }
    }

    pub fn deny(reason: &str, policy_level: &str) -> Self {
        PolicyDecision {
            allowed: false,
            reason: reason.to_string(),
            policy_level: policy_level.to_string(),
            decision_path: vec![],
        }
    }

    pub fn with_path(mut self, path: Vec<String>) -> Self {
        self.decision_path = path;
        self
    }
}

/// Hierarchical policy engine
pub struct PolicyEngine {
    /// Platform-level policies (default)
    platform_policy: PolicyLevel,
    /// Project-level policies (override platform)
    project_policies: HashMap<String, PolicyLevel>,
    /// Environment-level policies (override all)
    environment_policies: HashMap<String, PolicyLevel>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        PolicyEngine {
            platform_policy: PolicyLevel::new(),
            project_policies: HashMap::new(),
            environment_policies: HashMap::new(),
        }
    }

    /// Set platform-level policy (applies to all)
    pub fn set_platform_policy(&mut self, device_type: DeviceType, policy: DevicePolicy) {
        self.platform_policy.set_policy(device_type, policy);
    }

    /// Set project-level policy (overrides platform)
    pub fn set_project_policy(
        &mut self,
        project: String,
        device_type: DeviceType,
        policy: DevicePolicy,
    ) {
        self.project_policies
            .entry(project)
            .or_insert_with(PolicyLevel::new)
            .set_policy(device_type, policy);
    }

    /// Set environment-level policy (highest priority)
    pub fn set_environment_policy(
        &mut self,
        environment: String,
        device_type: DeviceType,
        policy: DevicePolicy,
    ) {
        self.environment_policies
            .entry(environment)
            .or_insert_with(PolicyLevel::new)
            .set_policy(device_type, policy);
    }

    /// Evaluate if access should be allowed
    pub fn evaluate(&self, context: &PolicyContext) -> PolicyDecision {
        let mut decision_path = vec![];

        // 1. Check environment-level policy (highest priority)
        if let Some(env_policy) = self.environment_policies.get(&context.environment) {
            decision_path.push(format!(
                "Checked environment policy: {}",
                &context.environment
            ));
            if let Some(device_policy) = env_policy.get_policy(context.device_type) {
                match device_policy.decision {
                    AccessDecision::Allow => {
                        if device_policy.matches_device(&context.device_id) {
                            return PolicyDecision::allow(
                                &device_policy.reason.clone().unwrap_or_default(),
                                "environment",
                            )
                            .with_path(decision_path);
                        } else {
                            decision_path.push(format!(
                                "Device {} not in whitelist/blacklist",
                                &context.device_id
                            ));
                        }
                    }
                    AccessDecision::Deny => {
                        return PolicyDecision::deny(
                            &device_policy.reason.clone().unwrap_or_default(),
                            "environment",
                        )
                        .with_path(decision_path);
                    }
                    AccessDecision::Inherit => {
                        decision_path.push("Environment policy: Inherit".to_string());
                    }
                }
            }
        }

        // 2. Check project-level policy
        if let Some(project) = &context.project {
            if let Some(proj_policy) = self.project_policies.get(project) {
                decision_path.push(format!("Checked project policy: {}", project));
                if let Some(device_policy) = proj_policy.get_policy(context.device_type) {
                    match device_policy.decision {
                        AccessDecision::Allow => {
                            if device_policy.matches_device(&context.device_id) {
                                return PolicyDecision::allow(
                                    &device_policy.reason.clone().unwrap_or_default(),
                                    "project",
                                )
                                .with_path(decision_path);
                            } else {
                                decision_path.push(format!(
                                    "Device {} not in whitelist/blacklist",
                                    &context.device_id
                                ));
                            }
                        }
                        AccessDecision::Deny => {
                            return PolicyDecision::deny(
                                &device_policy.reason.clone().unwrap_or_default(),
                                "project",
                            )
                            .with_path(decision_path);
                        }
                        AccessDecision::Inherit => {
                            decision_path.push("Project policy: Inherit".to_string());
                        }
                    }
                }
            }
        }

        // 3. Check platform-level policy (default/fallback)
        decision_path.push("Checked platform policy".to_string());
        if let Some(device_policy) = self.platform_policy.get_policy(context.device_type) {
            match device_policy.decision {
                AccessDecision::Allow => {
                    if device_policy.matches_device(&context.device_id) {
                        return PolicyDecision::allow(
                            &device_policy.reason.clone().unwrap_or_default(),
                            "platform",
                        )
                        .with_path(decision_path);
                    } else {
                        decision_path.push(format!(
                            "Device {} not in whitelist/blacklist",
                            &context.device_id
                        ));
                        return PolicyDecision::deny("Device not in whitelist", "platform")
                            .with_path(decision_path);
                    }
                }
                AccessDecision::Deny => {
                    return PolicyDecision::deny(
                        &device_policy.reason.clone().unwrap_or_default(),
                        "platform",
                    )
                    .with_path(decision_path);
                }
                AccessDecision::Inherit => {
                    decision_path.push("Platform policy: Inherit (default deny)".to_string());
                }
            }
        }

        // 4. Default: Deny if no policy matches
        PolicyDecision::deny("No matching policy (default deny)", "platform")
            .with_path(decision_path)
    }

    /// Get all policies for debugging/audit
    pub fn get_platform_policy(&self) -> &PolicyLevel {
        &self.platform_policy
    }

    pub fn get_project_policies(&self) -> &HashMap<String, PolicyLevel> {
        &self.project_policies
    }

    pub fn get_environment_policies(&self) -> &HashMap<String, PolicyLevel> {
        &self.environment_policies
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_policy_creation() {
        let policy = DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow)
            .with_reason("USB allowed for development".to_string());

        assert_eq!(policy.device_type, DeviceType::Usb);
        assert_eq!(policy.decision, AccessDecision::Allow);
        assert!(policy.reason.is_some());
    }

    #[test]
    fn test_policy_level_set_and_get() {
        let mut level = PolicyLevel::new();
        let policy = DevicePolicy::new(DeviceType::Serial, AccessDecision::Allow);

        level.set_policy(DeviceType::Serial, policy);
        assert!(level.get_policy(DeviceType::Serial).is_some());
    }

    #[test]
    fn test_policy_context_creation() {
        let ctx = PolicyContext::new(
            "ml-env".to_string(),
            "dev-123".to_string(),
            DeviceType::Camera,
        )
        .with_project("ai-project".to_string());

        assert_eq!(ctx.environment, "ml-env");
        assert_eq!(ctx.project, Some("ai-project".to_string()));
    }

    #[test]
    fn test_policy_engine_platform_allow() {
        let mut engine = PolicyEngine::new();

        // Platform allows USB
        engine.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow),
        );

        let ctx = PolicyContext::new("env1".to_string(), "usb-123".to_string(), DeviceType::Usb);
        let decision = engine.evaluate(&ctx);

        assert!(decision.allowed);
        assert_eq!(decision.policy_level, "platform");
    }

    #[test]
    fn test_policy_engine_platform_deny() {
        let mut engine = PolicyEngine::new();

        // Platform denies Camera
        engine.set_platform_policy(
            DeviceType::Camera,
            DevicePolicy::new(DeviceType::Camera, AccessDecision::Deny)
                .with_reason("Security policy".to_string()),
        );

        let ctx = PolicyContext::new(
            "env1".to_string(),
            "cam-456".to_string(),
            DeviceType::Camera,
        );
        let decision = engine.evaluate(&ctx);

        assert!(!decision.allowed);
        assert_eq!(decision.policy_level, "platform");
        assert!(decision.reason.contains("Security"));
    }

    #[test]
    fn test_project_overrides_platform() {
        let mut engine = PolicyEngine::new();

        // Platform denies USB
        engine.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Deny),
        );

        // Project allows USB
        engine.set_project_policy(
            "robotics".to_string(),
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow),
        );

        let ctx = PolicyContext::new(
            "robot-env".to_string(),
            "usb-123".to_string(),
            DeviceType::Usb,
        )
        .with_project("robotics".to_string());
        let decision = engine.evaluate(&ctx);

        assert!(decision.allowed);
        assert_eq!(decision.policy_level, "project");
    }

    #[test]
    fn test_environment_overrides_all() {
        let mut engine = PolicyEngine::new();

        // Platform allows
        engine.set_platform_policy(
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Allow),
        );

        // Project denies
        engine.set_project_policy(
            "project1".to_string(),
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Deny),
        );

        // Environment allows
        engine.set_environment_policy(
            "special-env".to_string(),
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Allow),
        );

        let ctx = PolicyContext::new(
            "special-env".to_string(),
            "serial-789".to_string(),
            DeviceType::Serial,
        )
        .with_project("project1".to_string());
        let decision = engine.evaluate(&ctx);

        assert!(decision.allowed);
        assert_eq!(decision.policy_level, "environment");
    }

    #[test]
    fn test_whitelist_enforcement() {
        let mut engine = PolicyEngine::new();

        let mut whitelist = HashSet::new();
        whitelist.insert("allowed-device-1".to_string());
        whitelist.insert("allowed-device-2".to_string());

        engine.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow).with_whitelist(whitelist),
        );

        // Whitelisted device should be allowed
        let ctx1 = PolicyContext::new(
            "env1".to_string(),
            "allowed-device-1".to_string(),
            DeviceType::Usb,
        );
        assert!(engine.evaluate(&ctx1).allowed);

        // Non-whitelisted device should be denied
        let ctx2 = PolicyContext::new(
            "env1".to_string(),
            "forbidden-device".to_string(),
            DeviceType::Usb,
        );
        assert!(!engine.evaluate(&ctx2).allowed);
    }

    #[test]
    fn test_blacklist_enforcement() {
        let mut engine = PolicyEngine::new();

        let mut blacklist = HashSet::new();
        blacklist.insert("forbidden-device-1".to_string());

        engine.set_platform_policy(
            DeviceType::Serial,
            DevicePolicy::new(DeviceType::Serial, AccessDecision::Allow).with_blacklist(blacklist),
        );

        // Blacklisted device should be denied
        let ctx1 = PolicyContext::new(
            "env1".to_string(),
            "forbidden-device-1".to_string(),
            DeviceType::Serial,
        );
        assert!(!engine.evaluate(&ctx1).allowed);

        // Non-blacklisted device should be allowed
        let ctx2 = PolicyContext::new(
            "env1".to_string(),
            "allowed-device".to_string(),
            DeviceType::Serial,
        );
        assert!(engine.evaluate(&ctx2).allowed);
    }

    #[test]
    fn test_default_deny() {
        let engine = PolicyEngine::new();

        let ctx = PolicyContext::new("env1".to_string(), "dev-123".to_string(), DeviceType::Usb);
        let decision = engine.evaluate(&ctx);

        assert!(!decision.allowed);
    }

    #[test]
    fn test_decision_path_audit_trail() {
        let mut engine = PolicyEngine::new();

        engine.set_platform_policy(
            DeviceType::Audio,
            DevicePolicy::new(DeviceType::Audio, AccessDecision::Allow),
        );

        let ctx = PolicyContext::new(
            "audio-env".to_string(),
            "audio-123".to_string(),
            DeviceType::Audio,
        );
        let decision = engine.evaluate(&ctx);

        assert!(!decision.decision_path.is_empty());
        assert!(decision
            .decision_path
            .iter()
            .any(|p| p.contains("platform")));
    }

    #[test]
    fn test_inherit_mechanism() {
        let mut engine = PolicyEngine::new();

        // Platform allows USB
        engine.set_platform_policy(
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Allow),
        );

        // Project says Inherit
        engine.set_project_policy(
            "proj1".to_string(),
            DeviceType::Usb,
            DevicePolicy::new(DeviceType::Usb, AccessDecision::Inherit),
        );

        let ctx = PolicyContext::new("env1".to_string(), "usb-123".to_string(), DeviceType::Usb)
            .with_project("proj1".to_string());
        let decision = engine.evaluate(&ctx);

        assert!(decision.allowed);
        assert_eq!(decision.policy_level, "platform");
    }
}
