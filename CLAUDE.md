# TinyBridge: Project Context & Guidelines

## Project Overview

**TinyBridge** is an open-source macOS native application that bridges Linux and macOS development. It intelligently routes workloads to the appropriate execution tier (native macOS, minimal Linux substrate, or remote GPU), with declarative Environment-as-Code at the core.

**Owner:** Georgi Mammen Mullassery
**License:** Apache 2.0
**Repository:** Private (Mullassery/tinybridge on GitHub)
**Status:** Phase 1 (In development)

## Vision

Replace Docker Desktop, OrbStack, and Lima as the preferred Linux development substrate for macOS developers, especially those building robotics, AI/ML, and data engineering systems.

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

### Core
- **Daemon + CLI:** Rust + tokio (performance, safety, single binary)
- **macOS App:** Swift + SwiftUI (native UI, system integration)
- **VZ Bridge:** Minimal C FFI only (Apple Virtualization Framework access)
- **Linux Substrate:** Apple VZ Framework, minimal kernel, VirtioFS, Rosetta 2

### No C++. Period.

The only C is a thin FFI header (~500 lines) connecting Swift VZ bindings to Rust. This is unavoidable and standard practice (Lima, Podman Machine use the same pattern).

### Build System
- **Rust:** Cargo workspace, universal binary (arm64 + x86_64 via Rosetta)
- **Swift:** Xcode + SwiftPM, code-signed + notarized .dmg
- **CI/CD:** GitHub Actions (build → sign → notarize → Homebrew)

## Architecture

See `docs/ARCHITECTURE.md` for complete details.

**High-level:**
```
Tier 1 (Native macOS)  ← default, zero overhead
    ↓
Tier 2 (Linux Substrate via Apple VZ Framework)  ← for Linux-only workloads
    ↓
Tier 3 (Remote Linux with GPU)  ← for CUDA/training
```

Routing is **transparent** — developers never specify a tier. The platform decides based on workload requirements.

## Implementation Phases

| Phase | Duration | Goal | Deliverable |
|-------|----------|------|-------------|
| 1 | Weeks 1-6 | Core VM + CLI | Alpha: <5s boot, basic env management |
| 2 | Weeks 7-12 | Execution routing + templates | Beta: Full dev workflow possible |
| 3 | Weeks 13-18 | Hardware + DDS networking | v1.0: Robotics-grade, stable |
| 4 | Weeks 19-24 | Remote GPU routing | v1.1: AI/ML workflows complete |
| 5 | Weeks 25-34 | GPU bridge + plugins | v2.0: Universal substrate |

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

### devforge-core (Rust)
- Shared types: `Environment`, `Workload`, `ExecutionTier`
- YAML schema parsing
- Configuration types

### devforge-daemon (Rust)
- Environment lifecycle (up/down/status)
- Execution router state machine
- Unix socket gRPC server
- Device manager (IOKit bridge calls)

### devforge-cli (Rust)
- CLI binary (`tinybridge` command)
- Calls daemon via gRPC
- User-facing output formatting

### devforge-vz (Rust)
- Safe wrapper around C FFI bindings
- VM lifecycle (boot, shutdown, snapshot)
- VirtioFS mounting
- Rosetta 2 configuration

### DevForgeApp (Swift)
- Menu bar app
- Environment list view
- Preferences UI
- System extension launcher

### DevForgeVZBridge (Swift)
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

1. **Apple VZ Framework over QEMU** — Better performance on macOS, native integration
2. **Minimal Linux kernel** — Only curated modules needed for robotics/AI; fast boot
3. **Rust for daemon/CLI** — Safety, performance, single binary distribution
4. **Swift for app** — Native macOS UX, system integration (camera, USB permissions)
5. **VirtioFS over SSHFS** — >90% native I/O performance
6. **Homebrew-only distribution (Phase 1-3)** — Simpler than Mac App Store, reaches developers
7. **DDS-aware networking** — No workarounds for ROS 2; multicast passes through
8. **Parallel environments** — CoW snapshots enable AI agent workflows

## Integration with Existing Portfolio

**Optional (Phase 5+):**
- **PyTerrainMap** → Ships as `tinybridge create --template pyterrain`
- **PyStreamMCP** → Optional intelligent routing layer
- **StatGuardian** → Environment contract validation (reproducibility)

These are plugins/templates, not hard dependencies.

## Distribution

**Phase 1-3:** Homebrew only (see `docs/HOMEBREW.md`)
**Phase 4+:** Optional Mac App Store tier

## Known Limitations & Future Work

### Current (Phase 1-4)
- No GPU passthrough to Linux substrate (routes to remote instead)
- No Windows/Linux host support (macOS only through Phase 4)
- No devcontainers support yet (Phase 2)

### Phase 5 Roadmap
- Vulkan-to-Metal GPU bridge (VirtioGPU Venus + MoltenVK)
- Metal Compute forwarding (MLX/PyTorch-MPS)
- WASM plugin architecture
- Cross-platform support (Linux/Windows hosts)

## Troubleshooting

**"VM won't boot"**
- Check Apple Virtualization.framework availability (macOS 14+)
- Verify disk space (Linux substrate needs 50GB sparse)
- Check logs: `devforgd` should have started

**"Slow file I/O"**
- Verify VirtioFS mounted (should be >90% native speed)
- Check for heavy I/O tasks (npm install, etc.)
- Profile with: `time devforge exec "find . | wc -l"`

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
