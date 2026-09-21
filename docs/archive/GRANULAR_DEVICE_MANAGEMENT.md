# Granular Device Management Controls

**Status:** Design & Implementation Plan  
**Priority:** Phase 3 Enhancement  
**Scope:** 20+ device types and services with independent controls

---

## Overview

Complete administrator control over every VM device and service with independent Enable/Disable toggles, global defaults, per-VM overrides, and comprehensive audit trails.

---

## Device Categories (20+ Configurable Items)

### Virtual Devices (7)
- [ ] Virtual Network Adapter (vNIC)
- [ ] Virtual Disk Controller (NVMe/SATA/SCSI)
- [ ] Virtual GPU Acceleration
- [ ] Virtual Sound Device
- [ ] Virtual USB Controller
- [ ] Virtual TPM 2.0
- [ ] Virtual Smart Card Reader

### Passthrough Devices (5)
- [ ] PCIe Device Passthrough
- [ ] GPU Passthrough
- [ ] Network Adapter Passthrough
- [ ] Storage Controller Passthrough
- [ ] USB Device Passthrough (Dynamic)

### Host Integration Services (4)
- [ ] File Sharing / VirtioFS
- [ ] Clipboard Synchronization
- [ ] Host-Guest Communication Channels
- [ ] Host Shared Folders

### Observability & Management (4)
- [ ] VM Telemetry & Diagnostics
- [ ] Performance Monitoring Agent
- [ ] Network Monitoring Service
- [ ] System Audit Logging

---

## Implementation Architecture

```rust
pub struct DeviceControl {
    pub device_id: String,
    pub device_name: String,
    pub device_category: DeviceCategory,
    pub enabled: bool,
    pub requires_vm_restart: bool,
    pub default_enabled: bool,
    pub hardened_default: bool,  // Disabled by default in hardened mode
    pub security_sensitive: bool,
    pub last_modified: DateTime<Utc>,
    pub modified_by: Option<String>,
}

pub enum DeviceCategory {
    Virtual,
    Passthrough,
    HostIntegration,
    ObservabilityAndManagement,
}

pub struct DeviceConfigurationProfile {
    pub name: String,  // Minimal, Development, Enterprise, HighPerformance, Custom
    pub devices: HashMap<String, bool>,
    pub description: Option<String>,
}

pub struct VmDeviceConfiguration {
    pub vm_id: Uuid,
    pub global_overrides: Option<DeviceConfigurationProfile>,
    pub devices: HashMap<String, DeviceControl>,
    pub profile: Option<String>,
}
```

---

## Control Hierarchy

```
Global Defaults (Platform Level)
    ↓ [can override]
VM Template Defaults (if using template)
    ↓ [can override]
Per-VM Configuration (highest priority)
    ↓
Active Device State
```

---

## Feature Details

### 1. Independent Enable/Disable
Each device has its own toggle that can be changed independently.

**Example:**
```bash
# Enable GPU passthrough only
tinybridge device enable gpu-passthrough --vm myvm
tinybridge device disable file-sharing --vm myvm
tinybridge device disable clipboard --vm myvm
# Result: GPU works, file sharing off, clipboard off
```

### 2. No VM Recreation Required
Most changes apply without VM recreation. VM restarts only when necessary.

**Device Change Chart:**
```
Device                              | Requires Restart?
─────────────────────────────────────────────────────
Virtual Network Adapter             | Yes (network config change)
Virtual Disk Controller             | Yes (storage change)
GPU Acceleration                    | No (driver unload/load)
Sound Device                        | No (can disable dynamically)
USB Controller                      | No
TPM 2.0                            | Yes (firmware)
Smart Card Reader                   | No
Passthrough (GPU/NIC/Storage)       | Yes (PCI binding)
File Sharing                        | No
Clipboard                          | No
Telemetry                          | No
Monitoring Agent                    | No
```

### 3. Global Defaults + Per-VM Overrides

**Global Platform Defaults:**
```yaml
device_defaults:
  virtual_network: enabled
  gpu_acceleration: enabled
  file_sharing: enabled
  clipboard: enabled
  
  # Disabled by default (sensitive)
  gpu_passthrough: disabled
  network_passthrough: disabled
  storage_passthrough: disabled
  tpm_2_0: disabled
  smart_card: disabled
  telemetry: disabled
  monitoring: disabled
```

**Per-VM Override:**
```yaml
# Override for this VM: enable GPU passthrough
vm_devices:
  myvm:
    gpu_passthrough: enabled
    # All other devices use platform defaults
```

### 4. Security-Sensitive Features Disabled by Default

In **hardened environments**, security-sensitive features are disabled:

```rust
pub struct HardenedProfile {
    disabled_by_default: vec![
        "GpuPassthrough",
        "NetworkPassthrough",
        "StoragePassthrough",
        "Tpm2_0",
        "SmartCardReader",
        "Telemetry",
        "MonitoringAgent",
        "NetworkMonitoring",
        "AuditLogging",
    ],
}
```

### 5. Device Profiles

Pre-configured templates for common use cases:

**Minimal Profile** (lowest overhead):
```yaml
devices:
  virtual_network: enabled
  virtual_disk: enabled
  gpu_acceleration: disabled
  sound: disabled
  usb_controller: disabled
  file_sharing: disabled
  clipboard: disabled
  telemetry: disabled
  monitoring: disabled
  # All passthrough: disabled
```

**Development Profile** (best experience):
```yaml
devices:
  # All virtual devices: enabled
  virtual_network: enabled
  virtual_disk: enabled
  gpu_acceleration: enabled
  sound: enabled
  usb_controller: enabled
  file_sharing: enabled
  clipboard: enabled
  
  # Passthrough: disabled
  gpu_passthrough: disabled
  
  # Telemetry: enabled
  telemetry: enabled
  monitoring: enabled
```

**Enterprise Profile** (balanced):
```yaml
devices:
  # Virtual devices: enabled
  virtual_network: enabled
  virtual_disk: enabled
  gpu_acceleration: enabled
  
  # Passthrough: disabled (unless approved)
  gpu_passthrough: disabled
  
  # Management: enabled with audit
  telemetry: enabled
  monitoring: enabled
  audit_logging: enabled
  
  # File sharing: disabled for security
  file_sharing: disabled
  clipboard: disabled
```

**High Performance Profile** (for workstations):
```yaml
devices:
  # All virtual devices: enabled
  virtual_network: enabled
  virtual_disk: enabled
  gpu_acceleration: enabled
  sound: enabled
  usb_controller: enabled
  
  # Passthrough: enabled
  gpu_passthrough: enabled
  network_passthrough: enabled
  storage_passthrough: enabled
  
  # File sharing: enabled for performance
  file_sharing: enabled
  clipboard: enabled
```

---

## User Experience

### CLI Commands

```bash
# List all devices and status
$ tinybridge device list --vm myvm
Device Status for myvm:

Virtual Devices:
  ✓ Virtual Network Adapter      (enabled)
  ✓ Virtual Disk Controller      (enabled)
  ✓ GPU Acceleration             (enabled)
  ✗ Sound Device                 (disabled)
  ✓ USB Controller               (enabled)
  ✗ TPM 2.0                      (disabled, requires restart)
  ✗ Smart Card Reader            (disabled)

Passthrough Devices:
  ✗ GPU Passthrough              (disabled, requires restart)
  ✗ Network Passthrough          (disabled, requires restart)
  ✗ Storage Passthrough          (disabled, requires restart)

Host Integration:
  ✓ File Sharing                 (enabled)
  ✓ Clipboard                    (enabled)
  ✓ Host Communication           (enabled)

Management & Observability:
  ✓ Telemetry                    (enabled)
  ✓ Monitoring Agent             (enabled)
  ✓ Network Monitoring           (enabled)
  ✓ Audit Logging                (enabled)

# Enable a device
$ tinybridge device enable gpu-passthrough --vm myvm
✓ GPU passthrough enabled
  ⚠️  Requires VM restart to take effect
  Run: tinybridge restart myvm

# Disable a device
$ tinybridge device disable clipboard --vm myvm
✓ Clipboard disabled
  ✓ Change takes effect immediately

# Apply a profile
$ tinybridge device apply-profile minimal --vm myvm
✓ Minimal profile applied to myvm
  Disabled 8 devices (takes effect immediately)
  Requires restart for: Virtual Disk Controller

# Check which changes need restart
$ tinybridge device changes --vm myvm --pending-restart
Pending restart required:
  • Virtual Disk Controller: enabled
  • GPU Passthrough: enabled
  • Network Passthrough: disabled
```

### UI Display

```
┌─ VM Device Configuration: myvm ────────────────────┐
│                                                    │
│ Profile: Development                               │
│ │                                                 │
│ VIRTUAL DEVICES                                    │
│  □ Virtual Network Adapter        [✓ Enabled]    │
│  □ Virtual Disk (NVMe)            [✓ Enabled]    │
│  □ GPU Acceleration               [✓ Enabled]    │
│  □ Sound Device                   [✗ Disabled]   │
│  □ USB Controller                 [✓ Enabled]    │
│  □ TPM 2.0                        [✗ Disabled]   │
│  □ Smart Card Reader              [✗ Disabled]   │
│ │                                                 │
│ PASSTHROUGH DEVICES (Security: Disabled by Default)
│  □ GPU Passthrough                [✗ Disabled]   │
│    ⚠️  Requires admin approval + restart           │
│  □ Network Passthrough            [✗ Disabled]   │
│    ⚠️  Requires admin approval + restart           │
│  □ Storage Passthrough            [✗ Disabled]   │
│    ⚠️  Requires admin approval + restart           │
│ │                                                 │
│ HOST INTEGRATION                                   │
│  ✓ File Sharing (VirtioFS)        [✓ Enabled]   │
│  ✓ Clipboard Sync                 [✓ Enabled]   │
│  ✓ Host Communication             [✓ Enabled]   │
│ │                                                 │
│ MANAGEMENT & OBSERVABILITY                        │
│  ✓ Telemetry                      [✓ Enabled]   │
│  ✓ Performance Monitoring         [✓ Enabled]   │
│  ✓ Network Monitoring             [✓ Enabled]   │
│  ✓ Audit Logging                  [✓ Enabled]   │
│ │                                                 │
│ [Apply Default Profile] [Save Custom Profile]    │
│ [Requires Restart: Disk Controller, GPU]         │
│ [Request Admin Approval for Passthrough Devices]│
│                                                    │
└────────────────────────────────────────────────────┘
```

---

## Audit Trail

Every device enable/disable action logged:

```json
{
  "event_type": "DeviceConfigurationChanged",
  "vm_id": "uuid",
  "device": "gpu-passthrough",
  "action": "enabled",
  "requires_restart": true,
  "requested_by": "alice@company.com",
  "timestamp": "2026-07-20T16:30:42.123456Z",
  "policy_checked": true,
  "approval_required": true,
  "approval_granted_by": "security-admin@company.com"
}
```

---

## Implementation Phases

### Phase 1: Core Device Control System
- [ ] DeviceControl struct and APIs
- [ ] Enable/Disable logic per device
- [ ] Configuration persistence
- [ ] Restart requirement detection

### Phase 2: UI & CLI
- [ ] CLI commands (list, enable, disable)
- [ ] Web UI device panel
- [ ] Status display
- [ ] Audit log viewer

### Phase 3: Profiles & Templates
- [ ] Predefined profiles (Minimal, Development, Enterprise, HighPerformance)
- [ ] Custom profile creation
- [ ] Template application
- [ ] Default management

### Phase 4: Integration with Policy Engine
- [ ] Device passthrough policies
- [ ] Security-sensitive approval workflows
- [ ] Compliance enforcement
- [ ] SIEM export

---

## Key Capabilities

- ✅ 20+ individual device controls for granular management
- ✅ Device profiles (Minimal, Development, Enterprise, HighPerformance)
- ✅ Comprehensive audit trails for all device operations
- ✅ Most changes take effect without VM restart
- ✅ Independent control of each device type
- ✅ Full transparency on active/inactive devices

---

## Success Metrics

✅ All 20+ devices independently controllable  
✅ Changes apply without VM recreation (except where technically necessary)  
✅ 100% audit coverage  
✅ Device profiles reduce configuration time by 90%  
✅ Security-sensitive features default disabled  
✅ Full transparency on active devices  
✅ Zero hidden functionality  

---

## Timeline

- **Week 14 (Current):** Core device control system
- **Week 15:** UI/CLI implementation
- **Week 16:** Profiles and templates
- **Week 17:** Integration with policy engine

**Target:** Ready for production v1.0 release with Phase 3

---

*Granular Device Management Controls are a key differentiator for TinyBridge v1.0.*
