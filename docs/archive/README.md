# Archive — historical, not current

Everything in this folder predates the root [README.md](../../README.md)'s honesty pass
(see that file's "Honest status" section). They are kept for historical record, not because
they describe the current product. Do not use them as documentation of what TinyBridge does
today.

Specific, verified problems with files in this folder (checked against the actual code in
this repo as of this pass):

- **`ARCHITECTURE.md`, `GETTING_STARTED.md`, `USER_README.md`, `DMG_PACKAGING.md`,
  `HOMEBREW.md`, `HOMEBREW_TAP_SETUP.md`, `MACOS_BUILD_GUIDE.md`, `MACOS_MENU_BAR_GUIDE.md`,
  `MACOS_UX_RESEARCH.md`** — describe a `TinyBridge.app` SwiftUI menu-bar application,
  `.dmg` installer with Apple notarization, and `brew install --cask tinybridge` /
  `uv tool install tinybridge` installation flows. None of this exists as a working product:
  there is a menu-bar app *prototype* at `crates/tinybridge-macos/` (Swift, `main.swift`,
  757 lines), but it is not a Cargo crate (no `Cargo.toml`), is not listed in the workspace
  `Cargo.toml`, is not built by CI, and talks to the daemon over a hardcoded TCP port 7890 —
  the real daemon uses a Unix-socket JSON-RPC protocol (see root README's Architecture
  section), so this prototype cannot currently connect to the real daemon even if built by
  hand. `ARCHITECTURE.md` additionally still contains unfilled placeholder text
  (`GitHub: github.com/yourusername/tinybridge (private)`) and a claim of a `tinybridge`
  crates.io crate that was never published.
- **`DEVICE_POLICY_GOVERNANCE.md`** — claims `Security Level: Enterprise-Grade` /
  `Compliance Ready: Yes` hardware-passthrough governance. The `tinybridge-devices` crate it
  describes is a real, compiling workspace member (~1,300 lines) but is not referenced by
  the daemon or CLI anywhere (`grep -r tinybridge_devices crates/` outside its own crate
  returns nothing) — there is no `tinybridge devices` command, and nothing wires it into a
  running VM. It is dead/orphaned code.
- **`GRANULAR_DEVICE_MANAGEMENT.md`** — a design plan for the same orphaned devices crate;
  self-labeled "Design & Implementation Plan," which is more honest than the file above, but
  it's still a plan for a feature with no current integration.
- **`DDS_OPT_IN_DESIGN.md`** — claims `Compliance Ready — Satisfies SOC 2 / ISO 27001 /
  PCI-DSS`. No such compliance work, audit, or certification exists for this project at any
  point in its history. This is the most serious fabrication found in this repo's docs and
  should never be reintroduced. (The underlying DDS *feature* is real and wired — see
  `../DDS_CLI_REFERENCE.md` — it's specifically this file's compliance-certification claims
  that are fabricated.)
- **`IP_MONITORING.md`** — describes a "production-grade," "enterprise-grade" IP monitoring
  system with VPN/firewall detection and OpenTelemetry export. The backing code
  (`crates/tinybridge-daemon/src/ip_monitor.rs`, 465 lines) is real but is dead code: `mod
  ip_monitor;` is declared in `main.rs`, but `IpMonitor::new()` is called only from the
  module's own unit tests — it is never instantiated by the running daemon, never wired to
  any RPC handler, and `detect_network_path`/`record_security_event` are plain setters with
  no actual VPN/firewall/anomaly-detection logic behind them.
- **`CLAUDE.md`, `PRODUCT_VISION.md`** (already deleted, not moved here) — an internal
  agent-context file and a duplicate product-vision draft that both stated `License:
  Proprietary` / `Repository: Private`, which have been false since the Apache-2.0 relicense
  and this repo going public. Superseded entirely by the root README and root
  `PRODUCT_VISION.md`; removed rather than archived since they added no historical value
  beyond what git history already preserves.
- Everything else here (`WEEK_*`, `SESSION_SUMMARY*`, `PHASE_*`, `IMPLEMENTATION_*`,
  `HOMEBREW_FIX_COMPLETE.md`, `HOMEBREW_PUBLISHED.md`, `CRITICAL_GAPS_ANALYSIS.md`,
  `TESTING_REPORT.md`, `TINYBRIDGE_2_0_INTEGRATION_ROADMAP.md`,
  `ENHANCED_IP_MONITORING_SUMMARY.md`, `DDS_IMPLEMENTATION_SUMMARY.md`,
  `COMPETITIVE_ANALYSIS_ORBSTACK.md`, `COMPETITIVE_POSITIONING.md`, `okr_phase_3_week_1.md`,
  `README_REWRITE_SUMMARY.md`, `GITHUB_ABOUT.md`) are dated progress logs, phase-status
  reports, and planning docs from before the honesty pass. Not individually fact-checked
  line by line; assume anything in them may be stale, aspirational, or contradicted by the
  root README.
