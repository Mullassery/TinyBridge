# Changelog

All notable changes to this project are documented in this file. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Entries below `[Unreleased]` are reconstructed only where confidently verifiable (git tag
dates); this project did not keep a changelog before this pass, so no per-release feature
lists are fabricated for the historical tags. See `git log <tag>..<tag>` or
[GitHub Releases](https://github.com/Mullassery/TinyBridge/releases) for the real commit
history of each version.

## [Unreleased]

## [0.6.1] - 2026-09-26

### Added
- `.github/dependabot.yml` (cargo, swift, github-actions ecosystems).
- `security-audit` CI job running `cargo audit` (advisory-only for now — see
  `ROADMAP_HONEST.md` §4 for why it isn't a hard gate yet).
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `ROADMAP_HONEST.md`, this `CHANGELOG.md`,
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
  panicking. (These modules are dead/unwired scaffolding per `ROADMAP_HONEST.md` §2-3 — not
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
  each in `docs/archive/README.md`. See `ROADMAP_HONEST.md` §3 for the full accounting.

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
