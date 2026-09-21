# Hardware Device Passthrough: Policy & Governance

**Status:** Implemented in Phase 3  
**Security Level:** Enterprise-Grade  
**Compliance Ready:** Yes (audit trails, reporting, hierarchical policies)

---

## Overview

TinyBridge implements a comprehensive hardware device passthrough governance system that provides developer convenience with enterprise-grade security and compliance requirements.

**Design Principles:**
1. ✅ **Default Allow (Developer-Friendly):** Device passthrough enabled by default for best out-of-the-box experience
2. ✅ **Hierarchical Control:** Policy hierarchy (Platform > Project > VM > User) allows flexible restrictions
3. ✅ **Audit Everything:** All device access decisions logged for compliance and forensics
4. ✅ **Enterprise Security:** Support for security policies, compliance requirements, DLP, multi-tenant isolation
5. ✅ **Clear Communication:** Users see exactly why access is restricted and how to request override

---

## Architecture

### Policy Hierarchy

Policies are evaluated from highest to lowest priority:

```
Platform Level (highest)
    ↓ [can override]
Project Level
    ↓ [can override]
VM Level
    ↓ [can override]
User Level (lowest)
    ↓ [default: allow if no policies]
System Default (allow)
```

**Example:**
```
Platform: Block all USB (DLP policy)
Project:  Allow specific USB (exception)
VM:       Block Camera (device conflict)
User:     (no policy)
Result:   USB allowed, Camera blocked
```

### Policy Rules

```rust
pub struct PolicyRule {
    pub id: Uuid,
    pub device_type: DeviceType,      // USB, Serial, Camera, Audio
    pub decision: AccessDecision,      // Allow, Block, Default
    pub block_reason: Option<BlockReason>,
    pub level: PolicyLevel,            // Platform/Project/VM/User
    pub requires_admin_approval: bool,
    pub compliance_reference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub description: Option<String>,
}
```

### Block Reasons

Admins can specify WHY a device is blocked:

```rust
pub enum BlockReason {
    SecurityPolicy,         // Security team requires blocking
    Compliance,            // Regulatory requirement (HIPAA, PCI-DSS, etc.)
    Dlp,                   // Data Loss Prevention blocks USB storage
    ResourceGovernance,    // GPU reserved for high-priority workloads
    UnsupportedDevice,     // Device not compatible with this VM
    OperationalStability,  // Device access causes crashes
    MultiTenantIsolation,  // Tenant A can't access Tenant B's hardware
    Custom(String),        // Custom reason
}
```

---

## Use Cases

### 1. Security Policy: Block USB Entirely

**Scenario:** Security team wants to prevent all USB device access (data exfiltration risk)

**Configuration:**
```rust
let rule = PolicyRule::block(
    DeviceType::Usb,
    PolicyLevel::Platform,
    BlockReason::Dlp,
)
.with_description("USB devices blocked to prevent data exfiltration")
.with_compliance_reference("SEC-2024-001");

policy_engine.add_rule(rule);
```

**User Experience:**
```bash
$ tinybridge devices attach myvm /dev/bus/usb/001/010
❌ Error: Device passthrough is blocked: Data Loss Prevention policy 
          blocks this device (enforced by Platform policy) [SEC-2024-001]
   
   Contact your administrator to request an override.
```

---

### 2. Compliance Requirement: Restrict Serial Ports

**Scenario:** Healthcare (HIPAA) requires logging all serial device access

**Configuration:**
```rust
let rule = PolicyRule::block(
    DeviceType::Serial,
    PolicyLevel::Platform,
    BlockReason::Compliance,
)
.with_compliance_reference("HIPAA-PHI-001")
.with_description("Serial devices blocked per HIPAA requirements")
.requires_approval();  // Need admin override

policy_engine.add_rule(rule);
```

**Audit Trail:**
```json
{
  "id": "uuid",
  "event_type": "AttachmentAttempted",
  "device_type": "Serial",
  "user_id": "alice@company.com",
  "env_id": "medical-app-vm",
  "decision": "Block",
  "block_reason": "Compliance",
  "policy_level": "Platform",
  "timestamp": "2026-07-20T15:30:42Z",
  "context": "Serial devices blocked per HIPAA requirements",
  "compliance_reference": "HIPAA-PHI-001"
}
```

---

### 3. Project Exception: Allow Specific Device

**Scenario:** Platform blocks USB, but a specific project needs USB scanner

**Configuration:**
```rust
// Platform: Block all USB (security policy)
let platform_rule = PolicyRule::block(
    DeviceType::Usb,
    PolicyLevel::Platform,
    BlockReason::SecurityPolicy,
);
policy_engine.add_rule(platform_rule);

// Project: Allow USB for this specific project (exception)
let project_rule = PolicyRule::allow(
    DeviceType::Usb,
    PolicyLevel::Project,
)
.with_description("USB allowed for document scanning project");
policy_engine.add_rule(project_rule);
```

**Result:**
```
Platform: Block USB
Project:  Allow USB
Result:   USB ALLOWED (Project override wins)
```

---

### 4. Temporary Admin Override

**Scenario:** Developer needs urgent USB access for debugging, but it's normally blocked

**Configuration:**
```rust
// Block is in effect
policy_engine.check_access(DeviceType::Usb, Some(env_id), Some("bob@company.com"));
// Result: Blocked

// Admin grants 1-hour override
policy_engine.grant_override(Some(env_id), DeviceType::Usb, 3600);

// Within 1 hour, USB is now allowed
policy_engine.check_access(DeviceType::Usb, Some(env_id), Some("bob@company.com"));
// Result: Allowed (override in effect)
```

**Audit Trail:**
```json
{
  "event_type": "OverrideGranted",
  "device_type": "USB",
  "env_id": "debugging-vm",
  "timestamp": "2026-07-20T15:30:42Z",
  "context": "Override expires in 3600 seconds"
}
```

---

## API Reference

### Device Manager

```rust
let mut manager = DeviceManager::new();

// Check if device can be attached (without actually attaching)
let (can_attach, reason) = manager.can_attach(
    DeviceType::Serial, 
    env_id, 
    Some("user@company.com")
);

if !can_attach {
    println!("Cannot attach: {}", reason.unwrap());
}

// Attach device (policy checks happen automatically)
manager.attach(device_id, env_id, Some("user@company.com"))?;
```

### Policy Engine

```rust
let mut policy_engine = manager.policy_engine();

// Create and add policy rules
let rule = PolicyRule::block(
    DeviceType::Usb,
    PolicyLevel::Platform,
    BlockReason::Dlp,
);
policy_engine.add_rule(rule);

// Check access
let result = policy_engine.check_access(
    DeviceType::Usb,
    Some(env_id),
    Some("user@company.com")
);

println!("Allowed: {}", result.allowed);
println!("Message: {}", result.user_message());

// Get audit log
let events = policy_engine.get_audit_log();
for event in events {
    println!("{:?}", event);
}

// Export for compliance
let json = policy_engine.export_audit_log();
// Save to file for SOC/audit team
std::fs::write("compliance-audit.json", json)?;
```

### Compliance Reporting

```rust
// Generate compliance report for an environment
let report = manager.get_compliance_report(env_id);

println!("Total devices: {}", report.total_devices);
println!("Attached: {}", report.attached_devices);
println!("Active policies: {}", report.policy_rules);
println!("Audit events: {}", report.audit_events_count);
println!("Blocked attempts: {}", report.blocked_attempts);

// Export audit events as JSON
let audit_json = manager.export_audit_log();
// Send to compliance tool (Splunk, LogicMonitor, etc.)
send_to_compliance_system(audit_json)?;
```

---

## Configuration Examples

### Example 1: Financial Services (PCI-DSS)

```rust
// Block all removable media to prevent cardholder data theft
let rule = PolicyRule::block(
    DeviceType::Usb,
    PolicyLevel::Platform,
    BlockReason::Compliance,
)
.with_compliance_reference("PCI-DSS-2.2.1")
.with_description("USB blocked to protect payment card data");

// Allow serial for specific payment terminal
let exception = PolicyRule::allow(
    DeviceType::Serial,
    PolicyLevel::Project,
)
.with_description("Serial allowed for payment terminal integration");
```

### Example 2: Healthcare (HIPAA)

```rust
// Block camera to prevent unauthorized recording of patient information
let rule = PolicyRule::block(
    DeviceType::Camera,
    PolicyLevel::Platform,
    BlockReason::Compliance,
)
.with_compliance_reference("HIPAA-164.312(a)(2)(i)")
.with_description("Camera blocked to protect PHI");

// Require admin approval for USB (audit requirement)
let usb_rule = PolicyRule::block(
    DeviceType::Usb,
    PolicyLevel::Platform,
    BlockReason::Compliance,
)
.requires_approval()  // Must get admin approval to override
.with_compliance_reference("HIPAA-164.312(b)");
```

### Example 3: Multi-Tenant SaaS

```rust
// Each tenant isolated: can only access their own devices
let platform_rule = PolicyRule::block(
    DeviceType::Usb,
    PolicyLevel::Platform,
    BlockReason::MultiTenantIsolation,
)
.with_description("USB blocked for tenant isolation");

// Tenant A project: allow USB (within their own devices)
let tenant_a_rule = PolicyRule::allow(
    DeviceType::Usb,
    PolicyLevel::Project,
)
.with_description("USB allowed for Tenant A USB devices only");
// (Additional per-device authorization layer implemented at VM level)
```

---

## Developer Experience

### When Device Access is Allowed

```bash
$ tinybridge devices attach myvm /dev/ttyUSB0
✓ Successfully attached /dev/ttyUSB0 to myvm
  Device is accessible at /dev/ttyUSB0 inside the VM
```

**Behind the scenes:**
- Policy engine checked Platform > Project > VM > User rules
- All checks passed
- Audit event logged (allowed)
- Device attached

### When Device Access is Blocked

```bash
$ tinybridge devices attach myvm /dev/bus/usb/001/010
❌ Device passthrough is blocked: Data Loss Prevention policy 
  blocks this device (enforced by Platform policy) [SEC-2024-001]

Reason: USB devices blocked to prevent data exfiltration

This policy was created by: security@company.com
Compliance reference: SEC-2024-001

To request access:
  1. Explain your use case to your manager
  2. Manager approves and opens ticket with IT/Security
  3. Security team grants temporary override
  4. Use command: tinybridge devices grant-override myvm usb --duration 3600

Contact security-team@company.com for assistance
```

**Behind the scenes:**
- Policy engine checked rules
- Platform policy blocks USB
- Audit event logged (blocked)
- Error returned with actionable guidance

---

## Admin Operations

### List All Policies

```bash
$ tinybridge policy list
Policy Rules:
  [1] USB Device - BLOCK (Platform)
      Reason: Data Loss Prevention
      Reference: SEC-2024-001
      Created: 2026-07-01 by security@company.com
      Description: USB devices blocked to prevent data exfiltration
  
  [2] Serial Port - ALLOW (Project: payment-system)
      Created: 2026-07-05 by devops@company.com
      Description: Serial allowed for payment terminal
```

### Get Compliance Report

```bash
$ tinybridge policy report --env medical-app-vm
Compliance Report: medical-app-vm
Generated: 2026-07-20 15:30:42 UTC

Summary:
  Total Devices Registered: 12
  Devices Attached: 3
  Active Policies: 8
  Audit Events: 47
  Blocked Attempts: 3

Recent Activity:
  ✓ 2026-07-20 15:30:10 Camera blocked (HIPAA requirement)
  ✓ 2026-07-20 15:29:44 Serial attached (compliance logged)
  ✗ 2026-07-20 15:28:33 USB blocked (DLP policy)

Export for compliance:
  $ tinybridge policy export --format json > compliance-audit.json
  $ tinybridge policy export --format csv > compliance-audit.csv
```

### Grant Emergency Override

```bash
$ tinybridge policy override myvm usb --duration 3600 --reason "Emergency debugging"
✓ Override granted for 1 hour
  User can now attach USB devices to myvm
  Audit logged: Override granted by admin for emergency debugging
  Override expires: 2026-07-20 16:30:42 UTC
```

---

## Audit & Compliance

### Audit Event Structure

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "event_type": "AttachmentDenied",
  "device_id": null,
  "device_type": "USB",
  "policy_rule_id": "550e8400-e29b-41d4-a716-446655440001",
  "user_id": "alice@company.com",
  "env_id": "550e8400-e29b-41d4-a716-446655440002",
  "decision": "Block",
  "block_reason": "Dlp",
  "policy_level": "Platform",
  "timestamp": "2026-07-20T15:30:42.123456Z",
  "context": "USB devices blocked to prevent data exfiltration"
}
```

### Compliance Reports

```
Report Format: JSON (import to Splunk, DataDog, etc.)
Events: All attachment attempts, overrides, policy changes
Retention: Immutable audit trail (no deletion without audit)
Export: CSV, JSON, SIEM-native formats
Frequency: Real-time + daily snapshots
```

---

## Security Considerations

### Privilege Escalation Prevention
- User cannot override admin policies
- Admin override has expiration time
- All overrides logged to audit trail
- Override granting requires admin credentials

### Compliance Lock-In
- Policies can be marked as requiring admin approval to disable
- Compliance references prevent accidental policy removal
- Audit trail proves compliance enforcement

### Tenant Isolation
- Policy evaluation includes environment context
- Tenants cannot see other tenants' policies
- Device access scoped per tenant

---

## Future Enhancements

1. **Device Whitelist/Blacklist** — Specific USB VID:PID allow/block rules
2. **Time-Based Policies** — Allow USB only during business hours
3. **Risk Scoring** — Automatically block high-risk devices based on threat intel
4. **Integration with IAM** — Tie policies to AD/Okta/Ping Identity groups
5. **Webhook Notifications** — Alert SIEM when policies are violated
6. **Policy Templates** — Pre-built PCI-DSS, HIPAA, SOC2, ISO 27001 policies

---

## Summary

TinyBridge's device policy governance enables:

✅ **Developer Convenience:** Passthrough enabled by default  
✅ **Enterprise Security:** Comprehensive policy hierarchy and blocking reasons  
✅ **Compliance Ready:** Full audit trails, reporting, and compliance references  
✅ **Operational Control:** Admin overrides, policy enforcement, multi-tenant isolation  
✅ **Auditability:** Every device access decision logged and exportable  

**Result:** Perfect balance between developer productivity and enterprise governance.

---

*Device Policy & Governance is available in TinyBridge Phase 3.*
