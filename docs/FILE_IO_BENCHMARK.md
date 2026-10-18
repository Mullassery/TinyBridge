# File-sharing (VirtioFS / 9p) throughput benchmark

TinyBridge shares host folders into guests via **VirtioFS** on macOS (Apple
Virtualization.framework) and **Virtio-9p** on Linux (libvirt). This document
explains how to measure file-sharing throughput honestly, because the numbers
that used to be quoted here were **targets, not measurements** (RECHECK-062).

> **Current status of the claims.** The "~90% native file I/O" and "1.5s boot"
> figures in `docs/archive/TESTING_REPORT.md` are aspirational targets. That
> document says so itself, but the numbers have a habit of being quoted without
> the caveat. Until someone runs the harness below on real hardware and records
> the result, **do not treat them as measured performance.**

## Linux 9p is a stub

The Linux adapter's `mount_storage` records the share in VM metadata but does
**not** modify the libvirt domain XML, so the folder does not actually appear in
the guest. It now prints an explicit `WARNING:` when called (previously it
logged a reassuring "Mounting … via 9p" line, which was misleading). The
benchmark exits with an error on such a guest rather than reporting a number for
a share that isn't mounted.

## Running it

Prerequisites: a running TinyBridge guest with a shared folder mounted.

```sh
# macOS: start a VM with a share (see README), then:
scripts/bench-fileio.sh /mnt/share 256

# Override the ssh target / result directory if needed:
GUEST_HOST=vm@192.168.64.2 RESULTS_DIR=/tmp/bench scripts/bench-fileio.sh /mnt/share 512
```

The harness runs `dd` (1 MiB blocks, `conv=fdatasync`) inside the guest for two
cases:

| case | what it measures |
|---|---|
| `guest-disk (control)` | the guest's own virtual disk (the "native" baseline) |
| `shared-folder` | the VirtioFS/9p mount |

For each it reports write and read throughput and writes a JSON result to
`benchmark-results/fileio_<timestamp>.json`. **The ratio of `shared-folder` to
`guest-disk` is the honest "% of native" number** — compare it against Lima's
published 80–90%, on the same host, with the same file size.

## What to do with the result

- Record measured numbers in the README/CHANGELOG with the host model, guest
  kernel, file size, and date — not just a percentage.
- If the share is materially slower than the control, that's a real finding;
  file it rather than adjusting the target.
- On Linux, there is nothing to measure until 9p is implemented.
