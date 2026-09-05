# TinyBridge

A macOS-native Linux VM runtime, built on Apple's Virtualization.framework. Boots a real
Linux virtual machine via a genuine Rust -> C ABI -> Swift -> Virtualization.framework call
chain - not a mock.

## Honest status (read this first)

This project was previously documented and committed as "production-grade" and
"cross-platform (Windows/macOS/Linux)." That was not accurate. Here is the real state:

- **macOS is the only platform with a working hypervisor backend.** `crates/tinybridge-vz`
  calls Apple's Virtualization.framework through a Swift bridge
  (`swift/Sources/TinyBridgeVZBridge`), and it is wired end-to-end through
  `crates/tinybridge-vmhost`'s `VmController` and `crates/tinybridge-daemon`. This has been
  verified to actually boot a real ARM64 Linux kernel under `VZVirtualMachine` on real Apple
  Silicon hardware (state transitions `Stopped -> Running`, with a NAT guest IP detected) -
  see "What's actually been verified" below.
- **Windows (Hyper-V) and Linux (KVM/QEMU) have no real hypervisor backend.**
  `crates/tinybridge-core/src/windows_adapter.rs` and `linux_adapter.rs` only mutate an
  in-memory `HashMap` - there is no Hyper-V or KVM API call anywhere in either file, and
  neither is wired into the daemon, CLI, or any RPC path (dead code, kept as clearly-labeled
  scaffolding for a genuine future implementation). Building and testing real Windows/Linux
  hypervisor backends isn't possible from a macOS development environment, so this is
  correctly deferred rather than faked further.
- The CLI's own `--help` text already says it plainly: `tinybridge --help` describes this as
  a **"macOS Linux development substrate."**

If you're evaluating this project: treat it as a real, working macOS-only VM runtime with
a genuine (if young) Virtualization.framework integration, not a finished cross-platform
product.

## What's actually been verified

Directly observed on Apple Silicon (M-series, macOS 26), not just implemented and assumed
to work:

1. The Swift bridge (`swift/Sources/TinyBridgeVZBridge`) builds cleanly against
   Virtualization.framework (`swift build -c release`) and exports the real C ABI symbols
   (`tb_vm_create`, `tb_vm_start`, `tb_vm_stop`, `tb_vm_get_status`, ...).
2. `tinybridge-vz-sys`'s `bindgen`-generated Rust bindings compile and link against that
   dylib for real.
3. `tinybridge-vmhost`'s real production binary - codesigned with the
   `com.apple.security.virtualization` entitlement
   (`crates/tinybridge-vmhost/tinybridge-vmhost.entitlements`) - was started, and driven over
   its actual Unix-socket JSON-RPC protocol (`vmhost.start`/`vmhost.status`/`vmhost.stop`)
   with a real ARM64 Linux kernel image. Observed status transitions:
   `Stopped` → (`vmhost.start`) → `Running` (with a detected NAT guest IP,
   `192.168.105.2`) → (`vmhost.stop`) → `Stopped`.
4. A non-obvious platform requirement was found and fixed in the process:
   Virtualization.framework dispatches its callbacks onto the process's main GCD queue, so a
   plain `#[tokio::main]` process silently hangs forever on every VM lifecycle call. Fixed by
   running the async server on a background thread and dedicating the real process main
   thread to `dispatch_main()` (see `crates/tinybridge-vmhost/src/main.rs`).
5. Ad-hoc codesigning with the virtualization entitlement (no paid Apple Developer account
   required) was confirmed sufficient for local use - see `justfile`'s `sign-vmhost` recipe.

**RESOLVED 2026-08-28: guest boot to a real login prompt, with real credentials, now works.**
The root cause of every prior failure below was never an unfixable Apple regression - it was
using the wrong kernel. Kept the failed attempts below for a complete record, then see "The
actual fix" underneath for what resolved it.

**Prior failed attempts** (kept for a complete record - a real guest disk image was used
throughout, not a placeholder):

- A real Ubuntu 24.04 ARM64 cloud image was downloaded, checksum-verified, and converted from
  QCOW2 to the raw format `VZDiskImageStorageDeviceAttachment` requires (`qemu-img convert -O
  raw`), then confirmed GPT-partitioned with a Linux root filesystem as the first partition,
  matching the default `root=/dev/vda1` kernel cmdline.
- A real serial console was wired up end-to-end (`serial_log_path` on `TBVMConfig` ->
  `VZVirtioConsoleDeviceSerialPortConfiguration` + `VZFileHandleSerialPortAttachment` in
  `TinyBridgeVZ.swift` -> `VmConfig::with_serial_log_path()` in `crates/tinybridge-vz`), since
  previously `console=hvc0` in the kernel cmdline pointed at a device that was never attached
  and there was no way to observe boot output at all.
- The VirtIO graphics device was made conditional on a non-zero display size
  (`crates/tinybridge-vz/examples/vz_boot_test.rs` requests `0x0`, i.e. headless), because
  attaching it unconditionally opens a real WindowServer/SkyLight session that macOS gates
  behind Screen Recording TCC consent for the calling process - unnecessary for a serial-only
  boot check.
- With all of the above real and wired, `vm.start()` on this real kernel + real disk still
  fails with `Error Domain=VZErrorDomain Code=1 "The virtual machine failed to start."` before
  any guest code runs (confirmed via `Console.app`/`log show`, with and without the graphics
  device, with and without an app-bundle wrapper, with and without a real vs. placeholder
  disk - same failure every time). This is not a TinyBridge bug: it matches a
  [known macOS 26.x ARM64 Virtualization.framework regression](https://github.com/apple/container/issues/1254)
  that also breaks Apple's own `container` CLI and Podman on the same OS/architecture
  combination, per an Apple-affiliated maintainer's confirmation on that issue - there is
  currently no known app-level workaround. Re-verification is blocked on either an Apple OS
  update or testing on a macOS build that doesn't have this regression.

**Re-verified 2026-08-28 on macOS 26.5.2 (Apple Silicon): a different failure mode, not the
one above.** Repeated the same real-asset setup (Ubuntu 24.04 ARM64 cloud image, checksum
verified, `qemu-img convert -O raw`, confirmed `EFI PART` GPT signature at offset 512; kernel
from Firecracker's CI bucket, `vmlinux-5.10.223`, confirmed `file`-identified as a real ARM64
boot Image — the cloud-hypervisor and firecracker-microvm release URLs `BUILD_ASSETS_GUIDE.md`
documents both now 404, a separate doc-rot issue worth fixing) and ran `vz_boot_test` end to
end:

- `vm.start()` returned `Ok(())` (no `VZErrorDomain` error at all this time), and the VM
  reached `state: Running` with a real NAT-assigned guest IP (`192.168.105.2`) within 5.5s —
  further than the documented Code=1 immediate failure.
- Despite that, the serial console log (`console=hvc0`, wired to a real
  `VZFileHandleSerialPortAttachment`) stayed completely empty (0 bytes) for the full 60-second
  observation window. A kernel that actually executed even one instruction past early boot
  would have written something to `hvc0`; zero bytes after 60s means the guest CPU almost
  certainly never ran real code, despite the hypervisor-level state machine saying `Running`.
- `log show` around VM creation shows a concrete, reproducible AMFI rejection of the Swift
  bridge's dylib that doesn't appear anywhere in the previous investigation:
  `amfid: .../libTinyBridgeVZBridge.dylib not valid: Error Domain=AppleMobileFileIntegrityError
  Code=-423 "The file is adhoc signed or signed by an unknown certificate chain"`. `syspolicyd`
  still ultimately allows the library to load (`GK evaluateScanResult` succeeds, "allowed,
  cache" in the kernel log), so this isn't an outright load failure — but it's a real, distinct
  signal worth investigating as a possible contributor to the silent-no-boot symptom, since
  Virtualization.framework may degrade privileged operations (like serial port wiring) when the
  calling code's identity isn't AMFI-trusted, even with the `com.apple.security.virtualization`
  entitlement present on the outer process. Explicitly re-codesigning the dylib itself
  (`codesign --force --sign - --entitlements ... libTinyBridgeVZBridge.dylib`) did not change
  the outcome — same AMFI complaint, same empty console.
- Net effect: guest boot is still not achievable on this exact machine/OS build, but the
  concrete symptom has changed from "immediate explicit error" to "silent hang with a
  misleadingly optimistic `Running`+IP status," and there's now a specific, logged AMFI
  signature error to chase (Error -423) that wasn't part of the prior investigation. Getting a
  full Developer ID signature (not just ad-hoc) to test whether that resolves the AMFI
  complaint would require a paid Apple Developer account, which the project has otherwise
  avoided needing — worth a decision call before investing in that path.

### The actual fix (2026-08-28, same macOS 26.5.2 machine)

The AMFI signature theory above was a red herring. The real root cause: **the Firecracker CI
kernel used in every attempt above (`vmlinux-5.10.223`) is built for Firecracker's own minimal
microVM hardware model (virtio-mmio transport, no PCI), not Apple's Virtualization.framework
hardware model (which needs virtio-pci)** - so the kernel could never have found its root
disk or console regardless of any code-signing issue. Confirmed by extracting the *same
cloud image's own* matching kernel and initrd (loop-mounting `disk.raw`'s `/boot` partition -
partition 16 in this GPT layout, not partition 1, in a privileged Linux container since macOS
can't mount ext4 natively) and booting with those instead:

- `vmlinuz-6.8.0-138-generic` (gzip-wrapped; `gunzip -c` first to get the real
  `Linux kernel ARM64 boot executable Image` `VZLinuxBootLoader` needs) + its matching
  `initrd.img-6.8.0-138-generic`, both pulled straight out of the cloud image's own `/boot` -
  guaranteed to match (unlike guessing a kernel from an unrelated project's release).
- With this kernel+initrd (`VmConfig::with_initrd`, already present in the API but never
  exercised by `vz_boot_test` until now - the example only accepted 3 args, not 4), the
  console log fills with real boot output: `virtio_blk virtio2: [vda] ... vda: vda1 vda15
  vda16` (the disk is found, GPT partitions correctly enumerated), then a full systemd boot
  (dbus, rsyslog, polkit, snapd, ModemManager, cloud-init, ...) all the way to
  `Ubuntu 24.04.4 LTS ubuntu hvc0 / ubuntu login:` - a real, complete guest boot.
- A raw cloud image with no datasource boots but has no usable login (no password, no SSH
  key - cloud-init has nothing to configure). Added second-disk support
  (`seed_image_path`/`VmConfig::with_seed_image`, threaded through the C header, Swift
  bridge, and Rust FFI) to attach a cloud-init NoCloud seed ISO (`user-data`/`meta-data`,
  volume label `cidata`, built with `hdiutil makehybrid -iso -joliet
  -default-volume-name cidata`) as a second, read-only `VZVirtioBlockDeviceConfiguration`.
  Confirmed in the console log: `Datasource DataSourceNoCloud [seed=/dev/vdb]`, password set,
  SSH host keys generated - a real, working login now exists.
- **Also found and fixed a real, separate bug while validating this**: `tb_vm_get_status`'s
  `ip_address` was **hardcoded** to the literal string `"192.168.105.2"`
  (`self?.ipAddress = self?.ipAddress ?? "192.168.105.2"`, with an honest `// TODO: probe
  guest-side ... once we have a real guest image to test against` comment above it - not
  hidden, but never actually revisited until this session had a real guest to test against).
  Every prior "verified ... with a detected NAT guest IP" claim in this file (including
  earlier in this same investigation) was reading that constant, not a real value. Fixed
  with a real mechanism: `/var/db/dhcpd_leases` (bootpd's real DHCP lease file for
  vmnet-based NAT networking) has one entry per client with a `name=` field taken from the
  guest's own DHCP host-name option. `TBVMConfig` gained a `vm_name` field (the same name
  string already passed to `VirtualMachine::new`); the boot monitor now parses that lease
  file and returns the `ip_address` of the entry whose `name=` matches, or `nil` - never a
  guess - if there's no match yet. Confirmed against the real lease file: `name=tinybridge-vm
  / ip_address=192.168.64.3`, and `vm.status()` now reports exactly that real, resolved
  address instead of the old placeholder. Requires the guest's hostname (e.g. cloud-init's
  `hostname:` seed field) to match the VM's `name` - documented on the header field.

**Remaining known gap**: SSH/TCP/ICMP from the *host* to the guest's real, confirmed IP
(`192.168.64.3`) doesn't get a response, even though the DHCP lease proves the guest is up
and networked (`bridge100`'s ARP table shows the address as unresolved/incomplete). This
looks like macOS's "Local Network" privacy permission gating the *calling* process
(Terminal.app, in this investigation) rather than a TinyBridge bug - `tccutil`/`TCC.db` shows
no local-network grant recorded for it, and that permission can only be granted through a
real System Settings prompt, not scripted. Whoever picks this up next should grant Local
Network access to whatever terminal/app is being used, then retry SSH to the guest's real IP
(read from `vm.status()` or `/var/db/dhcpd_leases`) before assuming there's a code bug here.

## Platform Support

| Platform | Hypervisor | Status |
|---|---|---|
| macOS (Apple Silicon / Intel, macOS 13+) | Apple Virtualization.framework | **Real, wired, verified to boot a complete real guest to a login prompt with working cloud-init credentials** (2026-08-28 - see "The actual fix" above). Host-to-guest SSH is blocked on a local, one-time macOS "Local Network" permission grant, not a code issue. |
| Windows | Hyper-V / WHPX | Not implemented. `windows_adapter.rs` is unimplemented scaffolding, not wired to anything. |
| Linux | KVM/QEMU | Not implemented. `linux_adapter.rs` is unimplemented scaffolding, not wired to anything. |

## Requirements (macOS)

- macOS 13.0+ on Apple Silicon or Intel (Virtualization.framework requirement)
- Rust (see `rust-toolchain.toml`) and Swift (Xcode Command Line Tools are sufficient)
- `just` (optional, for the `justfile` recipes) - or run the equivalent `cargo`/`swift`/
  `codesign` commands directly

## Installing

`brew install mullassery/tinybridge/tinybridge` does not currently work for anyone outside
this project: the tap repository (`mullassery/homebrew-tinybridge`) is **still private**, so
`brew tap`/`brew install` fails with a git authentication error for external users (tracked
in [issue #1](https://github.com/Mullassery/TinyBridge/issues/1) and
[issue #2](https://github.com/Mullassery/TinyBridge/issues/2)). That remains the actual
blocker. The rest of issue #2 has since been fixed and this section is updated to match: the
tap's `tinybridge.rb`/`tinybridged.rb` formulas now point at `v0.5.0` (the latest release,
matching this repo's current `Cargo.toml` version), and the `v0.5.0` GitHub Release asset
(`tinybridge-0.5.0-aarch64-apple-darwin.tar.gz`) now bundles `libTinyBridgeVZBridge.dylib`
and a `SHA256SUMS` file (verified directly - both are present in the downloaded tarball).
`tinybridge-vmhost` inside that tarball still has no `LC_RPATH` (confirmed with `otool -l`:
one `LC_LOAD_DYLIB` for `@rpath/libTinyBridgeVZBridge.dylib`, zero `LC_RPATH` commands, so it
aborts with a dyld error if run standalone) - the bundled `INSTALL.txt` documents the
workaround (set `DYLD_LIBRARY_PATH` to wherever you place the dylib), and the `tinybridged`
formula automates that same workaround via `write_env_script`. So: a manual download of the
release tarball now works if you follow `INSTALL.txt`, but the tap itself still isn't
installable by anyone outside this project until it's made public. Building from source
remains the simplest path for external users today.

## Building from source

```bash
git clone https://github.com/Mullassery/TinyBridge.git
cd TinyBridge

# Builds the Swift Virtualization.framework bridge, copies the dylib where Cargo's linker
# expects it, builds the whole Rust workspace, and codesigns tinybridge-vmhost with the
# com.apple.security.virtualization entitlement (ad-hoc signing - no paid Apple Developer
# account needed).
just build

# Or, without `just`:
swift build --package-path swift/ -c release
mkdir -p target/swift-libs
cp swift/.build/release/libTinyBridgeVZBridge.dylib target/swift-libs/
cargo build --workspace
codesign --force --sign - \
  --entitlements crates/tinybridge-vmhost/tinybridge-vmhost.entitlements \
  target/debug/tinybridge-vmhost
```

`cargo test --workspace` requires `DYLD_LIBRARY_PATH=target/swift-libs` so the test
binaries for `tinybridge-vz` can find the real dylib at runtime; see `.github/workflows/ci.yml`
for the exact invocation this project's CI uses.

## CLI

The real command surface (from `crates/tinybridge-cli/src/main.rs`), not an aspirational
one:

```
tinybridge launch      Launch a new environment
tinybridge up          Start an environment (legacy alias for launch)
tinybridge down        Stop an environment
tinybridge gui         Attach a display window to a running environment
tinybridge headless    Detach the display window (VM keeps running)
tinybridge suspend     Suspend an environment (pause, preserving state)
tinybridge resume      Resume a suspended environment
tinybridge shutdown    Gracefully shut down an environment
tinybridge restart     Restart an environment
tinybridge repair      Re-provision SSH/DDS config for a running environment
tinybridge destroy     Destroy an environment
tinybridge status      Show environment status
tinybridge list        List all environments
tinybridge shell       Open an interactive shell in an environment
tinybridge ssh         SSH into an environment
tinybridge logs        Show environment logs
tinybridge update      Manage environment resources
tinybridge snapshot     Manage environment snapshots
tinybridge doctor      Run system diagnostics
tinybridge templates   List available templates
tinybridge images      List available images
tinybridge dds         Manage DDS networking
```

Run `tinybridge --help` or `tinybridge <command> --help` for full, current usage - that's
the source of truth, not this file.

## Architecture

```
tinybridge-cli  ──(Unix socket JSON-RPC)──>  tinybridged (daemon)
                                                   │
                                          tinybridge-daemon::VmManager
                                          (spawns one child process per VM)
                                                   │
                                     tinybridge-vmhost (per-VM child process)
                                          tinybridge-vmhost::VmController
                                                   │
                                            tinybridge-vz::VirtualMachine
                                                   │
                                          tinybridge-vz-sys (bindgen FFI)
                                                   │
                                   swift/Sources/TinyBridgeVZBridge (Swift, @_cdecl)
                                                   │
                                      Apple Virtualization.framework
```

Each running VM gets its own `tinybridge-vmhost` process (codesigned with the
virtualization entitlement), which owns exactly one real `VZVirtualMachine` and exposes
`start`/`stop`/`force_stop`/`status` over a `0600`-permissioned Unix socket. The daemon
(`tinybridged`) spawns and talks to these per-VM processes; it never touches
Virtualization.framework directly.

See `docs/ARCHITECTURE.md` for more detail (note: some of that document predates this pass
and may still describe the pre-wiring state in places).

## Security

See [SECURITY.md](SECURITY.md) for the current, accurate security posture: guest network
mode (NAT-only by default), guest image checksum verification, VM control socket
permissions, the virtualization entitlement requirement, and VirtioFS host-path scoping
(implemented and tested ahead of the share-mounting FFI call itself being wired up - see
`crates/tinybridge-vz/src/virtiofs.rs`).

## Recent fixes (v0.6.0)

- **Orphaned-process detection**: the daemon and `tinybridge-vmhost` used to blindly delete
  a leftover control socket on startup with no check for whether the process that created
  it was still running. `tinybridge-core::pid_lock::PidLock` now writes a `<socket>.pid`
  file and checks whether that PID is actually alive (via `ps -p`) before touching
  anything: a live instance's socket is left alone (startup errors out instead), a dead
  instance's stale socket+lock is cleaned up automatically.
- **`tinybridge doctor` virtualization check is now a runtime check, not just an
  architecture check**: it now also reads `sysctl kern.hv_support` (the same signal
  Hypervisor.framework/Virtualization.framework consult internally), so it correctly fails
  when hardware virtualization is unavailable even on Apple Silicon - e.g. running nested
  inside another VM without virtualization passthrough, or disabled by an MDM profile.

## Known debt / deliberately deferred

- **Distribution is still broken for external users, though partially fixed**: the Homebrew
  tap is private ([#1](https://github.com/Mullassery/TinyBridge/issues/1)) - that part is
  unresolved and is the actual blocker. The `v0.5.0` darwin release tarball now bundles the
  required dylib and a `SHA256SUMS` file (re-verified directly), but the `tinybridge-vmhost`
  binary inside it still has no `LC_RPATH`, so it needs `DYLD_LIBRARY_PATH` set manually per
  the bundled `INSTALL.txt` ([#2](https://github.com/Mullassery/TinyBridge/issues/2)). The
  older `v0.3.1` tag still ships assets internally named `0.3.0` (unchanged, low-priority
  since `v0.5.0` is current). See "Installing" above. Building from source is the only
  install path that doesn't require a manual `DYLD_LIBRARY_PATH` workaround.
- **CI is now green.** It had been red on every push since the `v0.5.0` release because
  `cargo fmt --check` failed on the first gate in `.github/workflows/ci.yml`, before clippy
  or the test suite ever ran. Fixing the formatting surfaced ~50 real clippy warnings across
  `tinybridge-core`, `tinybridge-cli`, and `tinybridge-daemon` that clippy had also never
  actually gotten a chance to check (same cascading-hidden-failure shape) - all fixed for
  real (not suppressed), confirmed via a real, watched CI run. With `DYLD_LIBRARY_PATH`
  pointed at `target/swift-libs`, `cargo test --workspace --exclude tinybridge-daemon`
  passes 396/396, and `cargo test -p tinybridge-daemon` passes 185/194 (the other 9 are the
  pre-existing, named failures CI already skips - see `.github/workflows/ci.yml`).
- Two `TODO`s left in the CLI: `crates/tinybridge-cli/src/commands/logs.rs` (log retrieval
  from the daemon is not yet implemented) and `crates/tinybridge-cli/src/commands/launch.rs`
  (system detection is not yet implemented).
- **Windows/Linux hypervisor backends**: not implemented (see "Honest status" above).
- **Guest image pipeline**: no bundled/auto-downloaded kernel+rootfs pair verified to boot
  to a login prompt yet. `scripts/build-rootfs-multi-tier.sh` now verifies its Ubuntu cloud
  image download against upstream `SHA256SUMS` before use, but building a full, tested,
  bootable rootfs image end-to-end is still open work. A real, complete boot attempt was made
  (see "What's actually been verified" above) and is currently blocked by an external macOS
  26.x ARM64 Virtualization.framework bug, not by anything in this pipeline.
- **VirtioFS host-directory sharing**: not wired to a real FFI call.
  Virtualization.framework requires directory shares to be configured at VM-creation time,
  and the config plumbing for that doesn't exist yet, so `VirtioFS::attach()` returns an
  explicit "not implemented" error rather than silently doing nothing. Host-path scoping
  (canonicalize + allowlist, reject `..` escapes, default read-only) is implemented and
  unit-tested ahead of that wiring.
- **`objc` 0.2 / `block` 0.1.6`** (used by `tinybridge-clipboard`'s macOS pasteboard
  integration) are unmaintained; `block` already triggers a Rust future-incompatibility
  warning. `objc2` is the maintained successor but migrating is a real API rewrite,
  deliberately not bundled into this pass - see the comment in
  `crates/tinybridge-clipboard/Cargo.toml`.
- **Performance**: no end-to-end guest-boot-to-shell benchmarks exist yet (see "What's
  actually been verified" above for what *has* been measured). Treat any boot-time number
  you see elsewhere in this repo's history/docs as unverified until re-measured against a
  real guest image.

## License

This project is licensed under the [Apache License 2.0](LICENSE).

## Contact

mullassery@gmail.com — see [SECURITY.md](SECURITY.md) for security-specific reporting.
