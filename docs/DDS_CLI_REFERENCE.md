# DDS CLI Reference

Complete command reference for DDS networking management in TinyBridge.

---

## Global Status

### Check DDS Status Across All Environments

```bash
tinybridge dds list
```

Output:
```
DDS Status Summary:
├─ Total environments:     10
├─ DDS enabled:           2
├─ DDS disabled:          8
└─ Last modified:         2026-07-20 14:32:15 UTC

Enabled Environments:
├─ ros2-dev       (ROS 2 Full Profile)    ✓ Running
└─ production-bot (Multi-Robot Profile)   ✓ Running

Disabled Environments:
├─ general-vm-1   (Not required)
├─ general-vm-2   (Not required)
└─ ... (5 more)
```

---

## Per-Environment Commands

### Check DDS Status

```bash
tinybridge dds status <env>
```

**Output (DDS Disabled):**
```
DDS Status: Disabled (Recommended for general-purpose VMs)
├─ Enabled Features:     0/15
├─ Security:            N/A
├─ Audit Events:        0
└─ Reason to enable:    Only required for ROS 2 / robotics systems
```

**Output (DDS Enabled):**
```
DDS Status: Enabled (ROS 2 Full Profile)
├─ Enabled Features:     7/15
│  ├─ Discovery                     ✓
│  ├─ Multicast Discovery           ✓
│  ├─ Unicast Discovery             ✓
│  ├─ Monitoring                    ✓
│  ├─ Topic Inspection              ✓
│  ├─ Telemetry                     ✓
│  └─ Cross-host Communication      ✓
│
├─ Disabled Features:    8/15
│  ├─ Router                        ✗
│  ├─ Relay                         ✗
│  ├─ Bridge                        ✗
│  ├─ Packet Capture                ✗
│  ├─ Security                      ✗
│  ├─ WAN Communication             ✗
│  ├─ VPN Integration               ✗
│  └─ (1 more)
│
├─ Security:
│  ├─ Encryption:                   ✗ (disabled)
│  ├─ Authentication:               ✗ (disabled)
│  └─ Access Control:               ✗ (disabled)
│
├─ Network Configuration:
│  ├─ Primary Domain ID:            0
│  ├─ Multicast Addresses:          239.255.0.1
│  ├─ Discovery Ports:              7400-7402
│  ├─ Data Ports:                   7410-7412
│  └─ Firewall Rules:               ENABLED
│
└─ Audit History:
   ├─ Enabled By:                   alice@company.com
   ├─ Enabled At:                   2026-07-20 14:32:15 UTC
   └─ Total Events:                 3
```

---

### Enable DDS

#### Enable with Default Profile (Disabled)

```bash
tinybridge dds enable <env>
```

#### Enable with Specific Profile

```bash
# ROS 2 Minimal (discovery + local multicast only)
tinybridge dds enable <env> --profile ros2-minimal

# ROS 2 Full (monitoring, telemetry, diagnostics)
tinybridge dds enable <env> --profile ros2-full

# Multi-Robot (routers, relays, cross-host)
tinybridge dds enable <env> --profile multi-robot

# Industrial (security, WAN, VPN)
tinybridge dds enable <env> --profile industrial

# Custom (no auto-features)
tinybridge dds enable <env> --profile custom
```

#### Enable with Reason (Audit Trail)

```bash
tinybridge dds enable <env> --profile ros2-full \
  --reason "ROS 2 robotics development environment"
```

---

### Disable DDS

```bash
# Immediate disablement (no reconnection required)
tinybridge dds disable <env>

# With reason (audit trail)
tinybridge dds disable <env> --reason "Project completion, DDS no longer needed"

# Force (if stuck state)
tinybridge dds disable <env> --force
```

---

## Feature Management

### List Available Features

```bash
tinybridge dds features list
```

Output:
```
DDS Features (15 total):

Discovery:
├─ discovery                Enable/disable DDS participant discovery
├─ multicast_discovery      UDP multicast-based discovery
└─ unicast_discovery        Point-to-point unicast discovery

Routing:
├─ router                   DDS routers (forward messages between networks)
├─ relay                    DDS relays (alternative routing)
└─ bridge                   DDS bridges (connect domains)

Monitoring:
├─ monitoring               DDS introspection and monitoring
├─ topic_inspection         Read active topics and data
└─ packet_capture           Debug DDS messages (verbose)

Observability:
└─ telemetry                Collect metrics on DDS operations

Security:
└─ security                 DDS encryption, authentication plugins

Network:
├─ cross_host_communication Route DDS across multiple hosts
├─ wan_communication        Wide-area network support
└─ vpn_integration          Route DDS over VPN tunnels
```

### Enable Specific Feature

```bash
tinybridge dds feature enable <env> <feature>

# Examples:
tinybridge dds feature enable myenv discovery
tinybridge dds feature enable myenv router
tinybridge dds feature enable myenv encryption
```

### Disable Specific Feature

```bash
tinybridge dds feature disable <env> <feature>

# Examples:
tinybridge dds feature disable myenv multicast_discovery
tinybridge dds feature disable myenv wan_communication
```

### Enable Multiple Features at Once

```bash
tinybridge dds features enable <env> \
  discovery multicast_discovery monitoring \
  --reason "Enable ROS 2 with basic monitoring"
```

---

## Profile Management

### List Available Profiles

```bash
tinybridge dds profiles list
```

Output:
```
DDS Profiles:

disabled              DDS completely disabled (default)
                      Features: 0/15

ros2-minimal          Minimal ROS 2 setup
                      Features: discovery, multicast_discovery, unicast_discovery
                      Security: disabled

ros2-full             Full ROS 2 with monitoring
                      Features: discovery, multicast, unicast, monitoring,
                                topic_inspection, telemetry
                      Security: disabled

multi-robot           Multi-robot with cross-host
                      Features: all discovery, routing, monitoring
                      Security: encryption, authentication

industrial            Enterprise with WAN + security
                      Features: all except packet_capture
                      Security: encryption, authentication, access_control
                      Network: VPN integration enabled

custom                Manual feature control
                      Features: none (configure individually)
```

### Apply Profile

```bash
tinybridge dds apply-profile <env> <profile>

# Examples:
tinybridge dds apply-profile ros2-dev ros2-full
tinybridge dds apply-profile production-bot multi-robot
tinybridge dds apply-profile enterprise-vm industrial
```

---

## Security Configuration

### Enable Encryption

```bash
tinybridge dds security enable <env> --encryption

# Verify
tinybridge dds status <env> | grep -A5 Security
```

### Enable Authentication

```bash
tinybridge dds security enable <env> --authentication
```

### Enable Access Control

```bash
tinybridge dds security enable <env> --access-control
```

### Configure Allowed Domains

```bash
tinybridge dds security domain-allow <env> 0 1 2
```

### Configure Blocked Domains

```bash
tinybridge dds security domain-block <env> 10 11 12
```

### Whitelist Participants

```bash
tinybridge dds security participant-allow <env> \
  "robot-1" "sensor-hub" "controller"
```

### Blacklist Participants

```bash
tinybridge dds security participant-block <env> \
  "untrusted-node"
```

### Enable Network Isolation

```bash
tinybridge dds security isolate-domains <env>
```

---

## Policy Management

### Create Platform Policy (Block DDS)

```bash
tinybridge dds policy create \
  --level platform \
  --feature multicast_discovery \
  --decision block \
  --reason "Multicast not allowed on corporate network" \
  --compliance-ref "FIREWALL-POLICY-2024"
```

### Create Project Exception

```bash
tinybridge dds policy create \
  --level project \
  --project robotics-team \
  --feature discovery \
  --decision allow \
  --reason "Research project requires DDS"
```

### Create Environment Override

```bash
tinybridge dds policy create \
  --level environment \
  --env production-robot \
  --feature wan_communication \
  --decision allow \
  --reason "Multi-site robot coordination"
```

### List Policies

```bash
tinybridge dds policy list
```

Output:
```
DDS Policy Rules (3 total):

[1] Platform Policy
    Feature:       multicast_discovery
    Decision:      BLOCK
    Reason:        SecurityPolicy
    Compliance:    FIREWALL-POLICY-2024
    Created:       2026-07-01 10:15:00 UTC
    Created By:    security-admin@company.com

[2] Project Exception
    Level:         Project (robotics-team)
    Feature:       discovery
    Decision:      ALLOW
    Created:       2026-07-05 14:32:00 UTC
    Created By:    project-lead@company.com

[3] Environment Override
    Level:         Environment (production-robot)
    Feature:       wan_communication
    Decision:      ALLOW
    Expires:       Never
    Created:       2026-07-10 09:00:00 UTC
```

### Delete Policy

```bash
tinybridge dds policy delete <policy-id>

# Example:
tinybridge dds policy delete 550e8400-e29b-41d4-a716-446655440000
```

---

## Temporary Overrides

### Grant Temporary Access

```bash
# Grant 1-hour override
tinybridge dds override grant <env> <feature> \
  --duration 3600 \
  --reason "Emergency debugging"

# Grant 24-hour override
tinybridge dds override grant <env> <feature> \
  --duration 86400 \
  --reason "Special event coordination"

# Grant indefinite override (requires confirmation)
tinybridge dds override grant <env> <feature> \
  --duration unlimited \
  --reason "Approved exception" \
  --approve
```

### List Active Overrides

```bash
tinybridge dds override list
```

Output:
```
Active DDS Overrides (2 total):

[1] Environment: debugging-robot
    Feature:     wan_communication
    Granted By:  alice@company.com
    Granted At:  2026-07-20 14:32:15 UTC
    Expires In:  57m 42s
    Reason:      Emergency debugging of multi-site setup

[2] Environment: research-vm
    Feature:     packet_capture
    Granted By:  bob@company.com
    Granted At:  2026-07-20 15:00:00 UTC
    Expires In:  23h 59m 10s
    Reason:      Performance analysis for publication
```

### Revoke Override

```bash
tinybridge dds override revoke <env> <feature>

# Example:
tinybridge dds override revoke debugging-robot wan_communication
```

---

## Audit & Compliance

### Export Audit Log

```bash
# JSON format
tinybridge dds audit export <env> --format json > dds-audit.json

# CSV format
tinybridge dds audit export <env> --format csv > dds-audit.csv

# Human-readable
tinybridge dds audit export <env> --format text
```

### Generate Compliance Report

```bash
tinybridge dds compliance report <env>
```

Output:
```
DDS Compliance Report for: ros2-dev
Generated: 2026-07-20 16:45:30 UTC

Summary:
├─ DDS Enabled:              YES
├─ Enabled Features:         7/15
├─ Security Configuration:   Partial (encryption disabled)
├─ Audit Events:             12
└─ Last Change:              2026-07-20 14:32:15 UTC (alice@company.com)

Policy Compliance:
├─ Platform Policies:        1 rule
├─ Project Policies:         2 rules
├─ Environment Policies:     0 rules
├─ Active Overrides:         0
└─ Status:                   COMPLIANT ✓

Features Status:
├─ Discovery:                Enabled (required for ROS 2)
├─ Multicast Discovery:      Enabled
├─ Unicast Discovery:        Enabled
├─ Router:                   Disabled
├─ Relay:                    Disabled
├─ Bridge:                   Disabled
├─ Monitoring:               Enabled
├─ Topic Inspection:         Enabled
├─ Packet Capture:           Disabled
├─ Telemetry:                Enabled
├─ Security:                 Disabled ⚠️  Recommended to enable
├─ Cross-host Comm:          Disabled
├─ WAN Comm:                 Disabled
├─ VPN Integration:          Disabled
└─ (1 more disabled)

Security:
├─ Encryption:               Disabled
├─ Authentication:           Disabled
├─ Access Control:           Disabled
└─ Recommendation:           Enable encryption for production

Audit Trail (last 5 events):
├─ 2026-07-20 14:32:15  DDS Enabled by alice@company.com
├─ 2026-07-20 14:33:22  Feature enabled: discovery
├─ 2026-07-20 14:33:45  Feature enabled: monitoring
├─ 2026-07-20 14:34:10  Feature enabled: telemetry
└─ 2026-07-20 14:34:55  Profile applied: ros2-full
```

### Export Full Compliance Package (SOC 2 / ISO 27001)

```bash
tinybridge dds compliance export \
  --environments all \
  --format comprehensive \
  --output compliance-package-2026-07-20.zip
```

Includes:
- DDS configuration audit trail (all VMs)
- Policy hierarchy documentation
- Security compliance checklist
- Override approval records
- Network configuration proof
- Encryption/authentication verification

---

## Troubleshooting

### DDS Not Discovering (Multicast Issues)

```bash
# Verify multicast is enabled
tinybridge dds feature status <env> multicast_discovery

# Check firewall rules
tinybridge dds network show <env>

# Enable VPN integration if on corporate network
tinybridge dds feature enable <env> vpn_integration

# Debug: packet capture to verify traffic
tinybridge dds feature enable <env> packet_capture
tinybridge dds debug packets <env> --duration 60
```

### DDS Performance Issues

```bash
# Enable telemetry for diagnostics
tinybridge dds feature enable <env> telemetry

# Check active domains
tinybridge dds domains list <env>

# Reduce message size if needed
tinybridge dds network config <env> --max-message-size 8192

# Enable monitoring for latency tracking
tinybridge dds feature enable <env> monitoring
tinybridge dds metrics <env>
```

### DDS Security Warnings

```bash
# Enable encryption
tinybridge dds security enable <env> --encryption

# Enable authentication
tinybridge dds security enable <env> --authentication

# Whitelist trusted participants
tinybridge dds security participant-allow <env> "known-robot-1"

# Check security status
tinybridge dds status <env> | grep -A5 Security
```

---

## Examples

### Setup: Development ROS 2 Environment

```bash
# 1. Create environment with DDS minimal
tinybridge dds enable ros2-dev --profile ros2-minimal

# 2. Verify status
tinybridge dds status ros2-dev

# 3. Enable monitoring for development
tinybridge dds feature enable ros2-dev monitoring
tinybridge dds feature enable ros2-dev topic_inspection

# 4. Export audit log
tinybridge dds audit export ros2-dev --format json
```

### Setup: Production Multi-Robot System

```bash
# 1. Apply industrial profile with security
tinybridge dds apply-profile production-fleet industrial

# 2. Enable encryption and authentication
tinybridge dds security enable production-fleet --encryption --authentication

# 3. Configure allowed participants
tinybridge dds security participant-allow production-fleet \
  "primary-robot" "secondary-robot" "control-station"

# 4. Enable VPN integration
tinybridge dds feature enable production-fleet vpn_integration

# 5. Verify compliance
tinybridge dds compliance report production-fleet

# 6. Generate compliance package
tinybridge dds compliance export --environments production-fleet
```

### Setup: Enterprise Policy

```bash
# 1. Block multicast at platform level
tinybridge dds policy create \
  --level platform \
  --feature multicast_discovery \
  --decision block \
  --reason "Security: multicast not allowed" \
  --compliance-ref "SEC-2024-001"

# 2. Allow exception for research team
tinybridge dds policy create \
  --level project \
  --project research-robotics \
  --feature multicast_discovery \
  --decision allow

# 3. Temporarily override for emergency
tinybridge dds override grant emergency-bot multicast_discovery \
  --duration 3600 \
  --reason "Emergency response coordination"

# 4. Verify policies
tinybridge dds policy list
tinybridge dds override list
```

---

*DDS CLI is available in TinyBridge Phase 3A (Weeks 15-18).*
