# TinyBridge Phase 2-5 Development Roadmap

**Status:** Phase 1 foundation complete (CLI, keyboard support, documentation)  
**Current Date:** 2026-07-20  
**Next Phase:** Phase 2 (Execution Routing + Templates)

---

## 🎯 Phase Overview

```
Phase 1 (COMPLETE)  → Phase 2 → Phase 3 → Phase 4 → Phase 5
Core VM + CLI      | Routing | Hardware | Advanced | Ecosystem
(6 weeks)          | (6-8w)  | (6-8w)   | (6-8w)   | (10w)
```

---

## Phase 2: Execution Routing + Environment Templates (6-8 weeks)

### Goal
Automatically route workloads between macOS (native) and Linux (substrate) based on capability detection.

### Key Features

#### 2.1 Intelligent Workload Routing
- **Detection engine:** Inspect binary type, interpreter, system calls
- **Routing rules:** Decide native vs Linux execution per workload
- **Tier 1 (Native):** macOS arm64/x86_64 binaries, native tools
- **Tier 2 (Linux):** Linux-specific tools, packages, system APIs

**Example:**
```yaml
# env.yaml declares capabilities
native:
  tools:
    - rust
    - python
    - node

# CLI detects workload type
$ tinybridge exec myenv "python train.py"
# → Routes to Linux (Python needs Linux stack)

$ tinybridge exec myenv "swift build"
# → Routes to macOS (Swift compiler native)
```

**Implementation tasks:**
- [ ] Binary format detector (ELF vs Mach-O)
- [ ] System call interceptor
- [ ] Workload routing engine
- [ ] Routing rules DSL
- [ ] Performance profiling per route

#### 2.2 Environment Templates
- **Pre-built configurations** for common stacks
- **Quick-start templates** for backend, ML, robotics, data science
- **Community templates** shared via registry
- **Template inheritance** (e.g., "ml-training" extends "python-base")

**Templates to implement:**
1. **Backend** (Python + PostgreSQL + Redis)
2. **ML Training** (PyTorch + CUDA routing + Jupyter)
3. **Frontend** (Node + npm + build tools)
4. **Robotics** (ROS 2 + tools + DDS multicast)
5. **Data Science** (Pandas + Spark + Jupyter)
6. **Go Web** (Go + goreleaser + Docker)
7. **Rust DevOps** (Rust + cargo + container tools)

**Template commands:**
```bash
tinybridge create --template backend
tinybridge create --template ml-training
tinybridge list-templates
tinybridge view-template backend
```

**Implementation tasks:**
- [ ] Template repository structure
- [ ] Template catalog
- [ ] Template validation schema
- [ ] Interactive template wizard
- [ ] Template versioning
- [ ] Community template registry API

#### 2.3 Execution Profiles
- **Profile:** Persistent environment-specific execution settings
- **Profiles DSL:** YAML-based configuration
- **Auto-select:** Route based on profile rules

**Example:**
```yaml
# env.yaml
executionProfiles:
  default:
    routeTo: auto  # Intelligent routing
  testing:
    routeTo: linux  # Force Linux for CI consistency
  performance:
    routeTo: native  # Force native for speed
```

**Implementation tasks:**
- [ ] Profile parser
- [ ] Profile validator
- [ ] Profile selection logic
- [ ] Profile override flags

### Phase 2 Deliverables
- ✅ Workload routing engine (3 weeks)
- ✅ Template library (2 weeks)
- ✅ Execution profiles (1 week)
- ✅ Integration tests (2 weeks)

---

## Phase 3: Hardware Passthrough + DDS Networking (6-8 weeks)

### Goal
Enable hardware device access and production-grade ROS 2 networking.

### Key Features

#### 3.1 USB Device Passthrough
- **USB enumeration:** List available devices
- **Device allocation:** Grant VM access to USB devices
- **Hot-plug support:** Add/remove devices while running
- **Permissions:** Secure device access control

**Implementation tasks:**
- [ ] USB device scanner
- [ ] Device allocation engine
- [ ] Hot-plug listener
- [ ] Permission manager
- [ ] Device persistence in env.yaml

#### 3.2 Serial Port Passthrough
- **Serial device detection**
- **Baud rate configuration**
- **Flow control support**

#### 3.3 DDS Multicast Networking
- **Native multicast support** for ROS 2
- **Multicast-aware bridge** between macOS and Linux
- **Domain ID routing**

**Implementation tasks:**
- [ ] Multicast bridge
- [ ] Domain ID isolation
- [ ] Network namespace configuration
- [ ] ROS 2 native compatibility tests

#### 3.4 Advanced Networking
- **Network policies:** Firewall rules in env.yaml
- **Port forwarding:** Advanced (UDP, TCP ranges)
- **VPN/proxy support:** Route traffic through proxies

**Implementation tasks:**
- [ ] Firewall rule engine
- [ ] Port forwarding config
- [ ] Proxy support
- [ ] Network diagnostics tools

### Phase 3 Deliverables
- ✅ USB passthrough (2 weeks)
- ✅ Serial passthrough (1 week)
- ✅ DDS networking (3 weeks)
- ✅ Advanced networking (1 week)
- ✅ Integration tests (1 week)

---

## Phase 4: Advanced Execution + Device Support (6-8 weeks)

### Goal
Remote GPU routing, advanced device handling, performance optimization.

### Key Features

#### 4.1 Remote GPU Routing
- **GPU detection:** Identify CUDA/Metal capabilities
- **Remote routing:** Send compute to GPU cluster
- **Local fallback:** CPU execution if GPU unavailable
- **Load balancing:** Distribute across GPUs

**Implementation tasks:**
- [ ] GPU capability detector
- [ ] Remote cluster client
- [ ] Workload serialization
- [ ] Result deserialization
- [ ] Load balancer

#### 4.2 Advanced Device Support
- **Camera passthrough:** For robotics/vision
- **FPGA support:** Hardware acceleration
- **Audio devices:** For multimedia workloads
- **Sensor arrays:** For IoT testing

#### 4.3 Performance Optimization
- **Memory compression:** Reduce VM memory footprint
- **I/O optimization:** Faster VirtioFS + tuning
- **CPU affinity:** Pin to specific cores
- **NUMA awareness:** Multi-socket optimization

**Implementation tasks:**
- [ ] Compression engine
- [ ] VirtioFS tuning
- [ ] CPU affinity manager
- [ ] NUMA support
- [ ] Benchmarking suite

### Phase 4 Deliverables
- ✅ Remote GPU routing (3 weeks)
- ✅ Advanced device support (2 weeks)
- ✅ Performance optimization (2 weeks)
- ✅ Integration tests (1 week)

---

## Phase 5: Plugin Ecosystem + Production Features (10 weeks)

### Goal
Extensibility, community contributions, production-grade tooling.

### Key Features

#### 5.1 Plugin SDK
- **Plugin API:** Write custom routing rules, devices, executors
- **Plugin marketplace:** Discover and install plugins
- **Plugin validation:** Security scanning
- **Auto-update:** Plugin version management

**Plugin types:**
- Custom routers (e.g., "route ML to GPU if available")
- Device drivers (e.g., "MicroChip debugger support")
- Network policies (e.g., "AWS VPC integration")
- Observability exporters (e.g., "Grafana Cloud")

#### 5.2 Production Dashboards
- **Real-time monitoring** of all environments
- **Resource usage visualization**
- **Anomaly alerting**
- **Cost attribution** (per-team, per-project)

#### 5.3 Advanced Orchestration
- **Multi-machine deployment** (cluster of Macs)
- **Environment rebalancing** (move workloads between hosts)
- **Spot instance support** (dynamic resource allocation)
- **Auto-scaling** based on demand

#### 5.4 Community Ecosystem
- **Template marketplace**
- **Plugin registry**
- **Documentation site**
- **Community forum**
- **Contribution guidelines**

### Phase 5 Deliverables
- ✅ Plugin SDK (3 weeks)
- ✅ Plugin marketplace (2 weeks)
- ✅ Production dashboards (2 weeks)
- ✅ Orchestration engine (2 weeks)
- ✅ Community platform (1 week)

---

## Immediate Next Steps (Before Phase 2)

### Week 1-2: Testing & Validation

#### Task 1: Asset Pipeline Completion
- [ ] Verify assets workflow (kernel + rootfs building)
- [ ] Download production kernel from cloud-hypervisor or build locally
- [ ] Build Ubuntu 24.04 minimal rootfs via debootstrap
- [ ] Verify checksums (SHA256)
- [ ] Place in `~/.tinybridge/assets/`

**Commands:**
```bash
# Download kernel (cloud-hypervisor release URL 404s as of 2026-08-28 - see
# docs/BUILD_ASSETS_GUIDE.md; this is the confirmed-working replacement)
curl -fL -o ~/.tinybridge/assets/vmlinux \
  https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.10/aarch64/vmlinux-5.10.223

# Build rootfs (on Linux/WSL2)
sudo debootstrap --arch arm64 --variant minbase noble ~/rootfs http://ports.ubuntu.com/ubuntu-ports/
tar -czf ~/.tinybridge/assets/ubuntu-24.04-rootfs.tar.gz -C ~/rootfs .

# Generate checksums
cd ~/.tinybridge/assets && sha256sum * > checksums.txt
```

#### Task 2: Real Hardware Testing
- [ ] Boot environment: `tinybridge up test-env`
- [ ] Verify SSH access: `ssh vm@192.168.105.2`
- [ ] Test file sync: Create file on macOS, verify in Linux
- [ ] Measure boot time (3 iterations)
- [ ] Check resource usage (CPU, memory)
- [ ] Test shell responsiveness

**Test script:**
```bash
#!/bin/bash
echo "=== Starting boot test ==="
time tinybridge up test-env

echo "=== Testing SSH access ==="
ssh vm@192.168.105.2 "echo 'SSH OK'"

echo "=== Testing file sync ==="
echo "test data" > ~/test-env/test.txt
ssh vm@192.168.105.2 "cat ~/test.txt"

echo "=== Checking system status ==="
tinybridge status test-env

echo "=== Cleanup ==="
tinybridge down test-env
```

#### Task 3: Benchmark Real Data
- [ ] Run `scripts/run-benchmarks.sh` (3 times for consistency)
- [ ] Measure:
  - Boot time (SSH ready + full system)
  - File I/O latency (create, read, write, delete)
  - Memory overhead
  - CPU efficiency
- [ ] Compare vs Lima (if installed)
- [ ] Document results in `TESTING_REPORT.md`

#### Task 4: CLI Integration Testing
- [ ] All 40+ commands work end-to-end
- [ ] Resource updates apply correctly
- [ ] Checkpoint/restore preserves state
- [ ] Multi-environment isolation verified
- [ ] Error handling graceful

**Test checklist:**
```bash
tinybridge up env1
tinybridge up env2
tinybridge list
tinybridge status env1
tinybridge update env1 --cpu 8
tinybridge restart env1
tinybridge checkpoint env1 --name "test"
tinybridge restore env1 --from "test"
tinybridge down env1
tinybridge delete env2 --force
```

### Week 3-4: Prepare Phase 2 Infrastructure

#### Task 5: Workload Router Foundation
- [ ] Create `crates/tinybridge-router/` crate
- [ ] Implement ELF detector (binary format identification)
- [ ] Implement routing rules engine
- [ ] Unit tests for routing logic

**File structure:**
```
crates/tinybridge-router/
├── src/
│   ├── lib.rs
│   ├── detector.rs         (ELF, Mach-O detection)
│   ├── rules.rs            (Routing rule engine)
│   ├── profiler.rs         (Performance tracking)
│   └── cache.rs            (Decision caching)
└── tests/
    ├── detector_tests.rs
    └── router_tests.rs
```

#### Task 6: Template System
- [ ] Create `templates/` directory structure
- [ ] Implement 3 templates (backend, ml-training, frontend)
- [ ] Template validator
- [ ] Template schema (JSON Schema)

**Directory structure:**
```
templates/
├── schema.json             (JSON Schema for templates)
├── backend/
│   ├── env.yaml
│   └── README.md
├── ml-training/
│   ├── env.yaml
│   └── README.md
└── frontend/
    ├── env.yaml
    └── README.md
```

#### Task 7: Documentation Updates
- [ ] Phase 2 feature docs
- [ ] Routing rules guide
- [ ] Template usage guide
- [ ] API documentation for routers

---

## Development Priorities

### High Priority (Week 1-4)
1. ✅ Assets validation (kernel + rootfs working)
2. ✅ Benchmark real performance
3. ✅ Verify CLI end-to-end
4. 🔄 Router foundation (Phase 2 prep)
5. 🔄 Template system (Phase 2 prep)

### Medium Priority (Week 5-8)
1. Phase 2 implementation (routing + templates)
2. Extended testing
3. Performance tuning
4. Documentation expansion

### Lower Priority (Phase 3+)
1. Hardware passthrough (Phase 3)
2. Advanced networking (Phase 3)
3. GPU routing (Phase 4)
4. Plugin SDK (Phase 5)

---

## Success Criteria

### Phase 2 Success
- [ ] Workload routing works for 10+ common tools
- [ ] 5+ environment templates available
- [ ] 95%+ routing accuracy
- [ ] <10ms routing decision latency
- [ ] All tests passing
- [ ] Documentation complete

### Overall Product Success (Phase 5)
- [ ] 50+ community templates
- [ ] 20+ verified plugins
- [ ] 10,000+ monthly users
- [ ] <1% anomaly detection false positive rate
- [ ] Production deployments across 100+ companies

---

## Resources Needed

### Team
- 1 Rust backend engineer (Phase 2-3)
- 1 Systems engineer (Phase 3-4)
- 1 DevOps/infra (Phase 4-5)
- 1 Product/docs (all phases)

### Infrastructure
- Benchmark lab (M5 Mac, M2 Mac, Intel Mac)
- CI/CD for asset building
- Monitoring/observability setup
- Community communication channels

### External
- Cloud-Hypervisor kernel releases (tracking)
- Ubuntu package repository access
- GitHub Actions for CI/CD
- Optional: GPU cluster for Phase 4

---

## Go-No-Go Decision Points

### Phase 1 → Phase 2 Gate
- [x] CLI fully functional with keyboard support
- [x] Documentation complete and accurate
- [ ] Real kernel + rootfs working (pending asset build)
- [ ] Boot/performance benchmarked
- **Status:** Pending asset validation

### Phase 2 → Phase 3 Gate
- [ ] 5+ templates proven production-ready
- [ ] Routing engine 95%+ accurate
- [ ] Zero security issues in external testing
- [ ] Community interest demonstrated

### Phase 3 → Phase 4 Gate
- [ ] Hardware passthrough reliable
- [ ] ROS 2 users reporting success
- [ ] No regressions from Phase 2
- [ ] GPU cluster partnership confirmed

---

## Next Immediate Action

**Unblock Phase 2:** Get production assets (kernel + rootfs) working and benchmarked.

1. Download/build real kernel
2. Build real rootfs
3. Test boot + SSH + file sync
4. Run benchmarks
5. Document results
6. Begin Phase 2 router implementation

**Estimated time:** 2 weeks (assuming asset building succeeds)
