# TinyBridge: Open-Source macOS Linux Development Substrate

**Working name: TinyBridge** (final branding TBD)
- GitHub: `github.com/yourusername/tinybridge` (private)
- CLI: `tinybridge up`, `tinybridge exec`, `tinybridge doctor`
- App: TinyBridge.app
- Crate: `tinybridge` on crates.io

## Context

Developers on macOS need Linux capabilities (ROS 2, systemd services, Linux-only ABIs, GPU training) without the pain of traditional VMs: frozen desktops, snapshot hell, slow file I/O, and incomplete feature sets. Open-source tools like Lima and Colima fill some gaps but have hard limitations: zero GPU support, broken ROS 2 DDS networking, limited USB/serial passthrough, no parallel environments, and no Environment-as-Code paradigm.

TinyBridge ships as a native macOS `.dmg` — a real product — built open-source (Apache 2.0). It is a **dual-mode VM manager and development substrate**: run the same Linux VM headlessly (CLI-driven, minimal overhead) or with a graphical desktop (windowed, like UTM), toggling between modes on demand without restart. Built on top of this flexibility, it also routes containerized workloads to the right execution tier (native macOS / Linux substrate / remote GPU) transparently, with a declarative Environment-as-Code model at the center.

---

## Gaps in Existing Open-Source Tools

| Gap | Current State |
|-----|----------|
| Full-featured lightweight Linux substrate for macOS | Lima: minimal but bare-bones; Docker Desktop: heavyweight; Colima: incomplete |
| GPU support for training/inference | Lima, Docker, Colima: zero native support. Podman libkrun achieves ~15% native speed with manual setup |
| ROS 2 DDS networking | Requires `--network=host` workaround in containers; no tool provides multicast-aware virtual networking |
| USB/serial passthrough | Docker Desktop: zero. Lima: partial. Missing kernel modules for common devices |
| Declarative full-stack environment spec | Takes 4 separate files across 4 tools (Nix flake + Lima YAML + docker-compose.yml + devcontainer.json) |
| Parallel isolated environments | No lightweight tool supports this; Docker requires separate containers with manual port management |
| Snapshot/rollback capability | Lima, Colima: none. UTM: partial only |
| File I/O performance | CNCF documented 3.5x overhead typical; small-file workloads hit 50x; Lima/Colima at ~60-75% native |

---

## Language Stack

**100% Rust + Swift. No C++.**

- **Control daemon + CLI** — Rust + tokio (headless, always-on, AppKit-free)
- **VM host process** — Rust + Swift (per-environment, owns VZVirtualMachine, optional NSWindow)
- **macOS UI** — Swift + SwiftUI (menu bar app, environment management — separate from daemon)
- **VZ Framework access** — Minimal Swift bridge with native Virtualization.framework APIs
- **Everything else** — Rust (device discovery via IOKit, networking, routing, etc.)

The architecture separates concerns: the always-on daemon (pure Rust, no GUI overhead) supervises per-VM host processes (Rust + Swift bindings) that own the VZVirtualMachine and optionally a display window. Users who need headless VMs pay zero AppKit cost; users who want GUI get it without restarting the VM.

---

## Architecture

```
macOS Host
├── TinyBridge.app  (Swift/SwiftUI)
│   ├── Menu bar: environment status, quick actions
│   ├── System extension: network + device management
│   └── Onboarding + preferences UI
│
├── tinybridge CLI  (Rust)
│   └── Commands: up/down/exec/shell/doctor/devices/sync/env
│
├── tinybridged daemon  (Rust + tokio)
│   ├── Unix socket IPC  (/var/run/tinybridge.sock)
│   ├── Calls into VZ via C FFI bridge (Swift wrapper)
│   ├── Environment lifecycle manager
│   ├── Execution router (native → container → remote)
│   ├── Device manager (IOKit via Swift bridge)
│   └── Remote execution bridge
│
├── Linux Substrate  (Apple Virtualization Framework)
│   ├── Minimal stripped kernel (fast boot, curated module set)
│   ├── VirtioFS  (filesystem: targeting >90% native I/O)
│   ├── Rosetta 2 bridge  (AMD64 Linux binary support)
│   ├── DDS-aware virtual network  (ROS 2 multicast support)
│   ├── USB/IP server  (curated: serial, cameras, Lidar, IMU, SDR)
│   └── Podman rootless  (OCI containers inside the substrate)
│
└── Remote Bridge  (Rust)
    ├── SSH multiplexing  (transparent remote exec)
    └── Cloud APIs  (RunPod, Vast.ai, AWS, GCP for CUDA)
```

---

## Execution Router Logic

```
tinybridge exec <cmd>
        │
        ▼
Does the command require a Linux ABI or syscall?
        │
    No  ├──────────────────► Tier 1: Native macOS
        │                    (zero overhead, fastest path)
   Yes  │
        ▼
Does the workload require CUDA / NVIDIA GPU?
        │
    No  ├──────────────────► Tier 2: Linux Substrate
        │                    (Apple VZ, headless, <5s boot,
        │                     VirtioFS, Rosetta 2)
  Yes   │
        ▼
                             Tier 3: Remote Linux
                             (SSH to configured GPU host,
                              RunPod/Vast.ai/AWS/GCP)
```

The developer never specifies a tier. The router decides based on:
- Environment declaration (what the env requires)
- Command context (ROS 2 launch → Tier 2, `torch.cuda` → Tier 3)
- Runtime detection (CUDA libs present → route remote)

---

## Environment-as-Code Schema

```yaml
# .tinybridge/env.yaml
apiVersion: tinybridge/v1
kind: Environment

metadata:
  name: pyterrainmap-dev
  version: "1.2.0"
  description: Robotics terrain mapping dev environment

substrate:
  os: ubuntu              # Supported: ubuntu, debian, alpine, fedora
  version: "24.04"        # Ubuntu: 24.04, 22.04, 20.04 (default: latest LTS)
  kernel: "6.12"          # pinned kernel version (optional)
  arch: [arm64, amd64]    # amd64 via Rosetta 2

resources:
  cpu: 8
  memory: 16GB
  disk: 50GB

native:                    # Tier 1: install natively on macOS
  tools:
    - rust@1.87
    - python@3.11
    - cmake
    - uv

linux:                     # Tier 2: available in Linux substrate
  packages:
    - ros-jazzy-desktop
    - gazebo-harmonic
    - libopencv-dev
  services:
    - name: postgres
      image: postgres:16
      env:
        POSTGRES_DB: terrain_db
    - name: redis
      image: redis:7-alpine

devices:                   # hardware passthrough to Tier 2
  usb:
    - class: serial        # all /dev/ttyUSB* and /dev/ttyACM*
    - vendor: 0x2b03       # specific: Ouster Lidar
  cameras: auto            # all AVFoundation cameras

remote:                    # Tier 3: CUDA routing
  profile: runpod-a100
  triggers:
    - cuda                 # auto-route when CUDA detected
    - "torch.distributed" # distributed training

vscode:
  enabled: true
  extensions:
    - ms-python.python
    - ms-iot.vscode-ros

network:
  dds:
    enabled: true          # ROS 2 DDS multicast-aware
    domain_id: 42
  ports:                   # auto-assigned if omitted
    - 8080:8080
    - 5432:5432
```

Single file. Version-controlled. Shareable with team. Diff-able.

---

## Key Technical Decisions

### macOS App (Swift/SwiftUI)
- Native macOS app — no Electron, no web views
- Menu bar status: running environments, resource usage
- System Extension for privileged operations (network interface creation, device passthrough)
- Handles code signing, notarization, system permissions (microphone, camera, USB)
- Launches and supervises `tinybridged`

### CLI (Rust)
- Single static binary, installed to `/usr/local/bin/tinybridge`
- Communicates with daemon via Unix socket (protobuf or JSON-RPC)
- Fast: sub-100ms for status commands
- Key crates: `clap`, `tokio`, `serde`, `tonic` (gRPC), `indicatif` (progress), `ratatui` (TUI)

### Daemon (Rust + tokio)
- Manages VM lifecycle (start, stop, snapshot, clone)
- Owns the execution router state machine
- Handles device hotplug events (macOS IOKit → USB/IP passthrough)
- Exposes gRPC API for CLI + app

### Linux Substrate (Apple Virtualization Framework)
- **NOT** a full desktop VM — headless, no GPU display, no GUI
- Custom stripped Linux kernel: fast boot (<5s), only modules needed for development
- Curated kernel module set for robotics/embedded hardware (see device section)
- VirtioFS with write-back caching — targeting >90% native I/O (better than Lima's ~60-75%)
- Rosetta 2 binary translator registered in the VM — run AMD64 Linux binaries natively
- Boot protocol: compressed kernel + initrd directly via VZ framework (no GRUB)

### Networking (DDS-Aware)
- Virtual Layer 2 network between macOS and Linux substrate
- **Multicast pass-through** — ROS 2 DDS discovery works out of the box (no `--network=host` required)
- Automatic `ROS_DOMAIN_ID` namespace isolation per environment (no cross-environment DDS bleed)
- `.tinybridge.local` DNS: `robotics-env.tinybridge.local` resolves from macOS and Linux both
- Optional bridged mode: expose Linux substrate on LAN (more flexible than localhost-only SSH)

### USB / Device Passthrough
- IOKit event monitor in daemon detects device plug/unpack
- USB/IP protocol tunnels devices to Linux substrate
- **Curated kernel module set** baked into the substrate kernel (what lightweight tools miss):
  - `usb_serial`, `cp210x`, `ch341`, `ftdi_sio` — serial adapters (Arduino, ESP32, sensors)
  - `cdc_acm` — USB CDC modems (Quectel LTE, GPS)
  - `uvcvideo` — UVC cameras
  - `hid_xpad` — Xbox controllers / joysticks
  - `rtl8xxxu` — USB WiFi (pentesting SDR)
  - `sdr` module family — RTL-SDR, HackRF
  - `xpad`, `joydev` — robotics gamepads
- Camera: AVFoundation frame capture → v4l2 virtual device in Linux substrate

### Snapshot / Environment Versioning
- CoW disk images (APFS sparse + reflink copy)
- `tinybridge env snapshot` — CoW snapshot with copy-on-write optimization
- `tinybridge env rollback <snapshot>` — restore from snapshot
- `tinybridge env clone` — fork environment for AI agent workflows (fixes Arcjet's 670-line workaround)
- Environments are git-diffable (YAML definition) + snapshot-restorable (VM state)

### GPU Strategy (Phased)
- **Phase 1-4**: No GPU in Linux substrate (honest about the limitation; CUDA routes to remote)
- **Phase 5**: Vulkan-to-Metal bridge via VirtioGPU Venus + MoltenVK (same approach as Podman libkrun but with maintained DX and documented performance)
- **Phase 5**: Metal Compute API forwarding for MLX/PyTorch-MPS workflows

### Remote Execution (CUDA Tier)
- SSH multiplexing with connection pooling (`~/.tinybridge/remotes/<name>`)
- Auto-detection: if command imports `torch.cuda`, `cupy`, or uses CUDA libs → route to configured remote
- Cloud provider integrations: RunPod, Vast.ai, AWS, GCP (spawn + terminate GPU instances)
- Same `env.yaml` deploys locally and remotely — no separate config

---

## Distribution

```
build pipeline (GitHub Actions)
    ├── Cargo build (Rust CLI + daemon) — universal binary (arm64 + x86_64)
    ├── Xcode build (Swift app) — signed with Developer ID
    ├── Package into .app bundle
    ├── Notarize with Apple
    ├── Create .dmg (create-dmg)
    └── Publish to GitHub Releases
```

- `.dmg` for direct download
- `brew install --cask tinybridge` (Homebrew cask)
- Developer ID signed + notarized (required for macOS Gatekeeper)
- Auto-update via Sparkle framework

---

## Workspace Structure

```
tinybridge/
├── Cargo.toml                    # Rust workspace root
├── rust-toolchain.toml
├── Package.swift                 # Swift package (app + FFI bridge)
│
├── crates/                       # All Rust, zero C++
│   ├── tinybridge-core/            # Shared types, schema, config (no_std compatible)
│   ├── tinybridge-daemon/          # tinybridged: lifecycle, router, IPC server
│   ├── tinybridge-cli/             # tinybridge CLI binary
│   ├── tinybridge-vz-sys/          # Rust FFI bindings to VZ C bridge (generated from C header)
│   ├── tinybridge-vz/              # Rust wrapper around -sys bindings (safe API)
│   ├── tinybridge-router/          # Execution tier decision engine
│   ├── tinybridge-devices/         # Device discovery (calls Swift bridge for IOKit)
│   ├── tinybridge-remote/          # SSH + cloud provider APIs
│   ├── tinybridge-doctor/          # Diagnostic engine
│   ├── tinybridge-templates/       # Environment template library
│   └── tinybridge-env/             # env.yaml parser, validator, versioning
│
├── swift/                        # All Swift, zero C++
│   ├── TinyBridgeApp/              # SwiftUI macOS app
│   │   ├── AppDelegate.swift
│   │   ├── MenuBarView.swift
│   │   ├── EnvironmentListView.swift
│   │   └── PreferencesView.swift
│   ├── TinyBridgeVZBridge/         # ← THE C FFI BRIDGE (~500 lines)
│   │   ├── VZBridge.swift        # Swift wrapper around Virtualization.framework
│   │   ├── include/
│   │   │   └── tinybridge_vz.h     # C header exposed to Rust (FFI boundary)
│   │   ├── TinyBridgeVZ.h          # Internal Objective-C header
│   │   └── TinyBridgeVZ.swift      # Swift implementation
│   ├── TinyBridgeIOKitBridge/      # Device discovery via IOKit (similar C bridge, ~300 lines)
│   │   ├── IOKitBridge.swift
│   │   ├── include/
│   │   │   └── tinybridge_iokit.h
│   │   └── TinyBridgeIOKit.swift
│   └── TinyBridgeExtension/        # System Extension (network/device passthrough)
│       └── TinyBridgeNetworkExtension.swift
│
├── c-bridge/                     # Generated C FFI bindings (DO NOT EDIT)
│   └── (bindgen output goes here)
│
├── kernel/
│   ├── config/                   # Stripped kernel .config (arm64 + x86)
│   └── modules/                  # Module list: serial, USB, camera, ROS
│
├── templates/
│   ├── robotics.yaml             # ROS 2 + Gazebo + OpenCV + Foxglove
│   ├── ai-local.yaml             # PyTorch + JAX + Ollama + MLX-native
│   ├── ai-cloud.yaml             # PyTorch + CUDA → remote GPU
│   ├── data.yaml                 # Postgres + Kafka + Spark + DuckDB
│   └── cloudnative.yaml          # Kubernetes + Helm + Terraform
│
├── packaging/
│   ├── TinyBridge.dmg.spec
│   ├── Casks/tinybridge.rb         # Homebrew cask
│   └── entitlements.plist
│
└── .github/
    └── workflows/
        ├── ci.yml                # Test + lint on PR (cargo test + swift test)
        └── release.yml           # Build + sign + notarize + publish .dmg
```

**Key points:**
- `tinybridge-vz-sys/` — Auto-generated via `bindgen` from C header (low-level FFI)
- `tinybridge-vz/` — Hand-written Rust wrapper providing safe API
- `TinyBridgeVZBridge/` — Swift wrapper around VZ Framework; exports minimal C header
- The C header is the **only** C code; zero C++ anywhere

This pattern mirrors how Lima and Podman Machine work on macOS.


---

## Key Rust Dependencies

```toml
[workspace.dependencies]
tokio           = { version = "1", features = ["full"] }
clap            = { version = "4", features = ["derive"] }
serde           = { version = "1", features = ["derive"] }
serde_yaml      = "0.9"
tonic           = "0.12"         # gRPC for CLI↔daemon IPC
prost           = "0.13"         # protobuf
tracing         = "0.1"
anyhow          = "1"
thiserror       = "2"
indicatif       = "0.17"         # progress bars
ratatui         = "0.29"         # optional TUI
ssh2            = "0.9"          # remote execution
reqwest         = "0.12"         # cloud API clients
semver          = "1"            # env version pinning
```

---

## `platform doctor` Architecture

Diagnostic engine runs in the daemon, exposed via CLI. Each check is a trait:

```rust
pub trait DiagnosticCheck: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, ctx: &EnvContext) -> DiagResult;
}
```

Check categories:
- **Substrate**: VM running, VirtioFS mounted, kernel version, Rosetta status
- **Network**: DDS multicast reachable, DNS resolving, port conflicts
- **Devices**: USB devices enumerated, kernel modules loaded, camera accessible
- **Native**: Tool versions, PATH conflicts, Homebrew/Nix state
- **Containers**: Podman socket, image pulls, volume mounts
- **Remote**: SSH connectivity, GPU availability, cloud credentials
- **ROS 2**: RMW implementation, domain ID, topic discovery
- **Security**: Secrets not in env.yaml, CVE scan on images

Output format:
```
tinybridge doctor

✓  Linux substrate running  (kernel 6.12.4, arm64 + amd64 via Rosetta)
✓  VirtioFS mounted  (/Users/georgi → /home/georgi, 94% native I/O)
✓  DDS multicast active  (domain 42, 3 nodes discovered)
⚠  Quectel modem /dev/ttyUSB1 not passed through  → run: tinybridge devices attach ttyUSB1
✗  Remote GPU profile not configured  → run: tinybridge remote add runpod --key $RUNPOD_KEY
⚠  Python 3.11 version mismatch: native=3.11.9, substrate=3.11.4  → run: tinybridge env sync python
```

---

## Environment Template System

Templates are YAML with inheritance:

```yaml
# templates/robotics.yaml
apiVersion: tinybridge/v1
kind: Template
metadata:
  name: robotics
  tags: [ros2, gazebo, lidar, camera]

extends: base-ubuntu-24.04

linux:
  packages:
    - ros-jazzy-desktop
    - gazebo-harmonic
    - ros-jazzy-ros2-control
    - python3-colcon-common-extensions

devices:
  usb:
    - class: serial       # all serial devices auto-attached
  cameras: auto

network:
  dds:
    enabled: true
    domain_id: auto       # auto-assigned per environment
```

Usage:
```bash
tinybridge create --template robotics my-robot-project
tinybridge create --template ai-cloud training-run
tinybridge create --template data analytics-stack
```

Custom templates stored in `~/.tinybridge/templates/` or `.tinybridge/templates/` in project.

---

## Phase Roadmap

### Phase 1 — Foundation (Weeks 1-6)
**Goal: Shippable alpha with core VM lifecycle**

- [ ] Rust workspace + Swift package setup
- [ ] `tinybridged` daemon with Unix socket IPC
- [ ] Apple VZ Framework integration (Swift VZ wrapper → Rust FFI)
- [ ] Minimal Linux kernel build pipeline (arm64 + x86 config)
- [ ] VirtioFS filesystem sharing
- [ ] `tinybridge up`, `down`, `status`, `shell` commands
- [ ] `env.yaml` schema parser + validator (`tinybridge-core`)
- [ ] SwiftUI menu bar app skeleton (shows running environments)
- [ ] `.dmg` build + code signing + notarization pipeline (GitHub Actions)
- [ ] Homebrew cask formula

**Deliverable**: Install `.dmg`, run `tinybridge up`, get a shell in a Linux environment with optimized boot.

---

### Phase 2 — Execution Layer (Weeks 7-12)
**Goal: Smart routing + templates + VS Code integration**

- [ ] Execution router (Tier 1 / 2 / 3 decision logic)
- [ ] `tinybridge exec <cmd>` — transparent command routing
- [ ] Podman rootless inside Linux substrate
- [ ] Container management within environments (services: postgres, redis, etc.)
- [ ] Rosetta 2 AMD64 support in substrate
- [ ] Environment templates: `robotics`, `ai-local`, `data`, `cloudnative`
- [ ] VS Code Remote-SSH auto-config (`.tinybridge/ssh_config` → `~/.ssh/config`)
- [ ] `tinybridge doctor` v1 — basic health checks
- [ ] Environment variables + secrets management
- [ ] Nix integration (detect + use nix-installed tools in Tier 1)

**Deliverable**: `tinybridge create --template robotics` → fully working ROS 2 environment in VS Code.

---

### Phase 3 — Hardware & DX (Weeks 13-18)
**Goal: Robotics-grade device support + parallel environments + DDS networking**

- [ ] `tinybridge devices` — IOKit device discovery
- [ ] USB/serial passthrough (curated kernel modules + USB/IP)
- [ ] Camera/video device routing (AVFoundation → v4l2 virtual device)
- [ ] DDS-aware virtual network (ROS 2 multicast works out of the box)
- [ ] `ROS_DOMAIN_ID` auto-isolation per environment
- [ ] Parallel environments with auto port assignment + workspace isolation
- [ ] CoW snapshot / rollback (`tinybridge env snapshot`, `rollback`, `clone`)
- [ ] `tinybridge doctor` v2 — ROS 2, device, and container diagnostics
- [ ] LAN access to Linux substrate (bridged networking mode)
- [ ] Environment git-versioning (`tinybridge env diff`, `log`, `rollback`)

**Deliverable**: `tinybridge devices attach lidar` → LiDAR sensor available in ROS 2 with multicast discovery. `tinybridge env clone agent-1` for parallel AI agent workflows.

---

### Phase 4 — Remote & Cloud (Weeks 19-24)
**Goal: CUDA routing + cloud sync**

- [ ] CUDA detection (import scanning + lib detection)
- [ ] Remote execution bridge (SSH multiplexed, transparent)
- [ ] `tinybridge remote add runpod --key $KEY`
- [ ] Auto-routing: CUDA workload → remote GPU host
- [ ] Cloud provider integrations: RunPod, Vast.ai, AWS (g4dn/p3), GCP (A100)
- [ ] `tinybridge sync` — push environment to cloud, execute, pull results
- [ ] Remote environment lifecycle (spin up cloud instance → execute → terminate)
- [ ] CI/CD parity mode (`tinybridge ci-run github-actions`)
- [ ] `ai-cloud` template with seamless local→remote handoff

**Deliverable**: `tinybridge exec python train.py` → automatically runs on RunPod A100 when CUDA detected, syncs results back.

---

### Phase 5 — GPU + Ecosystem (Weeks 25-34)
**Goal: Native GPU bridge + plugin architecture + community**

- [ ] Vulkan-to-Metal bridge in Linux substrate (VirtioGPU Venus + MoltenVK)
- [ ] Metal Compute API forwarding (MLX/PyTorch-MPS accessible from Linux)
- [ ] WASM plugin architecture (`tinybridge plugin install ros-foxglove`)
- [ ] Security auditing + compliance exports
- [ ] Database native support (schema inspection, lineage, diagnostics)
- [ ] Kubernetes local cluster (lightweight k3s in substrate)
- [ ] Community template marketplace

**Deliverable**: `torch.device('mps')` works inside the Linux substrate. Plugin ecosystem live.

---

## Feature Comparison vs. Open-Source Alternatives

| Capability | Lima | Docker Desktop | Podman | TinyBridge |
|-----------|------|----------|--------|---------|
| Open source | ✅ | ⚠️ (partial) | ✅ | ✅ Apache 2.0 |
| Free | ✅ | ⚠️ (enterprise paid) | ✅ | ✅ |
| GPU/Metal support | ❌ | ❌ | ❌ | ✅ Phase 4-5 |
| ROS 2 DDS multicast | ❌ | ❌ | ❌ | ✅ Phase 3 |
| USB serial passthrough | ⚠️ Partial | ❌ | ⚠️ Partial | ✅ Curated set |
| Environment-as-Code | ⚠️ Workaround | ⚠️ Separate tools | ⚠️ Separate tools | ✅ Core paradigm |
| Parallel environments | ❌ | ✅ (containers) | ✅ (containers) | ✅ Phase 2 |
| Snapshot/clone | ❌ | ⚠️ (docker commit) | ⚠️ (podman commit) | ✅ Phase 3 |
| LAN SSH access | ❌ localhost | ❌ localhost | ❌ localhost | ✅ Phase 3 |
| Native file I/O | 60-75% | 40-60% | 40-60% | >90% Phase 2 |
| GUI (macOS) | ❌ CLI only | ✅ | ❌ CLI only | ✅ Menu bar |

---

## Integration with Existing Portfolio

- **PyTerrainMap** — ships as a `tinybridge create --template pyterrainmap` environment; sensor drivers pre-configured
- **PyStreamMCP** — optional: `tinybridge exec` routing logic mirrors PyStreamMCP's query routing architecture
- **StatGuardian** — optional Phase 5: environment contract validation via StatGuardian (reproducibility contracts)
- **PyRoboFrames** — robotics template pre-installs PyRoboFrames sensor pipeline

---

## Open Source Licensing

**Apache License 2.0** — permissive, allows commercial use, requires attribution.

- No proprietary lock-in, no licensing fees, no compliance friction
- Community-driven roadmap and contributions welcome
- Optional "open core" model (Phase 5+): team management features available as optional paid tier for enterprises (centralized config, audit logs, remote access broker)

---

## Verification Plan

### Phase 1 verification
1. `brew install --cask tinybridge` on clean macOS 14 machine
2. `tinybridge up` → Linux substrate boots, `tinybridge status` shows running
3. `tinybridge shell` → bash prompt inside Linux
4. Edit a file on macOS, verify change visible in Linux via VirtioFS (within 100ms)
5. `tinybridge down` → substrate stops cleanly
6. Measure boot time and validate against production requirements

### Phase 2 verification
1. `tinybridge create --template robotics robot-ws` → ROS 2 jazzy available
2. `tinybridge exec ros2 run demo_nodes_cpp talker` → talker running
3. Open VS Code → Remote-SSH to `robot-ws.tinybridge.local` → extensions working
4. `tinybridge exec python script.py` on pure Python script → executes natively on macOS (Tier 1)

### Phase 3 verification
1. Plug in Arduino/serial device → `tinybridge devices` shows it
2. `tinybridge devices attach ttyUSB0` → `/dev/ttyUSB0` accessible in Linux substrate
3. Run ROS 2 talker/listener across two parallel environments → DDS discovery works
4. `tinybridge env snapshot v1` → modify env → `tinybridge env rollback v1` → restored

### Phase 4 verification
1. `tinybridge remote add runpod --key $KEY`
2. `tinybridge exec python -c "import torch; print(torch.cuda.is_available())"` → routed to RunPod, prints `True`
3. `tinybridge sync training-job` → pushes, executes, returns results
