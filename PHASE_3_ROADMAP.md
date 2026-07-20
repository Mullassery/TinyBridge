# Phase 3: Hardware Passthrough & ROS 2 DDS Networking

**Status:** Starting Implementation  
**Target Duration:** Weeks 13-18 (6 weeks)  
**Goal:** Enable robotics workflows with hardware and DDS support

---

## Phase 3 Goals

### Primary: Hardware Passthrough ✅ In Progress
Enable access to physical devices from inside the VM:
- USB devices (robot controllers, microcontrollers)
- Serial devices (/dev/ttyUSB*, /dev/ttyACM*)
- Cameras (USB, Webcam)
- Audio input/output
- GPU (limited: Rosetta 2 path, Phase 5 full GPU bridge)

### Secondary: ROS 2 DDS Networking ✅ In Progress
Make ROS 2 workloads work out of the box:
- Multicast passthrough (DDS requires UDP multicast)
- Network namespace awareness
- DDS domain isolation
- Multi-host ROS 2 discovery

### Tertiary: Quality Gates Enhancement ✅ In Progress
Production-grade validation for robotics:
- ROS 2 node health checks
- Topic subscription verification
- Service availability checks
- Hardware connectivity validation

---

## Implementation Plan

### Week 13-14: Hardware Device Manager

**Deliverables:**
1. USB device detection and passthrough
2. Serial device mapping
3. Camera enumeration
4. Device permission management
5. Hot-plug support (add/remove devices while running)

**New Crate:** `tinybridge-devices`

```
crates/tinybridge-devices/
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── usb.rs           # USB device detection/passthrough
│   ├── serial.rs        # Serial port mapping
│   ├── camera.rs        # Camera enumeration
│   └── device_manager.rs # Lifecycle management
├── Cargo.toml
└── tests/
```

**Key Types:**

```rust
pub enum DeviceType {
    Usb { vendor_id: u16, product_id: u16 },
    Serial { path: PathBuf, baud_rate: u32 },
    Camera { path: PathBuf, format: String },
}

pub struct Device {
    pub id: Uuid,
    pub name: String,
    pub device_type: DeviceType,
    pub host_path: PathBuf,
    pub vm_path: PathBuf,
    pub status: DeviceStatus,
}

pub enum DeviceStatus {
    Available,
    Attached,
    Detached,
    Error(String),
}

pub struct DeviceManager {
    devices: HashMap<Uuid, Device>,
    env_devices: HashMap<Uuid, Vec<Uuid>>, // env_id → [device_ids]
}
```

**CLI Integration:**

```bash
# List available devices
tinybridge devices list

# Attach device to environment
tinybridge devices attach myrobot /dev/ttyUSB0

# Show attached devices
tinybridge status myrobot --devices

# Detach device
tinybridge devices detach myrobot /dev/ttyUSB0

# Watch for hot-plug events
tinybridge devices monitor --watch
```

---

### Week 15-16: DDS Networking Support

**Deliverables:**
1. Multicast passthrough configuration
2. DDS domain awareness
3. ROS 2 environment setup
4. Network namespace isolation
5. Multi-host discovery support

**New Crate:** `tinybridge-dds`

```
crates/tinybridge-dds/
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── config.rs        # DDS configuration
│   ├── multicast.rs     # Multicast routing
│   ├── ros2.rs          # ROS 2 integration
│   └── discovery.rs     # Multi-host discovery
├── Cargo.toml
└── tests/
```

**Key Types:**

```rust
pub struct DdsConfig {
    pub enabled: bool,
    pub domain_id: u32,           // ROS_DOMAIN_ID (0-232)
    pub multicast_interface: Option<String>,
    pub allow_multicast: bool,
    pub dds_implementation: DdsImplementation,
    pub ros_domain_namespacing: bool,
}

pub enum DdsImplementation {
    CycloneDds,  // Default for ROS 2 Humble
    FastDds,     // Alternative
    OpenDds,     // Enterprise option
}

pub struct RosEnvironment {
    pub env_id: Uuid,
    pub dds_config: DdsConfig,
    pub discovery_peers: Vec<String>, // For multi-host setups
    pub topic_namespaces: Vec<String>,
}

pub struct DdsManager {
    environments: HashMap<Uuid, RosEnvironment>,
}
```

**env.yaml Schema Extension:**

```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: my-robot
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 4
  memory: 8GB
  disk: 50GB

# NEW: ROS 2 Configuration
ros2:
  enabled: true
  domain_id: 0                    # ROS_DOMAIN_ID
  dds_implementation: cyclone-dds # cyclone-dds, fast-dds, open-dds
  multicast_enabled: true         # CRITICAL for DDS
  discovery_peers: []             # For multi-host setups
  
  # Optional: namespace ROS 2 domains per environment
  domain_namespacing: true        # /myrobot/... namespace isolation

native:
  tools:
    - ros2@humble
    - gazebo
    - rviz2
```

**CLI Integration:**

```bash
# Create ROS 2 environment
tinybridge create --template robotics my-robot

# Verify DDS multicast
tinybridge ros2 check-dds my-robot

# Monitor ROS 2 topics
tinybridge ros2 topic list my-robot

# Launch ROS 2 node
tinybridge ros2 run my-robot turtlesim turtlesim_node

# Multi-host discovery
tinybridge ros2 discover-peers    # Find peers on network
```

---

### Week 17-18: Quality Gates Enhancement

**Deliverables:**
1. ROS 2 node health checks
2. Topic subscription verification
3. Service availability checks
4. Hardware connectivity validation
5. Integration with StatGuardian

**Enhancement to:** `tinybridge-daemon/src/quality_gates.rs`

```rust
pub enum QualityGate {
    // Existing (Phase 1-2)
    BootTime { target_secs: u32 },
    ResourceUsage { cpu_pct: u8, memory_pct: u8 },
    
    // NEW: ROS 2 specific (Phase 3)
    RosNodeHealth { node_name: String },
    RosTopicSubscription { topic: String, expected_hz: f32 },
    RosServiceAvailable { service: String },
    RosNetworkConnectivity { peer_count: usize },
    
    // NEW: Hardware specific (Phase 3)
    HardwareDetected { device_type: String },
    SerialConnectivity { port: String, baud_rate: u32 },
    CameraFrameRate { path: String, min_fps: u32 },
    
    // NEW: Combined (Phase 3)
    DdsMulticastWorking { domain_id: u32 },
    EndToEndRosLatency { max_latency_ms: u32 },
}

pub struct QualityGateResult {
    pub gate: QualityGate,
    pub passed: bool,
    pub value: f32,
    pub threshold: f32,
    pub timestamp: DateTime<Utc>,
    pub error: Option<String>,
}
```

**Integration with StatGuardian:**

```
Quality Gate Result → StatGuardian Contract
  
Example:
  Gate: RosTopicSubscription("/cmd_vel", expected_hz: 10.0)
  Result: topic publishing at 10.2 Hz
  Contract: "Cmd vel must publish at 10±1 Hz"
  Status: ✓ PASS
  
  Gateway: HardwareDetected("ttyUSB0")
  Result: Device not found
  Contract: "Robot controller must be connected via USB"
  Status: ✗ FAIL → Incident
```

---

## New Crates Structure

### `tinybridge-devices` (Week 13-14)
```
Dependencies:
  - tokio (async hotplug monitoring)
  - serde (device config)
  - uuid (device IDs)
  - udev (Linux device enumeration)
  - std::fs (device path mapping)

Exports:
  - DeviceManager
  - Device
  - DeviceType
  - DeviceStatus
```

### `tinybridge-dds` (Week 15-16)
```
Dependencies:
  - tokio (async networking)
  - serde (DDS config)
  - uuid (environment IDs)
  - std::net (multicast handling)
  - chrono (timestamps)

Exports:
  - DdsManager
  - DdsConfig
  - RosEnvironment
  - DdsImplementation
```

### Updated: `tinybridge-daemon`
```
Add modules:
  - hardware_bridge.rs    # Device lifecycle
  - dds_network.rs        # DDS configuration
  - ros2_integration.rs   # ROS 2 helper functions

Enhanced:
  - quality_gates.rs      # New gate types
  - EnvironmentManager    # Hardware + DDS support
```

---

## Verification Plan

### Week 13-14 (Hardware)
```bash
# USB device passthrough
tinybridge devices list
→ Output: [Canon EOS, Arduino Uno, Keybrd]

tinybridge devices attach myrobot /dev/bus/usb/001/007
→ ls /dev/ttyUSB0 (in VM) shows Arduino device

# Serial port mapping
tinybridge devices attach myrobot /dev/ttyUSB0
→ VM can read/write to /dev/ttyUSB0

# Camera
tinybridge devices attach myrobot /dev/video0
→ fswebcam -d /dev/video0 (in VM) works
```

### Week 15-16 (DDS/ROS 2)
```bash
# Multicast passthrough
tinybridge ros2 check-dds myrobot
→ Multicast routing verified ✓

# ROS 2 discovery
ros2 topic list
→ Topics from peers visible

# Multi-host discovery
tinybridge ros2 discover-peers
→ Find other ROS 2 hosts on network
```

### Week 17-18 (Quality Gates)
```bash
# Health checks
tinybridge status myrobot --gates
→ ✓ ROS master online
→ ✓ /cmd_vel publishing at 10 Hz
→ ✓ Robot controller connected via USB
→ ✗ Gripper not responding (ERROR)

# Contract validation
tinybridge validate-contract myrobot
→ StatGuardian integration working
```

---

## Risk Mitigation

### Risk: USB Hotplug Not Detected
**Mitigation:** Use inotify on /dev/bus/usb + udev notifications  
**Fallback:** Poll every 2 seconds

### Risk: DDS Multicast Broken by Firewall
**Mitigation:** Detect in IP monitor (Network Path), warn user  
**Fallback:** Document workaround for firewall admins

### Risk: ROS 2 Domain Isolation Fails
**Mitigation:** Test with multiple environments on same host  
**Verification:** Ensure topic isolation works

### Risk: Quality Gate Overhead
**Mitigation:** Lazy evaluation (check only on demand)  
**Threshold:** <1% CPU for gate checking

---

## Testing Strategy

### Hardware Testing
```
Unit Tests:
  ✓ Device enumeration
  ✓ Path mapping
  ✓ Hot-plug detection
  ✓ Permission handling

Integration Tests (requires real hardware):
  ✓ USB device passthrough
  ✓ Serial port mapping
  ✓ Camera access
```

### DDS Testing
```
Unit Tests:
  ✓ DDS config parsing
  ✓ Domain ID validation
  ✓ Multicast flag handling

Integration Tests (requires ROS 2):
  ✓ Multicast verification
  ✓ Topic discovery
  ✓ Multi-host discovery
  ✓ Namespace isolation
```

### Quality Gates Testing
```
Unit Tests:
  ✓ Gate evaluation logic
  ✓ Contract matching
  ✓ Result serialization

Integration Tests:
  ✓ ROS 2 health checks
  ✓ Hardware connectivity
  ✓ StatGuardian integration
```

---

## Deliverables Checklist

### Week 13-14: Hardware Passthrough
- [ ] Crate: tinybridge-devices created
- [ ] USB device detection
- [ ] Serial device mapping
- [ ] Camera enumeration
- [ ] Hot-plug support
- [ ] CLI commands (devices list/attach/detach)
- [ ] EnvironmentManager integration
- [ ] 15+ passing tests
- [ ] Documentation

### Week 15-16: DDS Networking
- [ ] Crate: tinybridge-dds created
- [ ] DDS configuration support
- [ ] Multicast passthrough
- [ ] ROS 2 environment setup
- [ ] env.yaml schema extension
- [ ] Multi-host discovery
- [ ] CLI commands (ros2 check-dds, topic list, run, discover)
- [ ] 20+ passing tests
- [ ] Documentation

### Week 17-18: Quality Gates Enhancement
- [ ] New quality gate types (12+)
- [ ] ROS 2 health checks
- [ ] Hardware connectivity validation
- [ ] StatGuardian integration
- [ ] End-to-end latency measurement
- [ ] 10+ passing tests
- [ ] Documentation

### Overall Phase 3
- [ ] 3 new crates
- [ ] 45+ new tests (100% passing)
- [ ] 500+ lines of documentation
- [ ] Zero breaking changes
- [ ] Full backward compatibility
- [ ] Ready for v1.0 release

---

## Success Criteria

### Robotics Workflow
```
✅ User creates: tinybridge create --template robotics my-robot
✅ Attaches USB: tinybridge devices attach my-robot /dev/ttyUSB0
✅ Enables ROS 2: env.yaml has ros2.enabled: true
✅ Runs workload: tinybridge shell my-robot && ros2 launch my_pkg robot.launch
✅ Monitors health: tinybridge status my-robot --gates shows all checks passing
✅ Discovers peers: tinybridge ros2 discover-peers finds other robots
```

### Production Validation
```
✅ No regression in Phase 1-2 features
✅ All new tests passing (100%)
✅ Builds cleanly on all platforms
✅ CI/CD pipeline green
✅ Performance: <2% overhead for quality gates
✅ Security: No new vulnerabilities
```

---

## Next Phases After Phase 3

### Phase 4: Advanced Networking (Weeks 19-24)
- GPU routing (CUDA → remote GPU)
- VPN optimizations
- Advanced firewall detection
- Cross-network ROS 2 bridges

### Phase 5: Plugin Ecosystem (Weeks 25-34)
- Vulkan-to-Metal GPU bridge
- WASM plugin architecture
- Custom device drivers
- Enterprise templates

---

**Phase 3 Start Date:** Week 13 (target completion by Week 18)  
**Status:** In Progress  
**Next Commit:** Device manager implementation
