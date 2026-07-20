# TinyBridge

**Open-source macOS Linux development substrate.** Native performance. Linux capability. Zero VM pain.

## Overview

TinyBridge bridges macOS and Linux development by routing workloads transparently:

- **Tier 1 (Native macOS)** — Rust, Python, Node native binaries → zero overhead
- **Tier 2 (Linux Substrate)** — ROS 2, systemd, Linux-only ABIs → Apple VZ Framework headless kernel, <5s boot
- **Tier 3 (Remote GPU)** — CUDA workloads → RunPod, Vast.ai, AWS (transparent routing)

## Philosophy

- **Open Source** — Apache 2.0, no licensing fees, no vendor lock-in
- **Native First** — Swift/SwiftUI macOS app, Rust core, single static CLI binary
- **Environment-as-Code** — Declarative `.tinybridge/env.yaml`, git-versioned, reproducible
- **Developer-First** — Transparent execution routing, intelligent diagnostics, robotics/AI optimized

## Documentation

- **[Product Vision](docs/PRODUCT_VISION.md)** — Why TinyBridge exists, competitive positioning, success metrics
- **[Architecture Plan](docs/ARCHITECTURE.md)** — 5-phase roadmap, tech stack, workspace structure

## Status

🚧 **In development** (Phase 1: Core VM + CLI foundation)

- [ ] Phase 1: Linux substrate, CLI, `.dmg` installer (Weeks 1-6)
- [ ] Phase 2: Execution router, templates, VS Code (Weeks 7-12)
- [ ] Phase 3: USB/devices, DDS networking, snapshots (Weeks 13-18)
- [ ] Phase 4: Remote CUDA routing, cloud integration (Weeks 19-24)
- [ ] Phase 5: GPU bridge, plugin architecture, ecosystem (Weeks 25-34)

## License

Apache License 2.0 — See LICENSE file
