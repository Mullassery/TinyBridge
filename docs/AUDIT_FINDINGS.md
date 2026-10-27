# Audit Report: tinybridge
Date: 2026-10-26 | Type: Read-only systems design audit  
Repo: ../tinybridge (local: /Users/georgimullassery/Desktop/Projects/tinybridge) | Branch: main | Status: clean

## Overview
macOS-native Linux VM runtime built on Apple's Virtualization.framework. Rust workspace + C ABI + Swift bridge drives VZ, daemon over Unix socket JSON-RPC, CLI, per-VM vmhost process with codesigning/entitlements.

## Architecture / Entry points
- Workspace crates: core, cli, daemon, diagnostics, template-engine, error, vz-sys, vz, router, clipboard, ssh, templates, tunnel, dns, snapshots, devices, dds, vmhost (19 crates). Swift in `swift/`, assets, packaging.
- Entry points: CLI binary, `tinybridged` daemon, `tinybridge-vmhost`.

## Dependencies
Rust crates, Swift (VZ framework), bindgen for FFI, libc for syscalls. macOS-specific.

## Data / State
- VM configs, state, snapshots, assets. PidLock for process management. Unix sockets for IPC.

## APIs / Contracts
- JSON-RPC over Unix socket (daemon <-> CLI, vmhost protocol). SSH for guest access.

## Security observations
- Unsafe: 7 uses in crates (daemon health uses libc syscalls, vmhost main, vz FFI calls, clipboard macOS, examples) — necessary for FFI/syscalls but needs careful review.
- Socket permissions: daemon and vmhost set socket permissions to 0o600 (good). SSH keys set to 0o600.
- Path allowlist for VirtioFS (canonicalize + allowlist, reject .. escapes, read-only default) — strong control.
- Codesigning/entitlements required for VZ. PidLock/orphan detection.
- No network exposure by default (Unix sockets). Guest networking via DHCP lease parsing.
- Privilege separation: vmhost separate process.

## Tests / CI
- CI: assets.yml, ci.yml, commit-conventions.yml, homebrew-bump.yml. No unit tests found in repo scan (0). Heavy reliance on manual verification per README.

## Findings
| ID | Severity | Category | Evidence | Impact | Root cause | Recommendation | Confidence |
|---|---|---|---|---|---|---|---|
| TB-001 | High | Safety / Unsafe | crates/tinybridge-daemon/src/health.rs uses unsafe libc calls (sysconf, vm_statistics64, mach_host_self, statfs/statvfs); tinybridge-vz FFI unsafe; vmhost unsafe. | Memory safety bugs possible if FFI/contracts violated. | Required for macOS syscalls/VZ FFI. | Audit all unsafe blocks, document invariants, minimize unsafe surface, add safety comments, fuzz/test where possible. | High |
| TB-002 | Medium | Security / IPC | Unix sockets with 0o600 permissions set (good). Daemon/vmhost bound locally. | If permissions misconfigured or directory writable by others, IPC could be attacked. | Local IPC. | Ensure socket directory permissions restricted (0700), validate peer credentials if possible, handle TOCTOU. | High |
| TB-003 | Medium | Testing | No unit tests found; complex FFI/syscall code untested. | Regressions hard to catch. | Platform-specific, hard to test in CI. | Add unit tests where possible (pure logic), integration tests in controlled environments, property tests for path allowlist. | High |
| TB-004 | Low | Operability | Orphan detection via PidLock exists; health checks exist. | Good. | — | Continue hardening. | Medium |

## Strengths
- Proper privilege separation (vmhost process), strong path allowlist for host sharing, socket permissions enforced, codesigning/entitlements, real VZ integration, honest status reporting, guest IP from real DHCP leases.

## Prioritized remediation
1. Audit unsafe FFI/syscall boundaries (TB-001).
2. Harden socket dir permissions and peer validation (TB-002).
3. Expand test coverage for critical paths (TB-003).

## Limitations
Read-only audit; did not trace full VZ FFI contracts or vmhost lifecycle in detail.
