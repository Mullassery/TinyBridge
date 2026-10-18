# Guest assets

TinyBridge boots a real kernel/initrd/disk. Asset download is explicit and
SHA-256 verified:

```bash
tinybridge assets fetch --manifest assets/ubuntu-24.04-arm64.json
```

This writes `~/.tinybridge/assets/vmlinux` (or `--dest DIR`). The checked-in
manifest pins the Firecracker CI arm64 kernel used by the build guide:

- `vmlinux`: 15.4 MB, SHA-256
  `eb5d95ac8a67f7a86acf0cb35625633713ad5170b56de8617808d0e18bb832ec`

Still manual after this:

1. A root filesystem image (`disk.raw`/rootfs) — build it with
   `scripts/build-rootfs-multi-tier.sh` or use an Ubuntu cloud image.
2. An initrd if your kernel/rootfs pair requires one.
3. A cloud-init `seed.iso` for credentials.

The manifest format is intentionally simple so maintainers can pin additional
assets without changing the CLI:

```json
{"assets":[{"name":"vmlinux","url":"https://...","sha256":"..."}]}
```

`assets fetch` rejects path-traversal names, partial downloads, and checksum
mismatches before moving a file into place.
