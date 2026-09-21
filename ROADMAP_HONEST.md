# Honest Roadmap & Technical Debt

This file tracks what actually works, what's dead/orphaned, and what's real technical debt,
verified directly against the code and CI as of this pass (2026-09-21). It follows this
project's non-negotiable honesty rule: no "planned"/"may be added" hedging for things that
are simply broken, unwired, or don't exist — those are named as such. For day-to-day status
(platform support, install path, "what's actually been verified") see the root
[README.md](README.md) — it is kept more current than this file for that. This file exists
for the things README doesn't already cover in depth: dead code, dependency debt, and a
concrete follow-up punch list.

## 1. Works and verified

- **macOS Virtualization.framework VM lifecycle** (start/stop/status over Unix-socket
  JSON-RPC, per-VM `tinybridge-vmhost` process) — see README's "What's actually been
  verified" section. Verified by direct observation, not just by tests passing.
- **Guest boot to a real login prompt** with cloud-init credentials — verified 2026-08-28,
  see README.
- **Build/lint/test gates**, re-verified directly during this pass:
  - `swift build --package-path swift/ -c release` — succeeds.
  - `cargo build --workspace` — succeeds (one pre-existing, documented warning: `block
    v0.1.6` future-incompatibility, see §3).
  - `cargo fmt --check` — clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` — clean, 0 warnings.
  - `cargo test --workspace --exclude tinybridge-daemon` (with `DYLD_LIBRARY_PATH` set to
    an **absolute** path — a relative path silently fails to load the dylib for `cargo
    test`, though not for `build`/`clippy`) — **408 passed, 0 failed**.
  - `cargo test -p tinybridge-daemon` — **194 passed, 0 failed**. (README previously said
    185/194 with 9 pre-existing skipped failures; that's now stale — verified directly here
    that all 194 pass and CI no longer excludes any of them. README has been corrected in
    this pass.)
- **DDS networking CLI** (`tinybridge dds`) — real, wired end-to-end:
  `crates/tinybridge-cli/src/commands/dds.rs` → `crates/tinybridge-daemon/src/dds_rpc.rs` /
  `daemon.rs`.
- **Clipboard bridge** (`crates/tinybridge-clipboard`) — wired into the daemon
  (`crates/tinybridge-daemon/src/clipboard_sync.rs`), but depends on host→guest
  connectivity, which is currently blocked (see README's "Remaining known gap") — so treat
  this as wired-but-currently-inoperable rather than end-to-end verified.

## 2. Partially works / disclosed gaps (already covered in README, cross-referenced here)

- Windows/Linux hypervisor backends: unimplemented scaffolding (README, "Honest status").
- VirtioFS host-directory sharing: scoping logic implemented and tested; the actual FFI
  wiring is not (README, "Known debt").
- Host→guest SSH/TCP: blocked on a one-time macOS "Local Network" permission grant, not a
  code bug (README, "Remaining known gap").
- Homebrew distribution: tap is private, `tinybridge-vmhost` in the release tarball has no
  `LC_RPATH` (README, "Installing" / "Known debt", issues #1/#2).
- Two CLI `TODO`s (verified still present, unchanged from README's own count):
  `crates/tinybridge-cli/src/commands/logs.rs:46` and
  `crates/tinybridge-cli/src/commands/launch.rs:134`.

## 3. Documented-but-dead code and fabricated docs found and fixed in this pass

These were found by cross-checking every doc under `docs/` against `grep`-verified usage in
the actual crates. All of the docs below have been moved to `docs/archive/` (with specifics
in `docs/archive/README.md`) rather than left as if current:

- **`crates/tinybridge-macos/`** — a SwiftUI menu-bar app prototype (757-line `main.swift`,
  its own `Package.swift`). It is **not a Cargo crate** despite living under `crates/` (no
  `Cargo.toml`), **not a workspace member**, **not built by CI**, and talks to the daemon
  over a hardcoded TCP port 7890 while the real daemon speaks Unix-socket JSON-RPC — it
  could not connect to the real daemon even if built. Nine docs described this (or the
  `.dmg`/notarized-installer/Homebrew-cask distribution built around it) as if it were
  current or shipping: `ARCHITECTURE.md`, `GETTING_STARTED.md`, `USER_README.md`,
  `DMG_PACKAGING.md`, `HOMEBREW.md`, `HOMEBREW_TAP_SETUP.md`, `MACOS_BUILD_GUIDE.md`,
  `MACOS_MENU_BAR_GUIDE.md`, `MACOS_UX_RESEARCH.md`. **Follow-up decision needed:** either
  finish and wire this prototype (real work: move it into the Cargo workspace or a
  documented separate build step, fix the transport mismatch, add it to CI) or delete it —
  leaving it as unbuilt, unlisted, disconnected code invites exactly the confusion this pass
  found.
- **`crates/tinybridge-devices`** — a real, compiling workspace member (~1,300 lines,
  `cargo build --workspace` includes it) implementing hardware-passthrough policy. It is
  **never referenced by the daemon or CLI** (`grep -r tinybridge_devices crates/` outside
  its own directory returns nothing; there is no `tinybridge devices` command in
  `crates/tinybridge-cli/src/commands/`). Docs (`DEVICE_POLICY_GOVERNANCE.md`,
  `GRANULAR_DEVICE_MANAGEMENT.md`, now archived) claimed "Security Level: Enterprise-Grade"
  / "Compliance Ready: Yes" for this — false; it's dead code with no consumer.
- **`crates/tinybridge-daemon/src/ip_monitor.rs`** (465 lines) — `mod ip_monitor;` is
  declared in `main.rs:17`, but `IpMonitor::new()` is called **only from the module's own
  `#[cfg(test)]` block** (lines ~404, 422, 442, 456) — it is never instantiated by the
  running daemon and never wired to any RPC handler. `detect_network_path()` (line 292) and
  `record_security_event()` (line 302) are plain setters with no actual VPN/firewall
  detection or anomaly-detection algorithm behind them, despite `IP_MONITORING.md` (now
  archived) describing this as a "production-grade," "enterprise-grade" system with
  OpenTelemetry export and compliance/forensics use cases.
- **Fabricated compliance claim**: `docs/DDS_OPT_IN_DESIGN.md` (now archived) claimed
  "Compliance Ready — Satisfies SOC 2 / ISO 27001 / PCI-DSS." No compliance audit,
  certification, or SOC 2/ISO/PCI-DSS work has ever been done on this project. This was the
  single most serious fabrication found in the repo and must never be reintroduced.
- **Stale license/visibility claims**: `docs/CLAUDE.md` and `docs/PRODUCT_VISION.md` (both
  deleted, not archived — superseded by the root README and root `PRODUCT_VISION.md` with
  no additional historical value) stated `License: Proprietary` and `Repository: Private`,
  both false since the Apache-2.0 relicense and this repo going public.
- **~25 other dated planning/status docs** (`WEEK_*`, `SESSION_SUMMARY*`, `PHASE_*`,
  `IMPLEMENTATION_*`, `CRITICAL_GAPS_ANALYSIS.md`, `TESTING_REPORT.md`,
  `TINYBRIDGE_2_0_INTEGRATION_ROADMAP.md`, `COMPETITIVE_ANALYSIS_ORBSTACK.md`,
  `COMPETITIVE_POSITIONING.md`, etc.) were moved to `docs/archive/` as pre-honesty-pass
  planning artifacts, not individually fact-checked line by line — assume anything in them
  may be stale or aspirational.
- **`.github/INSTALL.md`** (deleted) — described `pip install tinybridge`, Python 3.10+
  requirements, and a `uv tool install` flow. This is a Rust/Swift project with no PyPI
  package at all; this file was boilerplate from an unrelated template, not edited for this
  project.
- **`.github/CI_ERRORS.md`** (deleted) — a generic template referencing a
  `../../CI_ERRORS_REPORT.md` file that does not exist in this repo and a `$REPO_NAME`
  placeholder never filled in.
- **`.github/workflows/test-dispatch.yml`** (deleted) — a workflow whose entire body was
  `echo "Workflow dispatch triggered"`; no functional purpose.

## 4. Real, unfixed technical debt (needs a dedicated follow-up session)

- **Vulnerable/unsound dependencies** (`cargo audit`, run directly during this pass):
  - `rsa 0.9.10` (pulled in transitively via the `ssh-key` dependency used by
    `tinybridge-ssh`) — **RUSTSEC-2023-0071**, "Marvin Attack" timing side-channel, medium
    severity (5.9). **No fixed version exists upstream** as of this pass; this can only be
    resolved by `ssh-key` (or its own dependency) shipping a fix, or by
    `tinybridge-ssh` dropping RSA key support / switching away from `ssh-key`. Not
    something this pass can safely fix.
  - `lru 0.12.5` (direct dependency of `tinybridge-router`, pinned via the workspace
    `Cargo.toml`) — **two active soundness advisories**: RUSTSEC-2026-0253 (use-after-free
    in `LruCache::pop()` under panic) and RUSTSEC-2026-0002 (`IterMut` violates Stacked
    Borrows). Latest available is `0.18.4` — a 6-minor-version jump likely to have breaking
    API changes in `tinybridge-router`; needs an actual upgrade-and-retest pass, not a blind
    version bump.
  - `number_prefix 0.4.0` (transitive, via `indicatif`) — flagged unmaintained
    (RUSTSEC-2025-0119). Low severity, no known vulnerability, just unmaintained.
  - A `security-audit` CI job (`cargo audit`) has been added in this pass
    (`.github/workflows/ci.yml`) but is **advisory-only** (`|| echo ...`, does not fail the
    build) specifically because of the `rsa` finding above having no available fix yet.
    Flip it to a hard failure once these are triaged/resolved or explicitly allow-listed via
    `cargo audit --ignore RUSTSEC-2023-0071` with a comment explaining why.
- **Blanket warning suppression hides dead code from the compiler**:
  `crates/tinybridge-daemon/src/main.rs:1` has
  `#![allow(unused_imports, unused_variables, unexpected_cfgs, dead_code)]` at the crate
  root. This is very likely *why* `ip_monitor.rs`'s dead code (see §3) was never caught by
  a compiler warning — `dead_code` lints are disabled for the entire binary. Recommend
  narrowing this to per-item `#[allow(dead_code)]` with a reason comment (as is already done
  correctly in a few other files, e.g. `crates/tinybridge-daemon/src/vz.rs:14`) so real dead
  code shows up again.
- **`crates/tinybridge-daemon/src/okf_updater.rs`** has 12 separate `#[allow(dead_code)]`
  annotations across a single file — a large fraction of its public surface is explicitly
  marked unused rather than removed or wired in. Worth an audit of whether "OKF" (OpenTelemetry
  Key Facts / quality-gate status calculation, per git history) is actually consumed anywhere
  or is itself another orphaned subsystem like `ip_monitor.rs`.
- **~250 `.unwrap()` calls** across `crates/*/src/*.rs` (rough count via grep, top-level
  files only, not recursive into submodules — the real number including submodules is
  higher). Not inherently wrong for a CLI tool, but worth an audit for the paths that handle
  untrusted input (guest disk images, RPC payloads, CLI args) vs. paths where a panic is an
  acceptable "this should never happen" invariant.
- **`homebrew-release/` directory has 9 stale binary release tarballs committed directly to
  git** (`tinybridge-0.3.0-*.tar.gz`, `tinybridge-vmhost-0.3.0-*.tar.gz`,
  `tinybridged-0.3.0-*.tar.gz`, and their `0.4.0` equivalents — ~9.7 MB), superseded by the
  current `v0.6.0`. This directly contradicts the project's own stated convention
  (`.gitignore`'s `/release/` entry: "Release build output ... uploaded to GitHub Releases,
  not committed"). Removing them now only stops the tree from growing further — the bytes
  are already in git history and would need a history rewrite (`git filter-repo`/BFG) to
  actually reclaim space, which is disruptive to any existing clones/forks and was not
  attempted in this pass. **Recommend**: stop committing new release tarballs here, remove
  the stale ones from the working tree in a dedicated follow-up, and decide separately
  whether a history rewrite is worth the disruption.
- **`docs/BUILD_ASSETS_GUIDE.md`'s Firecracker/cloud-hypervisor release URLs are 404**
  (already noted by README itself, re-surfaced here for visibility): the kernel URLs this
  doc references no longer resolve. README's own "actual fix" section documents the
  real, working alternative (extract kernel/initrd from the cloud image's own `/boot`), so
  this doc is stale relative to README, not just relative to upstream.
- **`.github/workflows/assets.yml`**: shellcheck (via `actionlint`) flags 3 `SC2086`
  (unquoted variable expansion) info-level issues around line 69 (`ls -lh assets/vmlinux*`
  style globs). Cosmetic, not a functional bug, left as-is; low priority.
- **`docs/CLIPBOARD.md` and `docs/SSH_EXPERIENCE.md`** (kept, not archived) both describe
  frictionless SSH/clipboard flows that depend on host→guest connectivity, which README
  documents as currently broken pending a manual macOS permission grant. Neither doc has
  been independently verified end-to-end in this pass; treat both as design references, not
  proof the described flow currently works.

## 5. Not built at all

- Any macOS GUI application that's part of the shipped product (the `tinybridge-macos`
  prototype in §3 is unbuilt, unwired scaffolding, not a shipped feature).
- ROS 2 DDS, "transparent CUDA routing," remote-GPU integration (RunPod etc.), and any
  compliance certification — all were described in archived docs as if real or imminent;
  none exist in the codebase in any form beyond the `tinybridge-dds` CLI plumbing described
  in §1 (which does real DDS *environment configuration*, not ROS 2-specific behavior).
- A tested, bundled guest kernel+rootfs pipeline (README, "Known debt").
