# Queue status — 2026-10-19

This file records what was shipped, what is blocked/failed, and what is rejected for this repository. It complements `AUDIT_FINDINGS.md`.

## Shipped
- Local SSH port forwarding (`tinybridge forward`).
- SHA-256-verified `tinybridge assets fetch` with a pinned arm64 kernel manifest.

## Blocked / failed
- Automatic rootfs/disk conversion requires Linux mkfs/mount tooling; macOS cannot create ext4.
- Intel x86_64 build/artifact.
- Guest SSH depends on the macOS Local Network permission on the test machine.

## Rejected / deferred
- GUI and Docker Desktop parity — Lima/OrbStack own those workflows.
- Windows/Linux hypervisor backends now — cannot be developed/verified on the macOS host.

## Policy

Any future feature or upgrade must check `AUDIT_FINDINGS.md` and this file before implementation. A blocked item is retried only when its external dependency is available; a rejected item needs new user evidence or a design change to reopen.
