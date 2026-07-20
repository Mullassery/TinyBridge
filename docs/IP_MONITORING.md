# TinyBridge IP Monitoring System

## Overview

TinyBridge implements a production-grade IP monitoring system that enables:

- **Automatic VM Discovery** — Show VMs with their IPs and connection options
- **Dynamic Port Forwarding** — Auto-update forwarding rules when VM IPs change
- **Security Monitoring** — Detect suspicious IP changes and network anomalies
- **Enterprise Integration** — Detect VPN routing and firewall rules
- **Network Diagnostics** — Track bandwidth, latency, and connectivity

---

## Architecture

### 1. Core Data Model

```rust
pub struct IpAddressRecord {
    // Identity
    pub env_id: Uuid,
    pub env_name: String,
    pub ssh_alias: String,

    // Network Configuration
    pub network: NetworkConfig {
        pub ipv4: Option<String>,
        pub ipv6: Option<String>,
        pub dns_servers: Vec<String>,
        pub gateway: Option<String>,
        pub subnet: Option<String>,
    },

    // Connectivity Status
    pub status: ConnectivityStatus, // Online / Offline / Unknown
    pub network_path: NetworkPath,  // Vpn / Blocked / Direct / Unknown

    // Monitoring
    pub last_ip_change: Option<DateTime<Utc>>,
    pub last_check: Option<DateTime<Utc>>,
    pub ssh_port: u16,
    pub security_events: Vec<SecurityEvent>,
}
```

### 2. Monitor System

```rust
pub struct IpMonitor {
    records: HashMap<Uuid, IpAddressRecord>,
    ip_to_env: HashMap<String, Uuid>, // Reverse lookup for O(1) IP→Env resolution
}
```

**Features:**
- O(1) lookup by environment ID or IP address
- Automatic reverse mapping maintenance
- Security event tracking
- Network path detection

---

## Use Cases

### 1. Detect VM Network Connectivity

**Problem:** Dashboard shows "Ubuntu VM" without knowing if it's online.

**Solution:** IP Monitor tracks connectivity status continuously.

```rust
monitor.set_connectivity_status(env_id, ConnectivityStatus::Online);

// Dashboard can now show:
// ubuntu-dev   → 192.168.64.15   ✓ Online
// ubuntu-test  → 192.168.64.20   ✗ Offline (5m 23s)
```

**Implementation:**
```rust
pub fn is_online(&self) -> bool {
    self.status == ConnectivityStatus::Online
}

pub fn list_online(&self) -> Vec<IpAddressRecord> {
    // Only return reachable VMs
}

pub fn count_online(&self) -> usize {
    // Quick health check count
}
```

---

### 2. Auto-Configure Port Forwarding

**Problem:** VM gets new DHCP address → SSH forwarding breaks.

**Solution:** IP Monitor detects change, triggers SSH config update.

```
Yesterday:
  env_id: "abc-123"
  ipv4: "192.168.64.12"
  ssh config: ssh -p 22 ubuntu@192.168.64.12

IP changes to 192.168.64.27:
  monitor.update_ipv4(env_id, Some("192.168.64.27")) → returns true (changed)
  → Triggers ssh_config_manager.update_hostname("myenv", "192.168.64.27")
  → SSH forwarding automatically updated

Today:
  ipv4: "192.168.64.27"
  ssh config: ssh -p 22 ubuntu@192.168.64.27 (auto-updated)
```

**Implementation:**
```rust
pub fn update_ipv4(&mut self, env_id: Uuid, new_ip: Option<String>) -> bool {
    // Update reverse lookup
    if let Some(old_ip) = &record.network.ipv4 {
        self.ip_to_env.remove(old_ip);
    }
    if let Some(ref new_ip) = new_ip {
        self.ip_to_env.insert(new_ip.clone(), env_id);
    }
    record.update_ipv4(new_ip) // Returns true if changed
}
```

---

### 3. Security and Compliance Monitoring

**Problem:** Enterprise needs to detect compromised VMs, unauthorized software, data exfiltration.

**Solution:** IP Monitor records security events and detects anomalies.

**Trackable Anomalies:**

```rust
pub enum SecurityEvent {
    // IP address changed unexpectedly
    UnexpectedIpChange { from: String, to: String },

    // Multiple rapid IP changes (possibly compromised)
    RapidIpChanges { count: u32 },

    // VM came online from unexpected subnet
    UnexpectedSubnet { ip: String },

    // Connection attempt from suspicious destination
    SuspiciousConnection { ip: String },
}
```

**Example Workflow:**
```rust
// Normal: IP changes once per month (expected)
monitor.record_security_event(env_id, 
    SecurityEvent::UnexpectedIpChange { 
        from: "192.168.64.10".into(), 
        to: "192.168.64.11".into() 
    });

// Alert: 5 IP changes in 2 minutes (possible compromise)
monitor.record_security_event(env_id, 
    SecurityEvent::RapidIpChanges { count: 5 });

// Check for alerts
if monitor.has_security_alerts(env_id) {
    let events = monitor.get_security_events(env_id);
    // Send to security monitoring system
    // Trigger incident response workflow
}
```

**Integration with OKF (OpenTelemetry Key Facts):**
- Security events exported to tamper-evident audit log
- Anomaly scores tracked over time
- Forensics replay available for incident investigation

---

### 4. VM Discovery

**Problem:** Users must manually SSH to 192.168.64.15 or run commands to find VM IPs.

**Solution:** VM Monitor provides discovery metadata for automatic UI population.

```rust
pub struct VmDiscoveryMetadata {
    pub name: String,
    pub ip_address: Option<String>,
    pub hostname: String,                    // myvm.local (from DNS monitor)
    pub ssh_command: Option<String>,         // ssh ubuntu@192.168.64.15
    pub http_url: Option<String>,            // http://192.168.64.15:8080
    pub status: ConnectivityStatus,          // Online / Offline
    pub network_path: NetworkPath,           // Vpn / Direct / Blocked
}
```

**UI Display Example:**

```
┌─ TinyBridge Environments ─────────────────┐
│                                           │
│ ubuntu-dev                        ✓ Online
│ IP: 192.168.64.15                         
│ Hostname: ubuntu-dev.local                
│ SSH: ssh ubuntu@192.168.64.15    
│ HTTP: http://192.168.64.15:8080          
│                                           │
│ ubuntu-test                       ✗ Offline
│ IP: 192.168.64.20 (last seen 5m ago)    
│ Last attempt failed                       
│                                           │
│ ml-training                       ✓ Online
│ IP: 192.168.64.42                        
│ Via: VPN (secure routing)                
│                                           │
└─────────────────────────────────────────┘
```

**Implementation:**
```rust
pub fn vm_discovery(&self) -> Vec<VmDiscoveryMetadata> {
    self.records
        .values()
        .map(|r| r.vm_discovery_metadata())
        .collect()
}

// In CLI/UI:
let vms = monitor.vm_discovery();
for vm in vms {
    println!("{:<15} {} {}", vm.name, vm.ip_address.unwrap_or_default(), 
        if vm.status == ConnectivityStatus::Online { "✓ Online" } else { "✗ Offline" });
}
```

---

### 5. VPN and Firewall Awareness

**Problem:** Enterprise users on VPN need to know if VM traffic routes through VPN and if firewall is blocking.

**Solution:** IP Monitor detects network path and reports status.

```rust
pub enum NetworkPath {
    Vpn,      // Traffic routed through VPN (secure)
    Blocked,  // Firewall blocking traffic
    Direct,   // Direct routing (no VPN)
    Unknown,  // Not yet detected
}
```

**Detection Logic:**
```rust
// When a VM comes online:
// 1. Check if IP is on VPN subnet
// 2. Check if traffic can reach host (ping/TCP)
// 3. Set network_path accordingly

monitor.detect_network_path(env_id, NetworkPath::Vpn);

// User sees:
// "Traffic is routed through corporate VPN - all enterprise security policies apply"
```

**Example Scenario:**
```
Scenario 1: On Corporate VPN
  VM IP: 10.0.1.15 (VPN subnet)
  network_path: Vpn
  Status: "✓ Online (via corporate VPN, all security policies active)"

Scenario 2: Behind Firewall
  VM IP: 192.168.1.100 (external)
  Can't reach from host
  network_path: Blocked
  Status: "✗ Firewall blocking - contact IT to open port 22"

Scenario 3: Direct Connection
  VM IP: 192.168.64.50 (local)
  Reachable immediately
  network_path: Direct
  Status: "✓ Online (direct connection)"
```

---

### 6. Usage Analytics and Troubleshooting

**Problem:** "SSH connection is slow" — but why? Need to diagnose.

**Solution:** IP Monitor tracks metrics for troubleshooting.

```rust
pub struct IpAddressRecord {
    pub last_check: Option<DateTime<Utc>>,      // Last connectivity check
    pub ssh_port: u16,                           // SSH port used
    pub network_path: NetworkPath,               // Vpn/Direct/Blocked
    // Future: latency_ms, packet_loss_pct, bandwidth_mbps
}
```

**Troubleshooting Workflow:**
```bash
# User reports slow SSH
$ tinybridge diagnostics myvm

Output:
  Name: myvm
  IP: 192.168.64.42
  Status: Online (since 1h 23m)
  SSH Port: 22
  Network Path: VPN
  Last Check: 23ms ago
  
  Diagnosis:
  ✓ VM is online and reachable
  ✓ SSH port is open
  ⚠️  Traffic routed through VPN (may add latency)
  
  Recommendation: 
  Try SSH directly without VPN if possible, 
  or contact IT to prioritize VPN traffic
```

---

## Integration Points

### With tinybridge-ssh
```rust
// When IP changes:
monitor.update_ipv4(env_id, new_ip)?
    → ssh_config_manager.update_hostname(ssh_alias, new_ip)?
    → SSH forwarding rules updated automatically
```

### With tinybridge-dns
```rust
// When VM comes online:
monitor.set_connectivity_status(env_id, ConnectivityStatus::Online)
    → dns_registry.activate(env_id)?
    → mDNS announces {env_name}.local
    → Users can ssh {env_name}.local automatically
```

### With tinybridge-tunnel
```rust
// When port forward is created:
forward_config.target_ip = monitor.get_by_env_id(env_id).network.ipv4?
    → Tunnel automatically routes to correct IP
    → If VM IP changes, tunnel updates automatically
```

### With OTel (Observability)
```rust
// Every IP change is logged as OTel event:
OTel Event: "environment.ip_changed"
  env_id: uuid
  env_name: string
  old_ip: string
  new_ip: string
  timestamp: now
  security_events: [...]

// Dashboard shows:
// - Boot time trend (detect regressions)
// - Connectivity uptime % per environment
// - IP change frequency (baseline vs anomaly)
// - Network path distribution (VPN vs Direct vs Blocked)
```

---

## API Reference

### Core Operations

```rust
// Register for monitoring
monitor.register(env_id, "myenv".to_string(), "myenv-alias".to_string());

// Update network configuration
monitor.update_ipv4(env_id, Some("192.168.64.15".to_string()));
monitor.update_ipv6(env_id, Some("fe80::1".to_string()));
monitor.update_network_config(env_id, NetworkConfig { ... });

// Track connectivity
monitor.set_connectivity_status(env_id, ConnectivityStatus::Online);

// Detect network path (VPN/Firewall/Direct)
monitor.detect_network_path(env_id, NetworkPath::Vpn);

// Security monitoring
monitor.record_security_event(env_id, SecurityEvent::UnexpectedIpChange { ... });

// Queries
monitor.get_by_env_id(env_id) → Option<IpAddressRecord>
monitor.get_env_by_ip("192.168.64.15") → Option<Uuid>
monitor.list_all() → Vec<IpAddressRecord>
monitor.list_online() → Vec<IpAddressRecord>
monitor.count_online() → usize

// Discovery
monitor.vm_discovery() → Vec<VmDiscoveryMetadata>

// Security
monitor.has_security_alerts(env_id) → bool
monitor.get_security_events(env_id) → Vec<SecurityEvent>
```

---

## Testing

All functionality is covered by unit tests:

```bash
cargo test -p tinybridge-daemon -- ip_monitor
```

Tests cover:
- ✅ IPv4 and IPv6 tracking
- ✅ Connectivity status updates
- ✅ Security event recording
- ✅ Network path detection
- ✅ VM discovery metadata generation
- ✅ Reverse IP→Env lookup
- ✅ Online/offline filtering

---

## Future Enhancements

1. **Bandwidth Metrics** — Track historical usage per environment
2. **Latency Tracking** — Detect slow connections (VPN vs Direct comparison)
3. **Packet Loss Detection** — Diagnose unreliable networks
4. **DNS Resolution Monitoring** — Track {env_name}.local resolution success
5. **Port Availability** — Pre-check which ports are open before port forwarding
6. **Firewall Rule Suggestions** — Automatically suggest rules to admins
7. **VPN Profile Detection** — Identify which VPN profile is active
8. **Multi-NIC Support** — Track multiple network interfaces per VM

---

## Platform Features

| Feature | TinyBridge |
|---------|---|
| **IP Monitoring** | ✅ Full production-grade system |
| **Auto-Update SSH** | ✅ Automatic on IP change |
| **Security Events** | ✅ Comprehensive anomaly detection |
| **VM Discovery** | ✅ Full metadata with connection details |
| **VPN Awareness** | ✅ Network path detection |
| **Diagnostics** | ✅ Integrated troubleshooting tools |

---

## Product Positioning

TinyBridge's IP monitoring provides enterprise-grade connectivity automation:

1. **Zero-Config Connectivity** — Users don't manually SSH to IPs; system discovers and forwards automatically
2. **Enterprise Ready** — Security alerts and VPN awareness built-in
3. **Production Grade** — OpenTelemetry integration for compliance and forensics
4. **Future-Proof** — Extensible for bandwidth, latency, and firewall features

**Result:** Developers get "it just works" connectivity without manual configuration or vendor lock-in.

---

*IP Monitoring is available in TinyBridge Phase 2+.*
