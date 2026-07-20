# TinyBridge: Product Vision

**Working name: TinyBridge** | Final branding TBD | GitHub: private repo

## Preamble

TinyBridge is an open-source macOS application that reimagines the Linux development experience. It bridges the gap between native macOS performance and the capability of full Linux environments, without the pain of traditional virtual machines.

The product is **shipping**: `.dmg` installers, Homebrew cask, Developer ID signed + notarized, production-grade from day one. Not a side project. Not a framework. A **real developer product**.

---

## The Problem

Developers on macOS face a painful trilemma:

### Option 1: Native macOS
- ✅ Fast, responsive, everything just works
- ❌ Cannot run ROS 2, systemd services, Linux-only Docker images, Linux-only Lidar drivers
- ❌ Python environments fragment across native + container contexts

### Option 2: Docker Desktop / OrbStack / VMware
- ✅ Full Linux capability
- ❌ VM overhead: memory allocated upfront, frozen desktops under load, 15-50s first-container latency
- ❌ Snapshots break over time; no environment versioning
- ❌ 3-50x slower file I/O for small-file workloads (npm install, pip install, terraform)
- ❌ Closed-source lock-in (OrbStack) or expensive licensing (Docker Desktop $21/user/month)
- ❌ Zero GPU support — AI/ML workflows stuck on CPU or forced to cloud
- ❌ ROS 2 DDS multicast breaks; requires `--network=host` workaround

### Option 3: Cloud Development
- ✅ Full Linux, unlimited GPU
- ❌ Latency kills iteration velocity
- ❌ Expensive for interactive development
- ❌ Vendor lock-in (Codespaces, Coder, Gitpod)
- ❌ Network failures kill productivity

**The Product:** TinyBridge solves this by making Linux feel **native** on macOS. Developers never "enter a VM." They work in a single cohesive environment where macOS and Linux workloads coexist transparently, with intelligent routing deciding whether to run locally (fast) or remotely (GPU).

---

## The Vision

TinyBridge is the **primary development substrate** for engineers building:
- **Robotics systems** (ROS 2, sensor hardware, Gazebo simulation)
- **AI/ML pipelines** (local inference, cloud training)
- **Data platforms** (Postgres, Kafka, Spark, data lineage)
- **Cloud-native infrastructure** (Kubernetes, microservices, DevOps)

The product is:
- **Open source** (Apache 2.0) — no licensing, no vendor lock-in, community-driven
- **Native macOS** (Swift/SwiftUI) — feels like a first-class macOS app, not a wrapper
- **Declarative** (Environment-as-Code) — environments are YAML files, version-controlled like code
- **Transparent** — developers never think about where their code runs; the platform decides
- **Extensible** — plugins add robotics, AI, database, security capabilities without touching core

---

## Core Capabilities

### 1. Environment-as-Code

A single `.tinybridge/env.yaml` file defines the entire development environment:

```yaml
apiVersion: tinybridge/v1
kind: Environment

metadata:
  name: terrain-mapper
  version: 1.0.0

substrate:
  os: ubuntu-24.04
  arch: [arm64, amd64]  # Rosetta 2 handles amd64

native:
  tools: [rust, python3.11, cmake, uv]

linux:
  packages:
    - ros-jazzy-desktop
    - gazebo-harmonic
  services:
    - postgres:16
    - redis:7

devices:
  usb:
    - class: serial      # all /dev/ttyUSB*
    - vendor: 0x2b03     # Ouster Lidar

remote:
  triggers:
    - cuda              # auto-route to GPU host
```

**One file. Version-controlled. Shareable with team. Diff-able. Reproducible forever.**

This replaces the current Nix flake + Lima YAML + docker-compose.yml + devcontainer.json nightmare.

### 2. Transparent Execution Routing

```
developer types:  tinybridge exec python train.py

↓ execution router decides:

  Is training.py pure Python?
    → no Linux-specific imports
    → runs natively on macOS (Tier 1) — **zero overhead**

  Does training.py import torch.cuda?
    → routed to remote GPU (Tier 3) automatically
    → developer sees single entry point; platform handles dispatch

  Does train.py call `ros2 launch`?
    → requires ROS 2 (Linux-only binary)
    → routed to Linux substrate (Tier 2) transparently
```

**Developers never specify a tier.** The router decides based on:
- Environment declaration (what's available)
- Import detection (what's needed)
- Hardware availability (which tier can execute)

### 3. Robotics-First Design

#### Multi-Environment DDS Discovery
- ROS 2 DDS multicast works out of the box (no `--network=host` workaround)
- Environments auto-assigned `ROS_DOMAIN_ID` (no inter-environment DDS bleed)
- `.tinybridge.local` domains resolve from macOS and Linux both
- `robot-1.tinybridge.local` accessible from talker in `robot-2` environment

#### Hardware Passthrough
- `tinybridge devices` — auto-discovers cameras, Lidar, serial devices, USB adapters
- Curated kernel module set: serial adapters (CP210x, CH341, FTDI), modems (Quectel), USB WiFi (RTL8x), SDR (RTL-SDR, HackRF), robotics (xpad, joydev)
- `tinybridge devices attach lidar:0` — Lidar available in substrate with zero latency
- Camera access: AVFoundation → v4l2 virtual device (seamless passthrough)

#### Simulation Integration
- Gazebo pre-configured in `robotics` template
- Foxglove visualization server auto-started
- Micro-ROS bridging for embedded systems

### 4. AI/ML Workflows

#### Local Inference (Fast Path)
```bash
# native MLX on macOS
tinybridge exec python -c "import mlx.core as mx; x = mx.random.normal((100, 100))"
→ runs natively with Metal acceleration (fastest)

# containerized PyTorch
tinybridge exec python -c "import torch; x = torch.randn(100, 100)"
→ routes to Linux substrate (Tier 2) for Linux-specific PyTorch
  → Metal passthrough (Phase 5) for GPU acceleration
```

#### Training Workflows (Cloud)
```bash
tinybridge exec python train.py --epoch=100 --batch=32

→ detects torch.cuda import
→ auto-routes to RunPod A100 ($0.53/hour)
→ syncs code + model weights
→ streams logs to local terminal
→ pulls results back when done
```

No manual SSH. No `.env` configuration. Transparent.

### 5. Data Engineering Support

Pre-configured services in `data` template:
- PostgreSQL 16 (schema inspection, lineage visualization)
- Kafka + schema registry
- Spark (with Metal GPU bridge in Phase 5)
- DuckDB (embedded OLAP)
- Iceberg + Nessie (open table formats)

`tinybridge doctor` auto-detects:
- Missing Python dependencies
- Postgres connectivity issues
- Kafka broker health
- Schema validation errors
- Performance bottlenecks

### 6. Diagnostic Intelligence

`tinybridge doctor` — AI-powered health check:

```
$ tinybridge doctor

✓ Linux substrate running (kernel 6.12.4, arm64 + amd64 via Rosetta)
✓ VirtioFS mounted (/Users/georgi → /home/georgi, 94% native I/O)
✓ DDS multicast active (3 ROS 2 nodes discovered, domain 42)
✓ Postgres accepting connections (10 active connections, 0 deadlocks)
⚠ Quectel modem at /dev/ttyUSB1 not attached
  → run: tinybridge devices attach ttyUSB1

✗ Remote GPU not configured
  → run: tinybridge remote add runpod --key $RUNPOD_KEY
  → run: tinybridge remote test torch.cuda

✓ 38 Python 3.11 packages installed
  → 0 security vulnerabilities (checked 2 minutes ago)
```

Actionable, not just warnings.

### 7. Parallel Environments

Enable AI agent workflows that OrbStack cannot support:

```bash
tinybridge env clone agent-1 agent-2 agent-3

# each gets:
# - isolated Linux substrate
# - auto-assigned port ranges (8080-8089, 9000-9009, etc.)
# - unique workspace (~/agent-1, ~/agent-2, ~/agent-3)
# - shared remote GPU queue (agent-1 job holds GPU, agent-2/3 queue)
```

Enables:
- Parallel testing of agent configurations
- Isolated experiment runs
- CI/CD parity (replicate GitHub Actions env locally)

### 8. Instant Snapshots & Rollback

```bash
tinybridge env snapshot checkpoint-v1
# make breaking changes...
tinybridge env rollback checkpoint-v1
# back to checkpoint in <3 seconds (CoW disk images)
```

Enables:
- Quick "what if?" experiments
- Recovering from config errors
- Version-controlled environment history

---

## Why This Matters

### For Roboticists
Today: Spend 2 hours debugging why their Docker ROS 2 setup cannot discover nodes. Multi-container networking, DDS multicast, USB device passthrough all require manual configuration.

With TinyBridge: `tinybridge create --template robotics`, 2 minutes later: multiple robots discovering each other via DDS, Gazebo running, Foxglove streaming sensor data, hardware attached. `tinybridge doctor` detects missing firmware and suggests fixes.

### For AI Engineers
Today: Develop locally in CPU-only containers, push to cloud for GPU training, deal with code sync issues, wait for results, pull them back. Context-switching hell.

With TinyBridge: `tinybridge exec python train.py` runs locally with Metal acceleration (Phase 5). If CUDA is needed, auto-routes to RunPod. Same entry point, transparent routing.

### For Data Engineers
Today: Postgres, Kafka, Spark running locally consume RAM constantly. File I/O through Docker bind mounts is 3-50x slower. Dbt materializations crawl.

With TinyBridge: Services auto-managed. VirtioFS >90% native I/O. Schema validation via StatGuardian contracts. `tinybridge doctor` catches data lineage issues before they hit production.

### For Platform Engineers
Today: Standardizing on Docker Desktop costs $21/user/month per enterprise employee. OrbStack is faster but macOS-only, locking out Linux/Windows developers. Lima is powerful but requires Ansible expertise.

With TinyBridge: Free, open-source, cross-platform (Phase 5 adds Linux/Windows). Declarative environments. Kubernetes templates. CI/CD parity.

---

## Distribution & Shipping

TinyBridge ships as a **real macOS product**:

- **macOS app** (`TinyBridge.app`) — Swift/SwiftUI, menu bar status, system extension for device access
- **CLI** (`tinybridge`) — single static Rust binary, installed to `/usr/local/bin`
- **Installer** — `.dmg` with code signing + Developer ID notarization (required for macOS Gatekeeper)
- **Package manager** — `brew install --cask tinybridge`
- **Auto-update** — Sparkle framework handles seamless updates

The user experience is **indistinguishable from a native macOS app** — not "another developer tool that happens to run on macOS."

---

## Competitive Positioning

### vs. OrbStack
- OrbStack: Proprietary, $8/user/month, macOS-only, zero GPU, no Environment-as-Code, ROS 2 broken
- **TinyBridge:** Open-source free, cross-platform (Phase 5), GPU Phase 5, Environment-as-Code core, ROS 2 native

TinyBridge wins on **trust, extensibility, robotics/AI integration, and cost.**

OrbStack will always have better GUI polish initially, but TinyBridge's **architectural foundations** (declarative environments, transparent execution routing, open-source plugin system) create defensible moats.

### vs. Docker Desktop
- Docker: Cross-platform, GUI polish, but $21/user/month enterprise, resource hog, slow file I/O
- **TinyBridge:** Free, optimized for macOS, >90% native I/O, declarative environments

TinyBridge wins on **cost, performance, and developer experience.**

### vs. Lima
- Lima: Powerful, open-source, but no GUI, requires Ansible expertise, no ecosystem
- **TinyBridge:** Open-source, has GUI, batteries-included (templates, AI diagnostics, device management), community ecosystem

TinyBridge wins on **accessibility and ecosystem.**

---

## 5-Phase Roadmap

**Phase 1 (Weeks 1-6): Foundation**
- Linux VM via Apple VZ Framework, <5s boot
- CLI + daemon + app shell
- `.dmg` installer, Homebrew cask
- **Ship Alpha:** Minimal viable Linux substrate

**Phase 2 (Weeks 7-12): Developer Experience**
- Execution router (Tier 1/2/3 dispatch)
- Environment templates (robotics, ai-local, data, cloudnative)
- Podman containers inside substrate
- VS Code Remote-SSH integration
- **Ship Beta:** Full developer workflow possible

**Phase 3 (Weeks 13-18): Hardware + Networking**
- USB/serial passthrough (curated kernel modules)
- DDS-aware networking (ROS 2 multicast)
- Camera passthrough (AVFoundation → v4l2)
- Parallel environments + auto port assignment
- Instant snapshots / rollback
- **Ship v1.0:** Robotics-grade stability

**Phase 4 (Weeks 19-24): Remote + Cloud**
- CUDA auto-detection + remote routing
- RunPod, Vast.ai, AWS integrations
- `tinybridge sync` for environment push/pull
- CI/CD parity mode
- **Ship v1.1:** AI/ML workflows complete

**Phase 5 (Weeks 25-34): GPU Bridge + Ecosystem**
- Vulkan-to-Metal GPU bridge (VirtioGPU Venus)
- Metal Compute forwarding (MLX support)
- WASM plugin architecture
- Security auditing + compliance exports
- Database native support (lineage, schema validation)
- Community template marketplace
- Windows/Linux host support
- **Ship v2.0:** Universal substrate

---

## Success Metrics (Year 1)

1. **Adoption:** 10K+ active developers within 12 months
2. **GitHub:** 5K+ stars, 200+ community contributions
3. **Robotics wedge:** #1 choice for ROS 2 developers on macOS
4. **AI/ML integration:** PyTerrainMap + PyStreamMCP + StatGuardian ship as official templates
5. **Ecosystem:** 50+ community plugins + templates
6. **Product velocity:** Monthly releases with community-sourced features

---

## Why Now?

1. **OrbStack reached maturity** (v2.2.1, 40+ releases in 18 months) but has **architectural limits**: no open-source option, no GPU, no Environment-as-Code
2. **Apple Silicon adoption complete** — Virtualization.framework is mature
3. **AI/robotics exploding** — demand for GPU routing + ROS 2 DDS is real and growing
4. **Community fatigue with closed-source** — Docker licensing, OrbStack pricing, VMware opacity drive demand for open alternative
5. **Declarative infrastructure trend** — Nix, Devbox, mise, Devcontainers mindshare ripe for TinyBridge's Environment-as-Code

TinyBridge arrives at **exactly the right moment** to establish a new category: **The open-source macOS development substrate.**

---

## Call to Action

TinyBridge is **shipping as a real product**: not a research project, not a side hustle, not a feature in another tool.

Target launch: **Q3 2026** (Phase 1 + 2, production-grade)

Target community: Roboticists, ML engineers, platform engineers, open-source contributors

Go-to-market: Hacker News, r/MacOS, robotics forums, ML community, open-source conferences

---

*TinyLx: Where macOS native performance meets Linux capability.*
