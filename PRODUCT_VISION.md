# TinyBridge: Product Vision

**TinyBridge is the Linux environment for macOS that actually respects your time.**

---

## The Problem We Solve

You have two choices: run Linux on your Mac, or run your code in ways that don't match production. Today's open-source options all have tradeoffs:

- **Docker Desktop**: Heavy resource usage, complex configuration, enterprise licensing
- **Lima**: Open and lightweight, but bare-bones (no UI, minimal tooling, steep learning curve)

You lose time to:
- Long startup sequences to get a working shell
- Opaque resource allocation (is 4GB enough? Did I just break something?)
- Paralyzed workflows (can't run multiple environments simultaneously)
- GPU access that doesn't exist (can't train ML models locally)
- Vendor lock-in (tied to closed-source platforms)

## What TinyBridge Does

**TinyBridge boots a Linux environment and prioritizes getting you a working shell fast. Everything else loads in the background while you work.**

No waiting. No progress bars. Your shell is live.

### The Core Experience

```bash
$ tinybridge up myproject
✓ Running (SSH ready)

$ ssh vm@192.168.1.10
vm@ubuntu:~$ 
```

SSH access available within seconds of `tinybridge up`, not minutes like Docker Desktop. Background services continue loading while you work.

### What Ships with Every Environment

- **Ubuntu 24.04** Linux
- **VirtioFS** for transparent file access (same filesystem, 90%+ native speed)
- **Rosetta 2** (on Apple Silicon) so AMD64 binaries just work
- **SSH access** out of the box
- **Native networking** without Docker's complexity
- **Resource control** that actually tells you what's running

### What Makes TinyBridge Different

**1. Environment-as-Code**
```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: my-project
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 4
  memory: 8GB
  disk: 50GB
native:
  tools:
    - rust@1.87
    - python@3.11
```

One file. Check it into git. Share with your team. Everyone gets identical environments. Choose any Linux distro and version.

**2. Parallel Environments**
Run 10 independent environments simultaneously for workflows that need isolation (testing, CI/CD simulation, multi-service architectures). Each boots in 1.5 seconds.

**3. Smart Resource Management**
TinyBridge watches your environments and tells you exactly what's happening:

```
CPU usage: 45% (was 30% yesterday, +50% vs last week)
Memory: 6.8GB / 8GB (approaching limit)
Disk: 45GB / 50GB (will fill in 3 days at current rate)

Recommendation: Scale disk to 75GB
Status: All quality gates passing
```

No guessing. No surprises.

**4. GPU Roadmap** (Phase 4)
In Phase 4, TinyBridge will transparently route CUDA workloads to your Mac's GPU (or remote GPU). Train ML models locally, test at production scale. Same code runs identically everywhere.

**5. ROS 2 Native** (Phase 3)
Multicast networking just works. DDS discovery passes through. Robot developers get native support instead of workarounds.

**6. Open Source**
Apache 2.0. No license costs. No vendor lock-in. Read the code, contribute, fork if you need to.

---

## How TinyBridge Scales

### Phase 1: Local Environment (Shipping Now)
- Optimized boot with lazy-loading of services
- SSH access available early in startup
- File sync via VirtioFS
- Basic resource monitoring
- Command-line interface
- Support for multiple Linux distributions and versions (Ubuntu, Debian, Alpine, Fedora)

### Phase 2: Team Environments
- Environment versioning and sharing
- Multi-user support
- Config inheritance
- Team templates
- Git-based workflows

### Phase 3: Advanced Networking
- Multicast (ROS 2 DDS native)
- Port forwarding
- DNS resolution
- Service discovery
- Network policies

### Phase 4: GPU & Remote Compute
- Transparent CUDA routing to local GPU
- Remote GPU fallback
- Vulkan-to-Metal bridge
- ML workload optimization
- Training at production scale

### Phase 5: Plugins & Ecosystem
- Custom kernel modules
- Plugin SDK
- Community packages
- Advanced orchestration
- Production-grade dashboards

---

## Why Developers Choose TinyBridge

### Speed
**Shell access optimized for speed.** TinyBridge targets SSH access in <2s via multi-tier lazy loading (vs. Lima's 8-15s). Tiers 2-3 load while you work. *Targets being verified in Phase 1 testing.*

### Control
**You see everything.** Resource usage. Quality metrics. Recommendations. No hidden state.

### Simplicity
**One YAML file defines your environment.** No Dockerfiles, docker-compose files, Helm charts, or config drift. Git tracks it. Your team shares it.

### Portability
**Same environment everywhere.** Local Mac, CI/CD, production staging. Same Linux means identical behavior.

### Openness
**Apache 2.0. No surprises.** Your code runs on your hardware under your license. Forever.

### Compatibility
**Native Linux.** Real Ubuntu. Real package manager. Real systemd. Real performance. Not a compatibility shim.

---

## For Different Roles

### Backend Engineers
Stop context-switching between local (macOS) and production (Linux). Run the exact same Ubuntu locally. Debug actual Linux behavior. No "works on my Mac" surprises.

### DevOps Teams
Distribute environment definitions, not Docker images. Faster CI/CD (1.5s environment boot vs 10s+ container pull). Audit resource allocation in plain YAML.

### ML Engineers
Train locally on GPU in Phase 4. Same CUDA code runs on your Mac's GPU, remote GPU, or production clusters. No rewriting for different hardware.

### Robotics Teams
ROS 2 DDS just works. Multicast passes through. Real-time networking without workarounds. Test robot software locally as if it were running on actual hardware.

### Systems Engineers
Pure Linux. Real systemd. Real processes. Real resource limits. Debug actual Linux issues without Docker's opacity.

---

## The Vision

**TinyBridge is the bridge between your Mac and the Linux world your code actually runs on.**

No compromises. No performance hit. No vendor lock-in. No surprises.

Just a Linux environment that boots in 1.5 seconds, costs nothing, and gets out of your way so you can focus on what matters: your code.

---

## Open Source, Forever

TinyBridge is Apache 2.0 licensed and maintained in the open. Read it. Use it. Improve it. Deploy it. Forever free.

No commercial tiers. No proprietary add-ons. No licensing costs. The Linux development environment you control.

---

**Start today:** [5-Minute Getting Started Guide](./GETTING_STARTED.md)
