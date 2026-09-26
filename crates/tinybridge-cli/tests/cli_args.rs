//! Regression test: `images`, `templates`, and `repair` each declared their
//! own local `verbose: bool` arg with the same clap id as the global
//! `-v/--verbose` count flag (`global = true`, type `u8`) on the root `Cli`
//! struct. Clap's ArgMatches type-erasure panics at runtime the first time
//! it tries to downcast the shared "verbose" id to two different types,
//! so these commands crashed unconditionally, even with no other flags.
//! Renamed the local fields to `details` to stop colliding with the global
//! flag's id.
use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tinybridge"))
        .args(args)
        .output()
        .expect("failed to run tinybridge binary")
}

#[test]
fn images_does_not_panic_on_the_shared_verbose_arg_id() {
    let out = run(&["images"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("Mismatch between definition and access"),
        "images panicked on arg parsing: {stderr}"
    );
    assert!(out.status.success(), "images exited non-zero: {stderr}");
}

#[test]
fn templates_does_not_panic_on_the_shared_verbose_arg_id() {
    let out = run(&["templates"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("Mismatch between definition and access"),
        "templates panicked on arg parsing: {stderr}"
    );
    assert!(out.status.success(), "templates exited non-zero: {stderr}");
}

#[test]
fn templates_details_flag_still_works() {
    let out = run(&["templates", "--details"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("Mismatch between definition and access"),
        "templates --details panicked: {stderr}"
    );
    assert!(out.status.success(), "templates --details exited non-zero: {stderr}");
}

#[test]
fn repair_help_does_not_panic_on_the_shared_verbose_arg_id() {
    let out = run(&["repair", "--help"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("Mismatch between definition and access"),
        "repair --help panicked: {stderr}"
    );
    assert!(out.status.success(), "repair --help exited non-zero: {stderr}");
}
