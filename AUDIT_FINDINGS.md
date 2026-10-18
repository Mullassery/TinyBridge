# Audit findings — 2026-10-19

This file records the repository-specific findings from the portfolio audit
conducted on 2026-10-19. It is documentation only; no code or release artifacts
were changed in this pass unless explicitly noted under "Implementation status".

Sources: the repository's own README/source/workflows, GitHub metadata and
Actions runs, PyPI metadata, and official competitor documentation. Claims are
marked as verified, inferred, or open questions. "Absent" means not found in the
inspected repository/docs, not proof that it cannot exist.

## What it is

macOS-native Linux VM runtime on Apple Virtualization.framework: Rust→C→Swift bridge, per-VM vmhost, daemon, CLI, cloud-init, VirtioFS, snapshots.

## Verified strengths

- Real VZ boot verified to a login prompt; per-VM codesigned vmhost; VirtioFS path scoping; 0600 daemon socket; cloud-init seed; 408 + 194 tests; CI green; honest status docs; release tarball contains CLI/daemon/vmhost/dylib/checksums.
- Genuine Apple-VZ-native, scriptable, no-GUI substrate.

## Verified gaps and user-facing risks

- README says the Homebrew tap is private and the project is not installable via brew — the tap is public and points at v0.6.3.
- `tinybridged` formula is pinned to v0.6.1 while `tinybridge` is v0.6.3 (CLI/daemon skew); README version is 0.6.1.
- No image download/provisioning pipeline; no host→guest port forwarding; SSH blocked by macOS Local Network consent on the verification machine.
- Only arm64 artifact despite 'Intel Macs' in the target-user text; no independent security review.

## Direct competitors

- Lima (auto image, file sharing, port forwarding, templates), OrbStack (Docker Desktop alternative, ~1-minute Linux machines), Multipass (cloud-init/snapshots/mounts), UTM (QEMU+VZ+utmctl), Apple container (macOS 26, OCI images).

## Ranked next actions

- P1: one-command pinned image download + cloud-init + port forwarding (M).
- P1: fix README/tap/version skew (S).
- P2: publish Intel build or explicit unsupported messaging (M).

## Do not build

- GUI, Docker Desktop parity, Kubernetes, Windows/Linux backends now; Lima/OrbStack own those.

## Evidence gaps / validation needed

- Boot verified on one Apple Silicon machine/macOS 26; no time-to-boot/memory benchmark; no independent security review.

## Recommended disposition

Invest; it is a flagship, but the install experience must match the engineering.

## Implementation status

Added local SSH port forwarding in this pass:

- `tinybridge forward <env> --local-port 8080 --remote-port 80` builds an
  `ssh -N -L` command using the managed key; foreground, Ctrl-C to stop.
- Unit tests cover the argument builder and extra SSH options.
- README now reflects the public Homebrew tap and workspace version `0.7.0`.

Still open: automatic guest image/asset provisioning, and host→guest SSH
requires the macOS Local Network permission on the verification machine.
