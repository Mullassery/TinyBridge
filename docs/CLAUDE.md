# TinyBridge: Project Context & Guidelines

## Project Overview

**TinyBridge** is an open-source, cross-platform Linux development substrate. Native Rust core (all platforms) + platform-specific UI (Swift on macOS). Intelligently routes workloads to the appropriate execution tier (native, containerized Linux, or remote GPU), with declarative Environment-as-Code at the core.

**Owner:** Georgi Mammen Mullassery
**License:** Proprietary
**Repository:** Private (Mullassery/tinybridge on GitHub)
**Status:** Phase 1 (In development) — macOS version shipping first

## Vision

Replace Docker Desktop and Lima as the preferred Linux development substrate on macOS. CLI-first, built for automation, scripting, and team workflows.

### Key Differentiators
- **Proprietary** (closed source, mullassery@gmail.com)
- **Dual-mode VM** (headless CLI-first, or windowed desktop on demand — same instance)
- **Environment-as-Code** (declarative YAML, git-versioned)
- **Full VM lifecycle** (suspend/resume, graceful shutdown, snapshots, resource scaling)
- **ROS 2 native** (DDS multicast networking works out of the box)
- **Transparent CUDA routing** (routes to remote GPU automatically)
- **Parallel environments** (AI agent workflows, team automation)
- **Hardware passthrough** (curated USB/serial/camera support)

### Competitive Positioning
- vs. Docker Desktop: Free, lighter, macOS-optimized, open source, better DX
- vs. Lima: GUI, batteries-included, ecosystem-rich, easier onboarding

## Technology Stack

### Technology Stack

**Philosophy:** Rust for performance and safety. CLI-first design for automation and scripting.

**Core:**
- **Control Daemon + CLI:** Rust + tokio (headless, always-on, AppKit-free)
- **VM Host Process:** Per-VM Rust binary linking Swift VZ bridge (owns VZVirtualMachine + optional NSWindow)
- **VM Backend:** Apple Virtualization.framework via Swift FFI
- **VM Substrate:** Real Linux kernel, VirtioFS, virtio-graphics, Rosetta 2

### No C++. Headless-First. Optional GUI.

Minimize C entirely (only for VZ Framework FFI). Zero Electron, zero native UI frameworks in the daemon. **AppKit lives only in per-VM host processes** — users who need pure headless server environments pay zero GUI overhead. When GUI is needed, one CLI command (`tinybridge gui`) attaches a window to the already-running VM without restart.

### Build System
- **Rust:** Cargo workspace, universal binary (arm64 + x86_64 via Rosetta)
- **Swift:** Xcode + SwiftPM, code-signed + notarized .dmg
- **CI/CD:** GitHub Actions (build → sign → notarize → Homebrew)

## Architecture

See `docs/ARCHITECTURE.md` for complete details.

**High-level (macOS):**
```
tinybridge up myproject              ← CLI-driven
    ↓
tinybridged daemon (headless)         ← Rust, JSON-RPC over Unix socket, no AppKit
    ↓
tinybridge-vmhost (per-VM)            ← Spawned child process, owns VZVirtualMachine
    ├─ VZ VM (headless by default)
    ├─ [Optional] NSWindow + display  ← On-demand: tinybridge gui
    └─ JSON-RPC socket for control
    ↓
SSH + Networking + Storage            ← Auto-configured
```

**Dual-mode flexibility:** Headless by default (zero GUI overhead), or attach a window to the same running VM with `tinybridge gui`—no restart, no separate VM, no "app must stay open" requirement.

## Implementation Phases

### macOS (Phase 1-5): Primary ship vehicle

| Phase | Duration | Goal | Deliverable |
|-------|----------|------|-------------|
| 1 | Weeks 1-6 | Core VM + CLI + daemon | Alpha: <5s boot, basic env management |
| 2 | Weeks 7-12 | Execution routing + templates | Beta: Full dev workflow possible |
| 3 | Weeks 13-18 | Hardware + DDS networking | v1.0: Robotics-grade macOS release |
| 4 | Weeks 19-24 | Remote GPU routing | v1.1: AI/ML workflows complete |
| 5 | Weeks 25-34 | GPU bridge + plugins | v2.0: macOS feature-complete |


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
- VirtioFS mounting, graphics device config
- Rosetta 2 configuration

### tinybridge-vmhost (Rust)
- Per-VM host process: owns `VZVirtualMachine`, manages lifecycle
- Spawned by daemon as isolated child process (crash isolation)
- Exposes JSON-RPC control socket (`vmhost.status`, `vmhost.show_window`, `vmhost.hide_window`)
- Links Swift VZ bridge and optional AppKit (window host) — only process that needs display APIs

### TinyBridgeApp (Swift)
- Menu bar app
- Environment list view
- Preferences UI
- System extension launcher

### TinyBridgeVZBridge (Swift)
- Wrapper around Virtualization.framework
- Exports minimal C header for Rust FFI
- ~500 lines total

## CLI Commands Reference

### Basic Lifecycle
```bash
tinybridge up myproject [--cpu 8] [--memory 16] [--disk 100]  # Create & start
tinybridge down myproject                                       # Stop
tinybridge suspend myproject                                    # Pause (preserves state)
tinybridge resume myproject                                     # Resume from pause
tinybridge shutdown myproject                                   # Graceful shutdown
tinybridge restart myproject                                    # Restart (same state)
```

### GUI Toggle
```bash
tinybridge gui myproject        # Attach display window
tinybridge headless myproject   # Detach display (VM keeps running)
```

### Resource Management
```bash
tinybridge update myproject --cpu 8 --memory 16 --disk 100
```

### Snapshots
```bash
tinybridge snapshot myproject create after-deploy
tinybridge snapshot myproject list
tinybridge snapshot myproject restore after-deploy
tinybridge snapshot myproject delete after-deploy
```

### Other
```bash
tinybridge shell myproject                      # Open shell
tinybridge status myproject                     # Check status
tinybridge list                                 # List all environments
tinybridge destroy myproject                    # Permanently delete
```

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
- Daemon: `tail -f ~/Library/Logs/tinybridge.log`

## Critical Decisions

1. **Rust for core** — Fastest, safest; single compilation model for all architectures
2. **Apple VZ on macOS** — Native VM backend via system framework; optimized performance
3. **Swift UI on macOS** — System integration (camera, USB, menu bar) native to OS
4. **Minimal Linux kernel** — Only curated modules needed for robotics/AI; optimized for fast boot
5. **VirtioFS over SSHFS** — >90% native I/O performance on macOS
6. **Homebrew distribution** — Native macOS package manager installation
7. **DDS-aware networking** — No workarounds for ROS 2; multicast passes through
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

## Known Limitations & Future Work

### Current (Phase 1-5 macOS)
- No GPU passthrough to Linux substrate (routes to remote instead)
- No devcontainers support yet (Phase 2)
- No TUI alternative to Swift app (Phase 5+)


### Future Roadmap
- **Phase 5:** Vulkan-to-Metal GPU bridge (VirtioGPU Venus + MoltenVK), Metal Compute forwarding (MLX/PyTorch-MPS), WASM plugin architecture

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
