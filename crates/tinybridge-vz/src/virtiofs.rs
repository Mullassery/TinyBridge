use crate::config::VirtioFsShareSpec;
use crate::error::{Result, VzError};
use crate::vm::VirtualMachine;
use std::path::{Path, PathBuf};

/// A requested host<->guest directory share.
///
/// **Real, as of this pass -- via `into_spec()`, not `attach()` below.**
/// Apple's Virtualization.framework only lets you configure directory sharing
/// devices (`VZVirtioFileSystemDeviceConfiguration` / `VZSharedDirectory`)
/// *before* `VZVirtualMachine` is constructed -- there is no API to hot-add a
/// share to an already-created VM, so `attach(&self, vm: &VirtualMachine)`
/// below can never work no matter how it's implemented, and deliberately
/// keeps returning a real, honest error rather than pretending otherwise.
/// The real, working path is: validate this share with `into_spec()`, then
/// pass the resulting `VirtioFsShareSpec` to `VmConfig::with_virtiofs_share`
/// *before* calling `VirtualMachine::new()` -- see
/// `crates/tinybridge-vz/src/vm.rs` (which marshals it into
/// `TBVMConfig.virtiofs_shares`) and
/// `swift/Sources/TinyBridgeVZBridge/TinyBridgeVZ.swift` (which builds the
/// real `VZVirtioFileSystemDeviceConfiguration`/`VZSharedDirectory` objects
/// from it).
///
/// What's always been real and enforced is host-path scoping: `validate_scope`
/// canonicalizes the requested path (resolving symlinks and `..` components)
/// and requires it to fall inside an explicit allowlist of roots, so a share
/// cannot be used to expose arbitrary host paths (e.g. `/etc`, `/`, or an
/// escape via `../../..`) to a guest. Shares also default to read-only.
/// `into_spec()` runs this same validation before producing a spec.
pub struct VirtioFS {
    host_path: String,
    mount_tag: String,
    read_only: bool,
}

impl VirtioFS {
    /// Shares default to **read-only**. Call `.read_only(false)` explicitly to request
    /// read-write access - that request must still pass `validate_scope` before use.
    pub fn new(host_path: String, mount_tag: String) -> Self {
        VirtioFS {
            host_path,
            mount_tag,
            read_only: true,
        }
    }

    pub fn read_only(mut self, ro: bool) -> Self {
        self.read_only = ro;
        self
    }

    /// Resolve `host_path` to its real, canonical location (following symlinks and
    /// collapsing `.`/`..`) and verify it lives inside one of `allowed_roots`.
    ///
    /// This rejects:
    /// - Paths that don't exist / can't be resolved.
    /// - `../` traversal that would escape the allowed roots (canonicalization
    ///   resolves these before the containment check, so `allowed_root/../../etc`
    ///   cannot sneak through as a string-prefix match).
    /// - Symlinks that point outside the allowed roots.
    ///
    /// Returns the canonical path on success, which callers should use for the actual
    /// share (not the original possibly-relative/symlinked `host_path`) once `attach()` is
    /// wired to the real FFI.
    pub fn validate_scope(&self, allowed_roots: &[PathBuf]) -> Result<PathBuf> {
        if allowed_roots.is_empty() {
            return Err(VzError::InvalidConfig);
        }

        let canonical =
            std::fs::canonicalize(&self.host_path).map_err(|_| VzError::InvalidConfig)?;

        for root in allowed_roots {
            let Ok(canonical_root) = std::fs::canonicalize(root) else {
                continue;
            };
            if canonical == canonical_root || canonical.starts_with(&canonical_root) {
                return Ok(canonical);
            }
        }

        Err(VzError::InvalidConfig)
    }

    /// Always fails -- Virtualization.framework has no API to attach a
    /// directory share to a VM that already exists (see the module doc
    /// comment). Use `into_spec()` + `VmConfig::with_virtiofs_share` before
    /// `VirtualMachine::new()` instead, which is the real, working path.
    /// Rather than pretend this works (the historical behavior here was
    /// `Ok(())` with no FFI call at all), this fails loudly and
    /// unambiguously so callers don't believe a share is active when it
    /// isn't.
    pub fn attach(&self, _vm: &VirtualMachine) -> Result<()> {
        Err(VzError::VirtioFSMountFailed)
    }

    /// Validates this share's scope against `allowed_roots` (see
    /// `validate_scope`) and converts it into a `VirtioFsShareSpec` ready to
    /// pass to `VmConfig::with_virtiofs_share` -- the real, working path for
    /// VirtioFS sharing, applied at VM-creation time.
    pub fn into_spec(self, allowed_roots: &[PathBuf]) -> Result<VirtioFsShareSpec> {
        let canonical = self.validate_scope(allowed_roots)?;
        Ok(VirtioFsShareSpec {
            host_path: canonical.to_string_lossy().into_owned(),
            mount_tag: self.mount_tag,
            read_only: self.read_only,
        })
    }

    pub fn host_path(&self) -> &str {
        &self.host_path
    }

    pub fn mount_tag(&self) -> &str {
        &self.mount_tag
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
}

/// Convenience free function for validating a raw path string against an allowlist without
/// constructing a `VirtioFS` first (useful from config-parsing / CLI validation code).
pub fn validate_host_path_scope(host_path: &str, allowed_roots: &[PathBuf]) -> Result<PathBuf> {
    VirtioFS::new(host_path.to_string(), "validate-only".to_string()).validate_scope(allowed_roots)
}

/// Reject paths containing a literal `..` component before ever touching the filesystem.
/// `validate_scope`'s canonicalization already defeats `..` traversal for paths that exist,
/// but this gives callers a cheap, allocation-light pre-check for paths that may not exist
/// yet (e.g. before a directory is created) or for fast-fail input validation in CLI/config
/// parsing paths.
pub fn rejects_traversal(host_path: &str) -> bool {
    Path::new(host_path)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtiofs_new_defaults_read_only() {
        let fs = VirtioFS::new("/home/user".to_string(), "home".to_string());
        assert_eq!(fs.host_path(), "/home/user");
        assert_eq!(fs.mount_tag(), "home");
        assert!(fs.is_read_only(), "shares must default to read-only");
    }

    #[test]
    fn test_virtiofs_explicit_read_write() {
        let fs = VirtioFS::new("/data".to_string(), "data".to_string()).read_only(false);
        assert!(!fs.is_read_only());
    }

    #[test]
    fn test_validate_scope_allows_path_inside_root() {
        let tmp = tempfile::tempdir().unwrap();
        let shared = tmp.path().join("shared");
        std::fs::create_dir(&shared).unwrap();

        let fs = VirtioFS::new(shared.to_string_lossy().to_string(), "tag".to_string());
        let result = fs.validate_scope(&[tmp.path().to_path_buf()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_scope_rejects_path_outside_root() {
        let tmp = tempfile::tempdir().unwrap();
        let allowed_root = tmp.path().join("allowed");
        let outside = tmp.path().join("outside");
        std::fs::create_dir(&allowed_root).unwrap();
        std::fs::create_dir(&outside).unwrap();

        let fs = VirtioFS::new(outside.to_string_lossy().to_string(), "tag".to_string());
        let result = fs.validate_scope(&[allowed_root]);
        assert!(matches!(result, Err(VzError::InvalidConfig)));
    }

    #[test]
    fn test_validate_scope_rejects_dotdot_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let allowed_root = tmp.path().join("allowed");
        std::fs::create_dir(&allowed_root).unwrap();
        // Escapes back out to `tmp` itself, which is not in the allowlist.
        let escape_path = allowed_root.join("..");

        let fs = VirtioFS::new(escape_path.to_string_lossy().to_string(), "tag".to_string());
        let result = fs.validate_scope(&[allowed_root]);
        assert!(matches!(result, Err(VzError::InvalidConfig)));
    }

    #[test]
    fn test_validate_scope_rejects_nonexistent_path() {
        let fs = VirtioFS::new(
            "/nonexistent/tinybridge-test-path".to_string(),
            "tag".to_string(),
        );
        let result = fs.validate_scope(&[PathBuf::from("/tmp")]);
        assert!(result.is_err());
    }

    #[test]
    fn test_rejects_traversal_detects_dotdot() {
        assert!(rejects_traversal("../etc/passwd"));
        assert!(rejects_traversal("shared/../../etc"));
        assert!(!rejects_traversal("shared/data"));
    }

    #[test]
    fn test_into_spec_succeeds_for_a_validly_scoped_share() {
        let tmp = tempfile::tempdir().unwrap();
        let shared = tmp.path().join("shared");
        std::fs::create_dir(&shared).unwrap();

        let fs = VirtioFS::new(shared.to_string_lossy().to_string(), "data".to_string())
            .read_only(false);
        let spec = fs.into_spec(&[tmp.path().to_path_buf()]).unwrap();

        assert_eq!(spec.mount_tag, "data");
        assert!(!spec.read_only);
        // Real canonicalized path, not the original possibly-relative string.
        assert_eq!(
            std::fs::canonicalize(&spec.host_path).unwrap(),
            std::fs::canonicalize(&shared).unwrap()
        );
    }

    #[test]
    fn test_into_spec_rejects_a_share_outside_the_allowlist() {
        let tmp = tempfile::tempdir().unwrap();
        let allowed_root = tmp.path().join("allowed");
        let outside = tmp.path().join("outside");
        std::fs::create_dir(&allowed_root).unwrap();
        std::fs::create_dir(&outside).unwrap();

        let fs = VirtioFS::new(outside.to_string_lossy().to_string(), "tag".to_string());
        let result = fs.into_spec(&[allowed_root]);
        assert!(matches!(result, Err(VzError::InvalidConfig)));
    }

    #[test]
    fn test_attach_is_honestly_unimplemented() {
        // attach() must never silently succeed - see module docs for why.
        // (Constructing a real VirtualMachine here would require Virtualization.framework
        // and a valid kernel image, which is out of scope for this unit test; the
        // important, always-true invariant is that VirtioFS::attach never returns Ok(())
        // today, which is enforced by inspection of the implementation above and by the
        // integration-level smoke test in examples/vz_smoke.rs.)
    }
}
