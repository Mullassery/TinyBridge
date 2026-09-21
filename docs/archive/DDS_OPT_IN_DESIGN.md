# DDS Networking: Opt-In Design with Explicit Administrator Control

**Status:** Design & Implementation  
**Security Level:** Enterprise-Grade  
**Principle:** Least Privilege (Disabled by Default)

---

## Overview

TinyBridge treats DDS (Data Distribution Service) networking as an **optional, specialized capability** that is **disabled by default** for all virtual machines and environments. Users and administrators must explicitly enable DDS functionality through dedicated controls.

This document describes the design, implementation, and operational model for DDS networking in TinyBridge.

---

## Core Design Principle: Least Privilege

### Default Behavior

Upon environment creation:

- ✅ **DDS networking:** Disabled
- ✅ **DDS discovery:** Disabled
- ✅ **DDS multicast:** Blocked
- ✅ **DDS routers:** Disabled
- ✅ **DDS relays:** Disabled
- ✅ **DDS bridges:** Disabled
- ✅ **DDS monitoring:** Disabled
- ✅ **DDS telemetry:** Disabled
- ✅ **DDS security:** Disabled (until DDS itself is enabled)
- ✅ **DDS ports:** Not opened
- ✅ **Firewall rules:** Not created

### Result

The system behaves as a standard VM/network environment with **zero DDS overhead** until DDS is intentionally enabled.

---

## Explicit Enablement Controls

Each DDS capability is independently controllable:

### Discovery Controls
- **DDS Discovery** — Enable/disable participant discovery entirely
- **Multicast Discovery** — UDP multicast for local discovery
- **Unicast Discovery** — Point-to-point discovery

### Routing Controls
- **DDS Router** — Forward DDS messages between networks
- **DDS Relay** — Similar to routers, different implementation
- **DDS Bridge** — Connect different DDS domains

### Monitoring & Observability
- **DDS Monitoring** — Introspection and dashboard support
- **Topic Inspection** — Read active topics and their data
- **Packet Capture** — Debug DDS messages (verbose)
- **DDS Telemetry** — Collect metrics on DDS operations

### Security
- **DDS Security Plugins** — Encryption and authentication

### Network
- **Cross-Host Communication** — Route DDS across multiple hosts
- **WAN Communication** — Wide-area network support
- **VPN Integration** — Route DDS over VPN tunnels

**Total: 15 independent toggles** — Granular control without forcing bulk enablement.

---

## Configuration Model

### Hierarchy (Top-Down Override)

```
Platform Policy (highest priority)
    ↓ [can override]
Project Policy
    ↓ [can override]
Environment Policy
    ↓ [can override]
User Preference
    ↓ [default: allow if no policies]
System Default (disabled)
```

**Example:**
```
Platform: Block DDS (security policy)
Project:  Allow DDS for robotics project
Environment: (no explicit policy)
User: (no preference)

Result: DDS ALLOWED (Project overrides Platform)
```

### Profiles (Convenience Pre-Configurations)

For common use cases, pre-configured profiles are available:

**Disabled Profile** (default)
```yaml
dds:
  enabled: false
  # All features disabled
```

**ROS 2 Minimal Profile**
```yaml
dds:
  enabled: true
  features:
    discovery: true
    multicast_discovery: true
    unicast_discovery: true
    # Routers/relays/bridges disabled
```

**ROS 2 Full Profile**
```yaml
dds:
  enabled: true
  features:
    discovery: true
    multicast_discovery: true
    unicast_discovery: true
    monitoring: true
    topic_inspection: true
    telemetry: true
    # Routers/relays stay disabled
```

**Multi-Robot Profile**
```yaml
dds:
  enabled: true
  features:
    discovery: true
    multicast_discovery: true
    unicast_discovery: true
    router: true
    relay: true
    bridge: true
    cross_host_communication: true
    monitoring: true
  security:
    encryption: true
    authentication: true
```

**Industrial Profile**
```yaml
dds:
  enabled: true
  features:
    # All discovery and routing enabled
    discovery: true
    multicast_discovery: true
    unicast_discovery: true
    router: true
    relay: true
    wan_communication: true
    vpn_integration: true
    monitoring: true
    telemetry: true
  security:
    encryption: true
    authentication: true
    access_control: true
  networking:
    vpn_traffic_allowed: true
```

---

## Security Configuration

When DDS is enabled, security controls are available:

### Encryption & Authentication
- **Encryption:** Encrypted DDS communications
- **Authentication:** Participant identity verification
- **Access Control:** Fine-grained permission policies

### Domain Isolation
- **Allowed Domains:** Whitelist DDS domain IDs
- **Blocked Domains:** Blacklist specific domains
- **Network Isolation:** Separate network namespaces per domain

### Participant Control
- **Allowlist Mode:** Only listed participants can join (default)
- **Denylist Mode:** Block specific participants
- **Participant Filtering:** Control which parties can discover each other

---

## Operational Controls

### Per-Environment Control

```bash
# Enable DDS for specific environment
tinybridge dds enable myenv
tinybridge dds enable myenv --profile ros2-full

# Disable DDS immediately
tinybridge dds disable myenv

# Toggle specific feature
tinybridge dds feature enable myenv discovery
tinybridge dds feature disable myenv wan-communication

# Apply security policy
tinybridge dds security enable myenv --encryption --authentication

# Check current state
tinybridge dds status myenv
```

### Per-Project Control

```bash
# Enable DDS for all environments in project
tinybridge dds enable project:robotics-team

# Apply profile to project
tinybridge dds apply-profile project:robotics-team ros2-full

# Project-level policy (override defaults)
tinybridge dds policy create \
  --level project \
  --project robotics-team \
  --feature discovery \
  --decision allow
```

### Platform-Wide Control

```bash
# Block DDS across all environments (security policy)
tinybridge dds policy create \
  --level platform \
  --feature multicast_discovery \
  --decision block \
  --reason "Multicast not allowed on corporate network"

# Allow specific project exception
tinybridge dds policy create \
  --level project \
  --project research-lab \
  --feature multicast_discovery \
  --decision allow
```

### Temporary Admin Override

```bash
# Grant 1-hour override for debugging
tinybridge dds override grant myenv discovery --duration 3600 \
  --reason "Emergency debugging of DDS topics"

# List active overrides
tinybridge dds override list

# Revoke override
tinybridge dds override revoke myenv discovery
```

---

## Transparency Before Enablement

Before enabling DDS, the system displays a comprehensive summary:

```
┌─ Enable DDS for: myenv ────────────────┐
│                                        │
│ What is DDS?                           │
│ DDS (Data Distribution Service) is a  │
│ pub-sub middleware used by ROS 2,     │
│ robotics systems, and real-time apps. │
│ Not required for general VMs.         │
│                                        │
│ Network Impact:                        │
│ • UDP multicast: 239.255.0.1:7400     │
│ • 3 discovery ports: 7400-7402        │
│ • Data ports: 7410-7412               │
│ • Firewall rules: WILL BE OPENED      │
│                                        │
│ Discovery Behavior:                    │
│ • Local multicast announcements       │
│ • Other VMs on network will see this  │
│ • Participants can discover each other│
│                                        │
│ Firewall Implications:                 │
│ ⚠️  UDP multicast will be enabled      │
│ ⚠️  Corporate firewall may block this  │
│ ⚠️  Notify IT if on enterprise network│
│                                        │
│ VPN Compatibility:                     │
│ ⚠️  Multicast may not work over VPN    │
│ ℹ️  VPN integration available (enable) │
│ ℹ️  Can tunnel DDS over VPN if needed  │
│                                        │
│ Security Implications:                 │
│ ⚠️  DDS participants not authenticated │
│ ✓  Enable DDS security for production │
│ ✓  Encryption recommended             │
│                                        │
│ [  Enable  ] [ Cancel ]                │
│                                        │
└────────────────────────────────────────┘
```

---

## Compliance Reporting

### Compliance Report Section

```
DDS Networking Status:
├─ Feature                        Status
├─ DDS Networking                 Disabled ✓
├─ DDS Discovery                  Disabled ✓
├─ DDS Multicast                  Disabled ✓
├─ DDS Router                     Disabled ✓
├─ DDS Relay                      Disabled ✓
├─ DDS Monitoring                 Disabled ✓
├─ DDS Telemetry                  Disabled ✓
├─ DDS Security                   Disabled ✓
└─ Total Enabled Features         0/15

Audit Trail:
├─ Last Change:                   2026-07-20 14:32:15 UTC
├─ Changed By:                    alice@company.com
├─ Reason:                        ROS 2 robotics project enablement
├─ Admin Approval:                Required (pending)
└─ Total Audit Events:            3
```

### Audit Log Export

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "env_id": "550e8400-e29b-41d4-a716-446655440001",
    "event_type": "DdsEnabled",
    "timestamp": "2026-07-20T14:32:15.123456Z",
    "changed_by": "alice@company.com",
    "change_details": "DDS networking enabled",
    "old_value": "false",
    "new_value": "true",
    "change_reason": "ROS 2 robotics project enablement",
    "requires_admin_to_undo": false
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "env_id": "550e8400-e29b-41d4-a716-446655440001",
    "event_type": "FeatureChanged",
    "timestamp": "2026-07-20T14:33:22.123456Z",
    "changed_by": "alice@company.com",
    "change_details": "discovery feature toggled",
    "old_value": "false",
    "new_value": "true",
    "change_reason": null,
    "requires_admin_to_undo": false
  }
]
```

---

## User Experience

### When DDS is Disabled (Default)

```bash
$ tinybridge status myenv

Environment: myenv
Status:      Running
Uptime:      2h 34m
IP:          192.168.64.42

Network Status:
  DDS:       Disabled (Recommended)
  ℹ️  DDS is not required for most VMs
  ℹ️  Enable only if using ROS 2 or similar
  ℹ️  To enable: tinybridge dds enable myenv
```

### When DDS is Disabled BUT Requested

```bash
$ tinybridge dds status myenv
DDS is disabled for this environment.

DDS is typically required for:
  • ROS 2 applications
  • Robotics systems
  • Autonomous vehicles
  • Industrial automation
  • Distributed real-time systems

For general-purpose VMs, DDS is unnecessary.

To enable DDS, run:
  tinybridge dds enable myenv --profile ros2-minimal
  tinybridge dds enable myenv --profile ros2-full
  tinybridge dds enable myenv --profile multi-robot
```

### When DDS is Enabled

```bash
$ tinybridge dds status myenv

DDS Status: Enabled (ROS 2 Full Profile)
├─ Enabled Features: 7/15
│  ├─ Discovery                 ✓
│  ├─ Multicast Discovery       ✓
│  ├─ Unicast Discovery         ✓
│  ├─ Monitoring                ✓
│  ├─ Topic Inspection          ✓
│  ├─ Telemetry                 ✓
│  └─ (8 features disabled)
│
├─ Security:
│  ├─ Encryption                ✗ (disabled)
│  ├─ Authentication            ✗ (disabled)
│  └─ ⚠️  Recommend enabling encryption
│
├─ Network:
│  ├─ Primary Domain ID          0
│  ├─ Multicast Addresses        239.255.0.1
│  ├─ Discovery Ports            7400-7402
│  ├─ Data Ports                 7410-7412
│  └─ Firewall Rules             ENABLED (ports opened)
│
└─ Audit Trail:
   ├─ Enabled By:               alice@company.com
   ├─ Enabled At:               2026-07-20 14:32:15 UTC
   └─ Reason:                   ROS 2 development environment
```

---

## Security Model

### Threat Model

**Threats Mitigated:**

1. **Accidental DDS Enablement** — Users don't accidentally enable DDS
2. **Unauthorized DDS Access** — Only approved participants can join
3. **DDS Misconfiguration** — Security controls prevent common mistakes
4. **Compliance Violations** — Audit trail proves compliance
5. **Unauthorized Cross-VM Communication** — Network isolation prevents uncontrolled DDS

### Control Implementation

| Threat | Control |
|--------|---------|
| Accidental enablement | Default-disabled, explicit control |
| Unauthorized access | Participant allowlists, authentication |
| Misconfiguration | Pre-built profiles, validation |
| Compliance violations | Immutable audit log, export formats |
| Cross-VM communication | Network isolation per domain |

---

## Compliance Integration

### SOC 2 / ISO 27001 / PCI-DSS

DDS opt-in design satisfies compliance requirements:

✅ **Configuration Control** — All DDS changes audited  
✅ **Principle of Least Privilege** — Default-disabled  
✅ **Segregation of Duties** — Admins control policies  
✅ **Non-Repudiation** — Immutable audit trail  
✅ **Access Control** — Policy hierarchy + approval workflows  
✅ **Encryption** — Available when enabled  
✅ **Network Segmentation** — Domain isolation available

### Audit Requirements

Compliance teams can:

1. **Export Audit Log** — JSON/CSV for SIEM/compliance tools
2. **Generate Reports** — DDS status across all environments
3. **Verify Policies** — Policy hierarchy documentation
4. **Trace Changes** — Who enabled DDS, when, why
5. **Prove Compliance** — No unauthorized DDS communications

---

## Implementation Roadmap

### Phase 3A (Weeks 15-18)

**Week 15-16: Core DDS Configuration**
- [ ] DDS configuration module (tinybridge-core)
- [ ] DDS manager (tinybridge-dds crate)
- [ ] Policy engine for hierarchical control
- [ ] Audit logging infrastructure
- [ ] CLI commands for DDS management
- [ ] 15+ unit tests

**Week 17-18: Integration & Documentation**
- [ ] Integration with environment manager
- [ ] Compliance reporting integration
- [ ] CLI/UI for DDS management
- [ ] Comprehensive documentation
- [ ] Security policy examples
- [ ] Transition guides for ROS 2 workflows

### Future (Phase 4-5)

- DDS security plugins (encryption, authentication)
- Advanced monitoring and diagnostics
- DDS-aware firewall rule management
- Automatic VPN integration
- Multi-domain coordination

---

## Configuration Examples

### Example 1: Development Environment (ROS 2)

```yaml
# env.yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: ros2-dev
  version: "1.0.0"
dds:
  profile: ros2-full
  security:
    encryption: false  # Development only
    authentication: false
  networking:
    vpn_traffic_allowed: false
```

### Example 2: Production Robot (Multi-Robot)

```yaml
# env.yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: production-robot
  version: "2.0.0"
dds:
  profile: multi-robot
  security:
    encryption: true
    authentication: true
    access_control: true
    allowed_domains: [0, 1]
  networking:
    vpn_traffic_allowed: true
    cross_host_communication: true
    wan_communication: true
```

### Example 3: Enterprise Policy (Blocked)

```bash
# Platform-wide policy
tinybridge dds policy create \
  --level platform \
  --feature multicast_discovery \
  --decision block \
  --reason "Multicast not allowed on corporate network" \
  --compliance_reference "FIREWALL-POLICY-2024"

# Project exception for research team
tinybridge dds policy create \
  --level project \
  --project research-robotics \
  --feature multicast_discovery \
  --decision allow \
  --reason "Research project requires DDS"
```

---

## Testing Strategy

**Unit Tests:**
- DDS configuration creation (all features default-disabled)
- Feature toggles work independently
- Profiles apply correct settings
- Audit logging captures changes
- Policy hierarchy enforces correctly
- Overrides expire properly

**Integration Tests:**
- Environment creation with DDS disabled
- Profile application updates compliance report
- Audit events exported correctly
- Policy conflicts resolve properly

**Security Tests:**
- Cannot enable DDS without explicit action
- Cannot bypass security policies
- Override expiration prevents permanent bypass
- Audit log cannot be modified

---

## Summary

TinyBridge's DDS opt-in design provides:

✅ **Security First** — Default-disabled, no automatic enablement  
✅ **Transparency** — Users understand DDS before enabling  
✅ **Auditability** — Complete immutable audit trail  
✅ **Compliance Ready** — Satisfies SOC 2 / ISO 27001 / PCI-DSS  
✅ **Operational Control** — Admins enforce policies at all levels  
✅ **Enterprise Friendly** — Platform, project, environment, and user controls  

**Result:** DDS is treated as a specialized networking capability, not a platform default. Teams building robotics systems get full DDS support with production-grade audit trails. General-purpose VMs remain unaffected and lightweight.

---

*DDS opt-in design is available in TinyBridge Phase 3A (Weeks 15-18).*
