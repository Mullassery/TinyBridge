# Security Policy

## Reporting Security Issues

Please do not open public GitHub issues for security vulnerabilities.

Email security concerns to: mullassery@gmail.com

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will acknowledge receipt within 24 hours and provide updates on remediation progress.

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest | Yes |
| Previous | Limited |
| Older | No |

## Security Best Practices

- Always use the latest version
- Report vulnerabilities privately
- Never share vulnerability details publicly before patch
- Use environment variables for secrets (not hardcoded)
- Keep dependencies updated
- Enable GitHub security features

## Guest Network Exposure

TinyBridge's macOS backend (`crates/tinybridge-vz`, the only backend with a real,
wired hypervisor integration - see [README.md](README.md#platform-support-honest-status))
attaches the guest VM's network device via Apple's `VZNATNetworkDeviceAttachment`
(`swift/Sources/TinyBridgeVZBridge/TinyBridgeVZ.swift`). This is **NAT-only**: the guest
gets outbound connectivity through the host and an address on Apple's private VZ NAT
network (`192.168.105.0/24` by convention), but the host does not bridge the guest onto
your LAN, and nothing on your local network can reach the guest directly. This is the
safe default and is not currently configurable.

The dead-code platform adapters in `crates/tinybridge-core` (`macos_adapter.rs`,
`linux_adapter.rs`, `windows_adapter.rs` - not wired into any real orchestration path,
see README) previously defaulted their `network_mode` metadata field inconsistently
(`"Bridged"` on macOS/Linux, `"NAT"` on Windows). That metadata field is currently unused
by the real macOS boot path, but the inconsistency was corrected so no future
implementation copies a Bridged-by-default pattern: all three now default to `"NAT"`.
**Bridged mode exposes the guest directly on your local network** (other devices on your
LAN can reach guest services); only use it deliberately, never as a default.

## Guest Image Integrity

VM boot images (kernel/rootfs/snapshots) are not currently verified against a
cryptographic signature - only a SHA-256 checksum, and only where wired up:

- `scripts/build-rootfs-multi-tier.sh` downloads the upstream Ubuntu cloud image and
  verifies it against Ubuntu's published `SHA256SUMS` before use, failing closed if the
  hash doesn't match.
- `tinybridge-snapshots::SnapshotMetadata` has a `checksum` field that is now actually
  populated (`with_checksum_from_file`) and enforced (`SnapshotManager::verify_integrity`,
  which fails closed - a snapshot with no recorded checksum is treated as untrusted, not
  silently accepted).

This protects against corrupted downloads and accidental/incidental tampering, not
against a fully compromised upstream mirror serving a maliciously re-signed checksum
file alongside a malicious image (that requires signature verification against a key
distributed out-of-band, which is not implemented).

## VM Control Socket

`tinybridge-vmhost`'s Unix control socket (one per running VM, JSON-RPC over
`AF_UNIX`) and the daemon's own control socket are created with explicit `0600`
(owner-read-write-only) permissions after `bind()`, rather than relying on the
process umask. A peer able to connect to this socket can start/stop/force-stop the VM,
so it must not be reachable by other local users regardless of umask configuration.

## Virtualization Entitlement

`tinybridge-vmhost` requires the `com.apple.security.virtualization` entitlement
(`crates/tinybridge-vmhost/tinybridge-vmhost.entitlements`) to call
Virtualization.framework at all. Ad-hoc codesigning with this entitlement is sufficient
for local use (no paid Apple Developer account required); see the `sign-vmhost`
recipe in the `justfile`.

## VirtioFS Host Path Sharing

Directory sharing between host and guest (`crates/tinybridge-vz/src/virtiofs.rs`) is
**not wired to a real FFI call yet** - Virtualization.framework requires directory shares
to be configured at VM-creation time, and `tb_vm_create`/`VmConfig` don't yet accept a
share list, so `VirtioFS::attach()` returns an explicit error rather than silently
succeeding (it previously returned `Ok(())` with no FFI call - i.e. it looked like it
worked but shared nothing). Ahead of that being wired up, host-path scoping is already
implemented and tested: `VirtioFS::validate_scope()` canonicalizes the requested host
path (resolving symlinks and `..` components) and requires it to fall inside an explicit
allowlist of roots, and shares default to **read-only**. Any future implementation of
`attach()` must call `validate_scope()` (or `validate_host_path_scope()`) before wiring a
share into a VM's configuration.

## Vulnerability Disclosure

When a security issue is confirmed:
1. We develop and test a fix
2. We release a new version with security patch
3. We notify users of the vulnerability and fix
4. We credit the reporter (if desired)

## Contact

Security Team: mullassery@gmail.com
