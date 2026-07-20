# TinyBridge: Project Context & Guidelines

## Project Overview

**TinyBridge** is an open-source, cross-platform Linux development substrate. Native Rust core (all platforms) + platform-specific UI (Swift on macOS). Intelligently routes workloads to the appropriate execution tier (native, containerized Linux, or remote GPU), with declarative Environment-as-Code at the core.

**Owner:** Georgi Mammen Mullassery
**License:** Apache 2.0
**Repository:** Private (Mullassery/tinybridge on GitHub)
**Status:** Phase 1 (In development) — macOS version shipping first, Windows/Linux in Phase 2

## Vision

Replace Docker Desktop, OrbStack, and Lima as the preferred Linux development substrate. Initially for macOS (Phase 1-4), expanding to Windows/Linux (Phase 2+) with native performance on each platform.

### Key Differentiators
- **Open source** (vs. OrbStack's closed proprietary)
- **Environment-as-Code** (declarative YAML, git-versioned)
- **ROS 2 native** (DDS multicast networking works out of the box)
- **Transparent CUDA routing** (routes to remote GPU automatically)
- **Parallel environments** (AI agent workflows)
- **Hardware passthrough** (curated USB/serial/camera support)

### Competitive Positioning
- vs. OrbStack: Open source, cheaper, robotics-native, GPU roadmap
- vs. Docker Desktop: Free, lighter, macOS-optimized, better DX
- vs. Lima: GUI, batteries-included, ecosystem-rich

## Technology Stack

### Dual-Platform Strategy: Rust-Maximized

**Philosophy:** Rust for all platforms (performance, safety). Platform-specific UIs only where native integration needed (Swift on macOS).

**macOS (Phase 1-4):**
- **Daemon + CLI:** Rust + tokio
- **UI:** Swift + SwiftUI (menu bar app, native system integration)
- **VM Backend:** Apple VZ Framework via C FFI (~500 lines; unavoidable for system APIs)
- **VM Substrate:** Minimal Linux kernel, VirtioFS, Rosetta 2

**Windows/Linux (Phase 2+):**
- **Daemon + CLI:** Rust + tokio (same core as macOS)
- **UI:** Headless CLI + optional Tauri/egui desktop app
- **VM Backend:** QEMU or KVM native bindings (pure Rust via rust-qemu or libvirt crates)
- **VM Substrate:** Standard Linux minimal kernel, virtio, KVM support

### No C++. Period.

Minimize C entirely. macOS uses a thin FFI header for VZ Framework (unavoidable system API access, same pattern as Lima/Podman). Windows/Linux uses pure Rust VM bindings where available.

### Build System
- **Rust:** Cargo workspace, universal binary (arm64 + x86_64 via Rosetta)
- **Swift:** Xcode + SwiftPM, code-signed + notarized .dmg
- **CI/CD:** GitHub Actions (build → sign → notarize → Homebrew)

## Architecture

See `docs/ARCHITECTURE.md` for complete details.

**High-level (macOS):**
```
Tier 1 (Native macOS arm64/x86_64)  ← default, zero overhead
    ↓
Tier 2 (Linux Substrate via Apple VZ Framework)  ← for Linux-only workloads
    ↓
Tier 3 (Remote Linux with GPU)  ← for CUDA/training
```

**High-level (Windows/Linux):**
```
Tier 1 (Native Windows/Linux amd64)  ← default, zero overhead
    ↓
Tier 2 (Containerized Linux via QEMU/KVM)  ← for isolated workloads
    ↓
Tier 3 (Remote Linux with GPU)  ← for CUDA/training
```

Routing is **transparent** — developers never specify a tier or OS. The platform decides based on workload + host OS.

## Implementation Phases

### macOS (Phase 1-5): Primary ship vehicle

| Phase | Duration | Goal | Deliverable |
|-------|----------|------|-------------|
| 1 | Weeks 1-6 | Core VM + CLI + daemon | Alpha: <5s boot, basic env management |
| 2 | Weeks 7-12 | Execution routing + templates | Beta: Full dev workflow possible |
| 3 | Weeks 13-18 | Hardware + DDS networking | v1.0: Robotics-grade macOS release |
| 4 | Weeks 19-24 | Remote GPU routing | v1.1: AI/ML workflows complete |
| 5 | Weeks 25-34 | GPU bridge + plugins | v2.0: macOS feature-complete |

### Windows/Linux (Phase 2+): Parallel development starts Week 7

| Phase | Duration | Goal | Deliverable |
|-------|----------|------|-------------|
| 2+ | Weeks 7-12 | QEMU/KVM Rust backend + CLI | Beta: Windows/Linux CLI working |
| 3+ | Weeks 13-18 | Hardware + DDS networking | v1.0: Windows/Linux feature-parity |
| 4+ | Weeks 19-24 | Remote GPU routing | v1.1: Cross-platform GPU support |
| 5+ | Weeks 25-34 | Optional TUI/egui app | v2.0: Full feature parity |

## Code Style & Conventions

### Rust
- **Edition:** 2021
- **MSRV:** 1.97 (use latest stable features)
- **Style:** rustfmt (auto-format)
- **Linting:** clippy (strict mode)
- **Testing:** All public APIs tested; >80% coverage target
- **Documentation:** Doc comments on public items, especially trait impls
- **Dependencies:** Minimize; prefer std library when reasonable
- **Async:** tokio for runtime, prefer structured concurrency
- **Error handling:** anyhow/thiserror for Result types; propagate with ?

### Swift
- **Language:** Swift 5.10+
- **Style:** SwiftLint + Apple conventions
- **Naming:** camelCase for properties/methods, PascalCase for types
- **Architecture:** MVVM for UI, dependency injection for services
- **Concurrency:** async/await (not Combine)
- **Performance:** Profile before optimizing; use Instruments

### Git & Commits
- **Commits:** Atomic, single responsibility per commit
- **Messages:** Conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`, etc.)
- **Co-author:** Always include `Co-Authored-By: Claude Haiku 4.5 <noreply@anthropic.com>` in commits
- **No amending:** Create new commits, don't amend existing ones
- **Branches:** Feature branches off main; PR workflow

## Key Interfaces & Modules

### tinybridge-core (Rust)
- Shared types: `Environment`, `EnvironmentStatus`, `SubstrateConfig`
- YAML schema parsing (`env.yaml`)
- IPC protocol types (JSON-RPC 2.0)
- Configuration types

### tinybridge-daemon (Rust)
- Environment lifecycle (up/down/status)
- Execution router state machine
- Unix socket JSON-RPC server
- Device manager (IOKit bridge calls)

### tinybridge-cli (Rust)
- CLI binary (`tinybridge` command)
- Calls daemon via Unix socket + JSON-RPC
- User-facing output formatting (tables, status badges)

### tinybridge-vz (Rust)
- Safe wrapper around C FFI bindings
- VM lifecycle (boot, shutdown, snapshot)
- VirtioFS mounting
- Rosetta 2 configuration

### TinyBridgeApp (Swift)
- Menu bar app
- Environment list view
- Preferences UI
- System extension launcher

### TinyBridgeVZBridge (Swift)
- Wrapper around Virtualization.framework
- Exports minimal C header for Rust FFI
- ~500 lines total

## Development Workflow

### Local Setup
```bash
git clone https://github.com/Mullassery/tinybridge.git
cd tinybridge

# Rust setup
rustup default stable
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# Swift setup
xcode-select --install

# Build everything
cargo build
xcodebuild -scheme TinyBridge build
```

### Testing
```bash
# Rust tests
cargo test --workspace

# Swift tests
xcodebuild -scheme TinyBridge test

# Integration tests
cargo test --test integration_tests -- --include-ignored
```

### Debugging
- Rust: `RUST_LOG=debug cargo run`
- Swift: Xcode debugger
- Daemon: `tail -f /var/log/tinybridge.log`

## Critical Decisions

1. **Rust for core (all platforms)** — Fastest, safest; single compilation model across macOS/Windows/Linux
2. **Apple VZ on macOS, QEMU/KVM on Windows/Linux** — Native VM backend per platform, not cross-platform abstraction
3. **Swift UI on macOS only** — System integration (camera, USB, menu bar) worth the effort; headless CLI sufficient elsewhere
4. **Minimal Linux kernel** — Only curated modules needed for robotics/AI; fast boot
5. **VirtioFS over SSHFS** — >90% native I/O performance on macOS; virtiofs on Linux
6. **Homebrew (macOS) + MSI/scoop/snap (Windows/Linux)** — Platform-native installers
7. **DDS-aware networking** — No workarounds for ROS 2; multicast passes through on all platforms
8. **Parallel environments** — CoW snapshots enable AI agent workflows

## Integration with Existing Portfolio

**Optional (Phase 5+):**
- **PyTerrainMap** → Ships as `tinybridge create --template pyterrain`
- **PyStreamMCP** → Optional intelligent routing layer
- **StatGuardian** → Environment contract validation (reproducibility)

These are plugins/templates, not hard dependencies.

## Distribution

**macOS (Phase 1-4):** Homebrew only (see `docs/HOMEBREW.md`)  
**macOS (Phase 5+):** Optional Mac App Store tier  
**Windows (Phase 2+):** MSI + Scoop/Chocolatey  
**Linux (Phase 2+):** snap + APT/RPM packages

## Known Limitations & Future Work

### Current (Phase 1-5 macOS)
- No GPU passthrough to Linux substrate (routes to remote instead)
- No devcontainers support yet (Phase 2)
- No TUI alternative to Swift app (Phase 5+)

### Current (Phase 2+ Windows/Linux)
- CLI-only initially (Phase 2); Tauri/egui desktop app in Phase 5+
- QEMU/KVM performance not as optimized as Apple VZ (Phase 3+)

### Future Roadmap
- **macOS Phase 5:** Vulkan-to-Metal GPU bridge (VirtioGPU Venus + MoltenVK), Metal Compute forwarding (MLX/PyTorch-MPS)
- **Windows/Linux Phase 2+:** QEMU/KVM optimizations, optional Tauri desktop app
- **All platforms Phase 5+:** WASM plugin architecture, devcontainers support

## Troubleshooting

**"VM won't boot"**
- Check Apple Virtualization.framework availability (macOS 14+)
- Verify disk space (Linux substrate needs 50GB sparse)
- Check logs: `tinybridged` should have started

**"Slow file I/O"**
- Verify VirtioFS mounted (should be >90% native speed)
- Check for heavy I/O tasks (npm install, etc.)
- Profile with: `time tinybridge exec "find . | wc -l"`

**"ROS 2 DDS not discovering"**
- Verify multicast-aware networking enabled in env.yaml
- Check firewall/network rules
- Confirm `ROS_DOMAIN_ID` set correctly

## Resources

- Product Vision: `docs/PRODUCT_VISION.md`
- Architecture: `docs/ARCHITECTURE.md`
- Homebrew Strategy: `docs/HOMEBREW.md`
- GitHub: https://github.com/Mullassery/tinybridge (private)

---

**Last Updated:** 2026-07-20
**Next Phase Review:** After Phase 1 completion
