use super::DiagnosticCheck;
use crate::result::{CheckResult, DiagnosticSeverity};
use async_trait::async_trait;

pub struct VirtualizationCheck;

#[async_trait]
impl DiagnosticCheck for VirtualizationCheck {
    async fn run(&self) -> CheckResult {
        if !cfg!(target_os = "macos") {
            return CheckResult::new(
                "Apple Virtualization Framework",
                DiagnosticSeverity::Fail,
                "Not running on macOS",
            )
            .with_recommendation("TinyBridge requires macOS 13 or later");
        }
        build_macos_result(check_apple_silicon(), check_hardware_virtualization_support())
    }
}

/// Builds the check result for macOS given the architecture and the
/// runtime hardware-virtualization signal. Factored out from `run()` so
/// every branch (including the "hardware virtualization is unavailable"
/// fail path, which can't be forced on a real machine without root and an
/// MDM profile) is unit-testable with fake inputs instead of only being
/// exercisable by code review.
fn build_macos_result(apple_silicon: bool, hv_support: Option<bool>) -> CheckResult {
    // Compile-time architecture is necessary but not sufficient: it says
    // nothing about whether hardware virtualization is actually usable
    // *right now* on *this* machine. A Mac running nested inside another
    // hypervisor without virtualization passthrough exposed, or with it
    // disabled by an MDM/config profile, is still aarch64 -- and would
    // still fail here without a runtime check.
    if hv_support == Some(false) {
        return CheckResult::new(
            "Apple Virtualization Framework",
            DiagnosticSeverity::Fail,
            "Hardware virtualization is not available on this machine",
        )
        .with_details(
            "`sysctl kern.hv_support` reports 0 -- the Hypervisor.framework/\
             Virtualization.framework cannot start VMs here even though this \
             is macOS. Common causes: running nested inside another VM without \
             virtualization passthrough enabled, or virtualization disabled by \
             an MDM/configuration profile.",
        )
        .with_recommendation(
            "If this is a VM/CI runner, enable nested virtualization on the \
             host hypervisor. If this is a managed Mac, check for an MDM \
             profile disabling virtualization.",
        );
    }

    let runtime_unconfirmed_note = if hv_support.is_none() {
        " Could not confirm hardware virtualization support at runtime \
          (`sysctl kern.hv_support` was unavailable) -- falling back to the \
          architecture check only."
    } else {
        ""
    };

    if apple_silicon {
        CheckResult::new(
            "Apple Virtualization Framework",
            DiagnosticSeverity::Pass,
            "Apple Virtualization Framework available on Apple Silicon",
        )
        .with_details(&format!(
            "macOS running on ARM64 architecture with VZ support.{runtime_unconfirmed_note}"
        ))
    } else {
        CheckResult::new(
            "Apple Virtualization Framework",
            DiagnosticSeverity::Warning,
            "Apple Virtualization Framework available but on Intel architecture",
        )
        .with_details(&format!(
            "TinyBridge is optimized for Apple Silicon (M1/M2/M3+).{runtime_unconfirmed_note}"
        ))
        .with_recommendation("Consider upgrading to Apple Silicon Mac for better performance")
    }
}

fn check_apple_silicon() -> bool {
    cfg!(target_arch = "aarch64")
}

/// Runtime check for whether hardware-assisted virtualization is actually
/// usable on this machine, via `sysctl kern.hv_support` -- the documented
/// macOS mechanism for this, and the same signal Hypervisor.framework/
/// Virtualization.framework consult internally. Returns `None` (not a
/// guess) if the sysctl can't be read at all, e.g. a future/past macOS
/// that renamed or removed it, or `sysctl` itself being unavailable.
fn check_hardware_virtualization_support() -> Option<bool> {
    let output = std::process::Command::new("sysctl")
        .args(["-n", "kern.hv_support"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    match String::from_utf8_lossy(&output.stdout).trim() {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_virtualization_check_runs() {
        let check = VirtualizationCheck;
        let result = check.run().await;
        assert!(!result.message.is_empty());
    }

    #[test]
    fn test_apple_silicon_detection() {
        let is_arm = check_apple_silicon();
        #[cfg(target_arch = "aarch64")]
        assert!(is_arm);
        #[cfg(not(target_arch = "aarch64"))]
        assert!(!is_arm);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_hardware_virtualization_support_reads_a_real_boolean_on_macos() {
        // On any real macOS test runner `kern.hv_support` must resolve to a
        // definite yes/no -- this is the actual sysctl, not a mock.
        let result = check_hardware_virtualization_support();
        assert!(
            result.is_some(),
            "kern.hv_support should be readable on macOS"
        );
    }

    // build_macos_result is factored out precisely so these branches --
    // including "hardware virtualization unavailable", which can't be
    // forced on a real machine without root and an MDM profile -- are
    // genuinely testable with fake inputs rather than only exercisable by
    // code review.

    #[test]
    fn fails_hard_when_hardware_virtualization_is_unavailable_even_on_apple_silicon() {
        let result = build_macos_result(true, Some(false));
        assert_eq!(result.severity, DiagnosticSeverity::Fail);
        assert!(result.message.contains("not available"));
    }

    #[test]
    fn fails_hard_when_hardware_virtualization_is_unavailable_on_intel() {
        let result = build_macos_result(false, Some(false));
        assert_eq!(result.severity, DiagnosticSeverity::Fail);
    }

    #[test]
    fn passes_on_apple_silicon_with_confirmed_hardware_virtualization() {
        let result = build_macos_result(true, Some(true));
        assert_eq!(result.severity, DiagnosticSeverity::Pass);
    }

    #[test]
    fn warns_on_intel_with_confirmed_hardware_virtualization() {
        let result = build_macos_result(false, Some(true));
        assert_eq!(result.severity, DiagnosticSeverity::Warning);
    }

    #[test]
    fn passes_on_apple_silicon_when_runtime_check_is_inconclusive_but_notes_it() {
        let result = build_macos_result(true, None);
        assert_eq!(result.severity, DiagnosticSeverity::Pass);
        assert!(result
            .details
            .as_deref()
            .unwrap_or_default()
            .contains("Could not confirm"));
    }
}
