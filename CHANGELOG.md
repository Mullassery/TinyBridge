# Changelog

All notable changes to this project are documented in this file. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Entries below `[Unreleased]` are reconstructed only where confidently verifiable (git tag
dates); this project did not keep a changelog before this pass, so no per-release feature
lists are fabricated for the historical tags. See `git log <tag>..<tag>` or
[GitHub Releases](../TinyBridge/releases) for the real commit
history of each version.

## [0.8.0] - 2026-10-19

### Added
- `tinybridge assets fetch --manifest assets.json [--dest DIR]`: downloads guest
  assets via curl and verifies each SHA-256 before moving it into place. Existing
  verified files are skipped; mismatches fail closed. Asset names are confined
  to the destination directory.
- Unit tests cover manifest parsing and path/checksum validation.

## [0.7.0] - 2026-10-19

### Added
- `tinybridge forward <env> --remote-port <port>` opens a local SSH `-L`
  forward (default local port 8080, guest host localhost) using TinyBridge's
  managed SSH key. Foreground process; Ctrl-C stops it. Requires guest SSH to
  be reachable. Unit tests cover the argument builder and extra SSH options.

## [0.6.3] - 2026-10-19

### Added
- **Homebrew formula bump is now automated on release** (RECHECK-061). Publishing
  a GitHub release dispatches a `tinybridge-release` event at
  `Mullassery/homebrew-tinybridge` (`.github/workflows/homebrew-bump.yml`), which
  computes the tarball's sha256 and rewrites `Formula/tinybridge.rb`'s `url` +
  `sha256` together (`homebrew-tinybridge`'s `bump-formula` workflow). Replaces the
  manual post-release formula edit; `scripts/release.sh` now prints the automation
  (and a `tools/bump_formula.sh` manual fallback) instead of a "update the formula"
  reminder.
- **File-sharing throughput harness** (RECHECK-062). `scripts/bench-fileio.sh`
  measures write/read throughput of the shared folder inside the guest against the
  guest's own disk as a control, and writes JSON to `benchmark-results/`.
  `docs/FILE_IO_BENCHMARK.md` documents the method and states plainly that the
  "~90% native file I/O" and "1.5s boot" figures are unverified targets.

### Fixed
- **Linux 9p shared folders no longer silently pretend to mount** (RECHECK-062).
  `mount_storage` records the share in metadata but does not modify the libvirt
  domain XML, so the folder never appears in the guest. It now emits an explicit
  `WARNING:` instead of a "Mounting … via 9p" line that read as success.

## [Unreleased]

### Added
- **`tinybridge logs` now reads a real daemon log instead of printing a
  placeholder** (RECHECK-010). The `tinybridged` daemon tees its tracing output
  to both stderr *and* `~/Library/Logs/TinyBridge/tinybridged.log`; `tinybridge
  logs` tails that file with `--tail N`, `--errors`, `--warnings` and
  `--follow` (poll-based, `tail -f`-style), and `--file` / `TINYBRIDGE_LOG_FILE`
  can point it elsewhere. When the file is absent it now says so and explains
  how to produce one, rather than printing `[Log output would appear here]`.
  Unit-tested (`read_tail`, level/name filtering).

### Fixed (from the SHER org-wide Ubuntu/Linux-Rust compatibility audit, 2026-10-04)
- **`cargo fmt --check` drift in `crates/tinybridge-cli/tests/cli_args.rs`.** Two
  `assert!` call sites had fallen out of rustfmt formatting (whitespace-only).
  Restored `cargo fmt --check` to exit 0, matching the root README's lint claim
  and the `justfile`'s lint recipe.
- **`otel` feature broke `--all-features` builds.** The feature's own doc comment
  says enabling it "does nothing yet," but `otel_provider.rs`'s
  `#[cfg(feature = "otel")]` block referenced `opentelemetry`/`opentelemetry_jaeger`
  crates that were never declared as dependencies — `cargo clippy --all-targets
  --all-features` and `cargo build --features otel` failed with `E0432 unresolved
  import`. Removed the broken block so the feature is a true no-op as documented;
  added a regression test covering `enable_tracing`/`enable_metrics = true` config,
  which previously only compiled with the feature disabled.

### Removed
- **`crates/tinybridge-macos/`** (757-line SwiftUI menu-bar app prototype). It was not a
  Cargo crate, not a workspace member, not built by CI, and talked to the daemon over a
  hardcoded TCP port while the real daemon speaks Unix-socket JSON-RPC -- it could not have
  connected to the real daemon even if built by hand. The nine docs describing it (or the
  `.dmg`/Homebrew-cask distribution around it) as shipping were already archived in an
  earlier pass. Finishing it for real would have meant a genuine rebuild (new transport,
  real workspace/CI/distribution integration), not a quick patch; deleted rather than
  carrying unbuilt, disconnected, misleadingly-placed code indefinitely. See
  `docs/internal/ROADMAP_HONEST.md` section 3.

### Fixed
- **The `unsafe impl Send`/`Sync` for `tinybridge_vz::VirtualMachine` was
  justified only by the one-line comment "the C FFI handles thread safety"**
  (RECHECK-004). The impls are sound, but the reasoning is now written down
  against the concrete mechanism that makes them sound: every `tb_vm_*` entry
  point in `TinyBridgeVZ.swift` is marshalled onto the main actor (because
  `VZVirtualMachine(configuration:)` is main-queue-bound) and the shared
  `vmInstances` map is guarded by `vmLock`. A `const _: fn()` assertion pins
  `VirtualMachine: Send + Sync` so the guarantee can't be dropped silently.
- **The daemon health report's memory and disk checks were fake.** `check_disk()`
  hardcoded `available_gb: 50` and always returned Healthy regardless of the real
  filesystem (that value was literal JSON, not a measurement), and `check_memory()` on
  macOS used a broken `sysctl` call whose "available" page count was always `0` (so memory
  was *always* reported Degraded), while every non-macOS platform was hardcoded Healthy
  with "Memory check not implemented". Both are now real, measured per check:
  - `check_memory()`: free+inactive+speculative pages via `host_statistics64` on macOS
    (plus real total via `sysctl(HW_MEMSIZE)`, fixing the prior broken call) and
    `MemAvailable`/`MemTotal` from `/proc/meminfo` on Linux. Unsupported platforms report
    Degraded with an explicit "not supported" message instead of a silent healthy.
  - `check_disk()`: real `statfs` (macOS) / `statvfs` (Linux) on the filesystem holding
    `TinyBridgeConfig::data_dir()` (walking up to the nearest existing ancestor since the
    dir is created lazily); new `available_gb`/`total_gb` details and a real
    `>= 10 GB` threshold.
  - New `test_memory_check_is_real` / `test_disk_check_is_real` assert the reported
    numbers are genuine host measurements (e.g. `total_mb > 0`,
    `available <= total`).
- **VirtioFS host-directory sharing is now wired to the real FFI.** Virtualization.framework
  has no API to hot-add a directory share to an already-created VM --
  `VZVirtioFileSystemDeviceConfiguration`/`VZSharedDirectory` must be set on
  `VZVirtualMachineConfiguration.directorySharingDevices` *before*
  `VZVirtualMachine` is constructed -- so `VirtioFS::attach(&self, vm: &VirtualMachine)`
  could never have worked regardless of implementation; it still returns a real, honest
  error. The real fix moves share configuration to VM-creation time: `TBVMConfig` (C ABI)
  gained `virtiofs_shares`/`virtiofs_share_count`; `TinyBridgeVZ.swift`'s `tb_vm_create`
  now builds real `VZSharedDirectory` + `VZVirtioFileSystemDeviceConfiguration` objects
  from them before `vmConfig.validate()`; `VmConfig` (Rust) gained
  `with_virtiofs_share(spec)`; `VirtioFS::into_spec(allowed_roots)` validates scope (same
  `validate_scope` logic as before) and produces the spec. Verified with a real
  ad-hoc-codesigned test binary (the `com.apple.security.virtualization` entitlement a
  plain `cargo test` binary doesn't have): a VM configured with a real, validly-scoped
  VirtioFS share passes `VZVirtualMachineConfiguration.validate()` and
  `VZVirtualMachine(configuration:)` construction. See `docs/internal/ROADMAP_HONEST.md` for full detail.
- `EnvironmentManager::up()` (`tinybridge launch`'s real code path) called
  `VmManager::create_vm` with no way to pass an initrd or cloud-init seed
  image, so it could never use the same config the only proven-working
  guest boot (2026-08-28) actually needs - `create_vm`/`tinybridge-vmhost`
  now accept optional `initrd`/`seed_image` paths
  (`TINYBRIDGE_INITRD_PATH`/`TINYBRIDGE_SEED_IMAGE_PATH`) and thread them
  into `VmConfig::with_initrd`/`with_seed_image`. `up()` also now fails
  fast with an actionable error naming the missing file(s) if
  `kernel`/`disk.raw` aren't present, instead of spawning
  `tinybridge-vmhost` with paths that don't exist. Live-verified: the real
  daemon code path now reaches a real `Running` VZ state in ~1s with real
  boot assets (see README's "vs Lima" section for the full account).
- `up()`'s generated SSH config entry hardcoded `hostname: "192.168.105.2"`
  unconditionally, ignoring the real resolved guest IP computed a few
  lines earlier in the same function - silently pointed SSH at the wrong
  host whenever they differed. Now only writes an entry once a real IP is
  known (`EnvironmentManager::build_ssh_entry`, unit-tested); skips
  writing one at all rather than guessing.

## [0.6.1] - 2026-09-26

### Added
- `.github/dependabot.yml` (cargo, swift, github-actions ecosystems).
- `security-audit` CI job running `cargo audit` (advisory-only for now — see
  `docs/internal/ROADMAP_HONEST.md` §4 for why it isn't a hard gate yet).
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `docs/internal/ROADMAP_HONEST.md`, this `CHANGELOG.md`,
  `.github/ISSUE_TEMPLATE/`, `.github/pull_request_template.md`, `docs/README.md` index.

### Changed
- Corrected stale test-count claims in `README.md` (previously "185/194" for
  `tinybridge-daemon`, actually 194/194; previously "396/396" for the rest of the
  workspace, actually 408/408 after re-verification).
- `README.md`'s reference to `docs/ARCHITECTURE.md` now points at `docs/README.md` and
  explains that file was archived rather than kept as current (it described a SwiftUI
  menu-bar app and notarized `.dmg` installer that don't exist as a shipped product).
- `SECURITY.md` no longer claims a "Security Team," a 24-hour acknowledgment SLA, or generic
  unverifiable "best practices" bullets — this is a one-person project and is now described
  as such.
- `.gitignore`: removed the bare `Cargo.lock` ignore rule (this workspace ships binaries;
  the lockfile is, and should stay, committed) and added an explanatory comment.

### Fixed
- `crates/tinybridge-core/src/macos_adapter.rs`, `windows_adapter.rs`, `linux_adapter.rs`,
  `platform_registry.rs`: every `RwLock::read()`/`write()` call used `.unwrap()`, so a single
  panic anywhere while holding the lock would poison it and make every subsequent VM-metadata
  lookup/mutation (or adapter registration/lookup) on that adapter panic too. Switched to
  `.unwrap_or_else(|poisoned| poisoned.into_inner())` to recover the guard instead — the
  underlying `HashMap` is never left torn by any operation on these types, so recovering a
  poisoned guard is safe. Added `test_survives_poisoned_lock` to each of the four files,
  proving a call after a simulated panic-while-holding-the-lock now succeeds instead of
  panicking. (These modules are dead/unwired scaffolding per `docs/internal/ROADMAP_HONEST.md` §2-3 — not
  reachable from the daemon/CLI/RPC path — so this closes a real bug pattern without touching
  any live code path.)

### Removed
- `docs/CLAUDE.md` and `docs/PRODUCT_VISION.md` — stated `License: Proprietary` /
  `Repository: Private`, both false post-relicense/going-public, and fully superseded by
  the root `README.md`/`PRODUCT_VISION.md`.
- `.github/INSTALL.md` — described a `pip install tinybridge` / Python packaging flow that
  has never applied to this Rust/Swift project.
- `.github/CI_ERRORS.md` — generic unfilled template referencing a nonexistent report file.
- `.github/workflows/test-dispatch.yml` — no-op workflow (`echo` only).

### Moved
- ~34 dated planning/status/vision docs (phase reports, week-progress notes, session
  summaries, and docs describing an unbuilt macOS GUI app or fabricated compliance
  certifications) moved from `docs/` to `docs/archive/`, with specifics on what's wrong with
  each in `docs/archive/README.md`. See `docs/internal/ROADMAP_HONEST.md` §3 for the full accounting.

## [0.6.0] - 2026-08-30
## [0.5.1] - 2026-08-23
## [0.5.0] - 2026-08-12
## [0.4.0] - 2026-08-06
## [0.3.1] - 2026-07-27
## [0.3.0] - 2026-07-26
## [0.2.0] - 2026-07-25
## [0.1.0] - 2026-07-22

Pre-changelog releases. Dates are from git tag history; detailed per-release notes were not
kept and are not reconstructed here to avoid fabricating history — see git log for the real
commits in each range.
