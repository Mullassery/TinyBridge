# TinyBridge

A macOS-native Linux VM runtime built on Apple's Virtualization.framework.

[![CI](https://github.com/Mullassery/TinyBridge/actions/workflows/ci.yml/badge.svg)](https://github.com/Mullassery/TinyBridge/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Platform: macOS](https://img.shields.io/badge/platform-macOS-lightgrey.svg)](#project-status)

## What it is

TinyBridge is a real Linux virtual machine runtime for macOS: a genuine Rust → C ABI → Swift call chain drives Apple's Virtualization.framework to boot an ARM64 Linux guest, provisioned via cloud-init, without a paid Apple Developer account. VMs run in their own codesigned `tinybridge-vmhost` process, supervised by a `tinybridged` daemon over Unix-socket JSON-RPC, with a CLI on top. It is a young, honest, single-maintainer project — the status notes below reflect what is actually built and verified.

## Features / Highlights

- **Real Virtualization.framework integration** — a Swift bridge (`swift/Sources/TinyBridgeVZBridge`), `bindgen`-generated FFI (`tinybridge-vz-sys`), and a per-VM `tinybridge-vmhost` process codesigned with the `com.apple.security.virtualization` entitlement. Verified end-to-end on Apple Silicon to boot a complete Ubuntu guest to a `login: ` prompt with working cloud-init credentials (2026-08-28).
- **Real guest networking** — guest IP is resolved from bootpd's actual DHCP lease file (`/var/db/dhcpd_leases`), never a hardcoded placeholder.
- **Cloud-init seed support** — `initrd` and `seed.iso` (NoCloud) inputs give a reproducible, credential-provisioned login.
- **MacOS-host directory sharing** — VirtioFS host-path scoping (canonicalize + allowlist, reject `..` escapes, read-only default) wired to the real `VZVirtioFileSystemDeviceConfiguration` FFI at VM-creation time.
- **Full VM lifecycle CLI** — `launch`, `shell`, `ssh`, `suspend`/`resume`, `snapshot`, `dds`, `doctor`, and more over a `0600`-permissioned Unix-socket daemon protocol.
- **Operational hardening** — orphaned-process detection (`PidLock`), runtime hypervisor checks in `tinybridge doctor` (`kern.hv_support`), fail-fast asset validation, and honest error handling.
- **Local port forwarding** — `tinybridge forward <env> --remote-port 80` opens
  an SSH local forward (`-L`) using TinyBridge's managed key; the guest must be
  reachable over SSH.

## Who it's for / Use cases

- Developers on **Apple Silicon or Intel Macs (macOS 13+)** who want a real, local Linux VM without a GUI hypervisor app.
- People evaluating an early-stage, code-first substrate for automating Linux VMs (boot, provision, snapshot, share host directories).
- Those happy to **build from source** and place guest assets (`kernel`, `initrd`, `disk.raw`, optional `seed.iso`) by hand — there is no automated image pipeline yet.

## What it is not

- **Not cross-platform.** macOS is the only working hypervisor backend. `crates/tinybridge-core/src/windows_adapter.rs` and `linux_adapter.rs` are unimplemented scaffolding (Hyper-V/KVM are never called). Building those backends isn't possible from a macOS dev machine, so they are deferred rather than faked.
- **Homebrew install is public, but formula skew is possible.** The tap is
  public (`brew install mullassery/tinybridge/tinybridge`), and the CLI formula
  tracks the latest release. Guest assets are still manual.
- **No turnkey guest images.** No automated download/build pipeline produces a bundled, bootable kernel+rootfs yet; you must acquire and place assets yourself (see `docs/BUILD_ASSETS_GUIDE.md`).
- **No published crates.** Nothing is on crates.io; the workspace is consumed from source.
- **Host→guest SSH not yet working out of the box.** On the machine where it was verified, SSH/TCP to the guest's confirmed IP is blocked by a macOS "Local Network" privacy consent, not by a code path issue — a one-time System Settings grant should resolve it.
- **Early stage.** No end-to-end boot-to-shell benchmarks, no independent security review, no Windows/Linux backends. Treat evaluation claims as what they are: verified on one Apple Silicon machine running macOS 26.

## Project status

`v0.7.0` (workspace), active and honest: CI is green (`cargo build --workspace`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`); `cargo test --workspace --exclude tinybridge-daemon` passes 408/408 and `cargo test -p tinybridge-daemon` passes 194/194. The macOS backend has been verified to boot a real guest to a working cloud-init login; see `ROADMAP_HONEST.md` and the root-level status notes in [docs/README.md](docs/README.md) for the full, unfiltered picture. No backward-compatibility or API-stability guarantees yet — it is a single-maintainer substrate, not a released platform.

## Installation

Building from source is the only currently working install path:

```bash
git clone https://github.com/Mullassery/TinyBridge.git
cd TinyBridge

# Builds the Swift bridge, copies the dylib where Cargo expects it, builds the
# workspace, and ad-hoc codesigns tinybridge-vmhost with the virtualization
# entitlement (no paid Apple Developer account needed).
just build

# Or, without `just`:
swift build --package-path swift/ -c release
mkdir -p target/swift-libs
cp swift/.build/release/libTinyBridgeVZBridge.dylib target/swift-libs/
cargo build --workspace
codesign --force --sign - \
  --entitlements crates/tinybridge-vmhost/tinybridge-vmhost.entitlements \
  target/debug/tinybridge-vmhost
```

### Requirements (macOS)

- macOS 13.0+ on Apple Silicon or Intel (Virtualization.framework requirement)
- Rust (see `rust-toolchain.toml`) and Swift (Xcode Command Line Tools are sufficient)
- `just` for the `justfile` recipes, optional

### Guest assets

You must place `kernel`, `initrd`, `disk.raw` (and optionally a cloud-init `seed.iso`) under `dirs::cache_dir()/TinyBridge/assets/`. See [docs/BUILD_ASSETS_GUIDE.md](docs/BUILD_ASSETS_GUIDE.md) for building these; the verified path extracts the matching kernel+initrd from a real Ubuntu cloud image's own `/boot`. `cargo test --workspace` needs `DYLD_LIBRARY_PATH=target/swift-libs` (absolute path) so the `tinybridge-vz` test binaries find the Swift dylib — see `ci.yml` for the exact CI invocation.

## Quick start

```bash
just build

# Start the daemon, launch an environment, shell in:
tinybridged &
tinybridge launch myenv --template ubuntu
tinybridge status myenv
tinybridge shell myenv
```

If assets are missing, `launch` fails fast with an actionable error naming the missing file(s).

## Usage

The real command surface (from `crates/tinybridge-cli/src/main.rs`):

```
tinybridge launch      Launch a new environment        tinybridge snapshot   Manage snapshots
tinybridge down        Stop an environment              tinybridge doctor     Run system diagnostics
tinybridge gui         Attach a display window          tinybridge templates  List templates
tinybridge headless    Detach display (VM keeps running)  tinybridge images   List images
tinybridge suspend     Suspend (pause, preserving state) tinybridge dds      Manage DDS networking
tinybridge resume      Resume a suspended environment   tinybridge status    Show environment status
tinybridge shutdown    Gracefully shut down             tinybridge list      List environments
tinybridge restart     Restart an environment           tinybridge shell     Interactive shell
tinybridge repair      Re-provision SSH/DDS config      tinybridge ssh       SSH into an environment
tinybridge forward     Forward a local TCP port into    tinybridge forward app --local-port 8080 --remote-port 80
tinybridge destroy     Destroy an environment           tinybridge logs      Tail the daemon log
tinybridge up          Start (legacy alias for launch)  tinybridge update    Manage environment resources
```

Run `tinybridge --help` or `tinybridge <command> --help` for current usage — that is the source of truth.

`tinybridge logs` tails the daemon log at `~/Library/Logs/TinyBridge/tinybridged.log`
(written by `tinybridged`), with `--tail N` / `--errors` / `--warnings` / `--follow`; use
`--file` or `TINYBRIDGE_LOG_FILE` to point at a different log.

## Documentation

- [docs/README.md](docs/README.md) — documentation index with per-doc accuracy notes
- [docs/BUILD_ASSETS_GUIDE.md](docs/BUILD_ASSETS_GUIDE.md) — building guest kernel/rootfs assets
- [docs/CLIPBOARD.md](docs/CLIPBOARD.md) — macOS↔Linux clipboard bridge (unverified end-to-end until host→guest SSH works)
- [docs/DDS_CLI_REFERENCE.md](docs/DDS_CLI_REFERENCE.md) — `tinybridge dds` command reference
- [docs/SWIFT_VZ_BUILD.md](docs/SWIFT_VZ_BUILD.md) — building the Swift Virtualization.framework bridge
- [ROADMAP_HONEST.md](ROADMAP_HONEST.md) — what's built, verified, broken, or deferred
- [SECURITY.md](SECURITY.md) — security posture and reporting

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) (dev setup, test/lint commands) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Security issues: report via [SECURITY.md](SECURITY.md), not public issues. Behavior changes are tracked in [CHANGELOG.md](CHANGELOG.md).

## Related projects

- [MudFish](https://github.com/Mullassery/MudFish) — Rust-native web crawling and intelligence engine with Python bindings
- [MessageBirds](https://github.com/Mullassery/MessageBirds) — open-source, event-driven Customer Data & Engagement Platform (Rust + TypeScript)
- [PrismNote](https://github.com/Mullassery/prismnote) — Jupyter-compatible data-science notebook with a Rust backend
- [SHER-KERNEL](https://github.com/Mullassery/SHER-KERNEL) — userspace prototype of OS-kernel object-model/scheduling/memory concepts
- [PyAPICheck](https://github.com/Mullassery/PyAPICheck) — transparent API security discovery with a Rust core and Python CLI/SDK

[Browse all projects →](https://github.com/Mullassery?tab=repositories)

## License

Licensed under the [Apache License 2.0](LICENSE).

Contact: mullassery@gmail.com — see [SECURITY.md](SECURITY.md) for security-specific reporting.