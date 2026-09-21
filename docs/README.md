# Documentation Index

**The root [README.md](../README.md) is the single source of truth for current project
status.** It is kept up to date with what has actually been built, tested, and verified,
including an explicit "Honest status" section and a "Known debt" list. If anything below
disagrees with it, trust the root README.

## Current reference docs

These describe real, wired code as of the last time each was reviewed. They are technical
references, not status claims — always cross-check against the root README for current
status:

- [BUILD_ASSETS_GUIDE.md](BUILD_ASSETS_GUIDE.md) — building guest kernel/rootfs assets;
  self-labeled "skeleton assets created, production assets require external access."
- [CLIPBOARD.md](CLIPBOARD.md) — macOS↔Linux clipboard bridge design
  (`crates/tinybridge-clipboard`, wired into the daemon via
  `crates/tinybridge-daemon/src/clipboard_sync.rs`). Note: this depends on host→guest SSH,
  which the root README documents as currently blocked by a local macOS permission gate —
  treat clipboard sync as unverified end-to-end until that's resolved.
- [DDS_CLI_REFERENCE.md](DDS_CLI_REFERENCE.md) — `tinybridge dds` command reference (the
  real, wired command; see `crates/tinybridge-cli/src/commands/dds.rs` and
  `crates/tinybridge-daemon/src/dds_rpc.rs`).
- [GIT_DEPLOYMENT_GUIDE.md](GIT_DEPLOYMENT_GUIDE.md) — general advice on using `env.yaml`
  with git branching workflows. Conceptual guidance, not a description of a built feature.
- [SSH_EXPERIENCE.md](SSH_EXPERIENCE.md) — SSH UX design philosophy/goals. Labeled as an
  "ideal flow"; the root README documents that host→guest connectivity is not currently
  working out of the box (blocked on a macOS Local Network permission grant), so treat the
  frictionless flow described here as aspirational, not current behavior.
- [SWIFT_VZ_BUILD.md](SWIFT_VZ_BUILD.md) — building the real Swift Virtualization.framework
  bridge (`swift/Sources/TinyBridgeVZBridge`), which the root README confirms is real and
  wired.

## Archive

[archive/](archive/) contains dated planning docs, phase/week status reports, session
summaries, and product-vision drafts written before the root README's honesty pass. Several
describe features, a macOS menu-bar GUI app, compliance certifications, and distribution
mechanisms that do not exist in this codebase (see `archive/README.md` for specifics). Kept
for historical record only — do not treat anything in that folder as current or accurate.
