//! Manual end-to-end smoke test for the real Virtualization.framework wiring.
//!
//! This drives the exact same `tinybridge_vz::VirtualMachine` API that
//! `tinybridge-vmhost` uses, against a real kernel image, and proves the full
//! Rust -> C ABI -> Swift -> Virtualization.framework call chain actually boots a VM
//! (not a mock). It requires two things `cargo test`/`cargo run` alone won't give you:
//!
//! 1. **Codesigning with the virtualization entitlement.** Unsigned/ad-hoc-without-
//!    entitlements binaries fail with "The process doesn't have the
//!    'com.apple.security.virtualization' entitlement." Ad-hoc signing *with* the
//!    entitlement is sufficient (no paid Apple Developer account needed):
//!    ```sh
//!    cargo build -p tinybridge-vz --example vz_smoke
//!    codesign --force --sign - \
//!        --entitlements ../tinybridge-vmhost/tinybridge-vmhost.entitlements \
//!        ../../target/debug/examples/vz_smoke
//!    ```
//! 2. **A real kernel image.** Apple's `VZLinuxBootLoader` expects an uncompressed
//!    arm64 Linux `Image` (not a compressed `bzImage`/`vmlinuz`, though Ubuntu's arm64
//!    `vmlinuz-*` packages happen to be the raw Image already). One quick way to get a
//!    real one without a Linux box, verified during development of this crate:
//!    ```sh
//!    curl -LO http://ports.ubuntu.com/ubuntu-ports/pool/main/l/linux-signed-hwe-6.14/linux-image-6.14.0-27-generic_6.14.0-27.27~24.04.1_arm64.deb
//!    ar x linux-image-*.deb data.tar.zst
//!    tar --zstd -xf data.tar.zst ./boot/vmlinuz-6.14.0-27-generic
//!    ```
//!    A disk image is required by `VmConfig` but its *content* doesn't need to be a
//!    valid filesystem just to observe the hypervisor-level Running state; any
//!    existing file (e.g. `dd if=/dev/zero of=disk.img bs=1m count=64`) works - the
//!    guest kernel will simply fail to find a root filesystem after boot.
//!
//! Run it:
//! ```sh
//! DYLD_LIBRARY_PATH=../../target/swift-libs ../../target/debug/examples/vz_smoke \
//!     /path/to/vmlinuz /path/to/disk.img
//! ```
//!
//! Expected real output (as observed on Apple Silicon, macOS 26): state transitions
//! Stopped -> Running within ~500ms, a NAT guest IP is detected shortly after, and
//! `force_stop()` cleanly returns it to Stopped.
//!
//! This is also a demonstration of a real, non-obvious platform requirement: a plain
//! process (like `tinybridge-vmhost`'s original `#[tokio::main]`) never drains the
//! macOS main GCD queue that Virtualization.framework callbacks are dispatched onto,
//! so VM lifecycle calls hang forever with no error. `tinybridge-vmhost` fixes this by
//! running its async server on a background thread and dedicating the real process
//! main thread to `dispatch_main()`. This example instead pumps the run loop directly
//! via `CFRunLoopRunInMode` so it can print intermediate status and return.

use std::os::raw::{c_double, c_void};

#[allow(non_camel_case_types)]
type CFRunLoopMode = *const c_void;
#[allow(non_camel_case_types)]
type CFTimeInterval = c_double;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRunLoopRunInMode(
        mode: CFRunLoopMode,
        seconds: CFTimeInterval,
        return_after_source_handled: bool,
    ) -> i32;
    static kCFRunLoopDefaultMode: *const c_void;
}

fn pump_main_queue(seconds: f64) {
    unsafe {
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, false);
    }
}

fn main() {
    let available = tinybridge_vz::VirtualMachine::is_available();
    println!("is_available: {available}");

    let kernel = std::env::args()
        .nth(1)
        .expect("usage: vz_smoke <kernel-image-path> <disk-image-path>");
    let disk = std::env::args()
        .nth(2)
        .expect("usage: vz_smoke <kernel-image-path> <disk-image-path>");

    let resources = tinybridge_core::Resources {
        cpu: 2,
        memory_bytes: 1024 * 1024 * 1024,
        disk_bytes: 0,
        gpu: None,
    };
    let config = tinybridge_vz::VmConfig::new(kernel, disk, resources)
        .with_cmdline("console=hvc0".to_string());

    let result = tinybridge_vz::VirtualMachine::new("smoke".to_string(), config);
    println!("create result: {result:?}");

    let Ok(vm) = result else {
        return;
    };

    println!("status before start: {:?}", vm.status());
    println!("start() call result: {:?}", vm.start());

    for i in 0..10 {
        pump_main_queue(0.5);
        println!("[t={:>5}ms] status: {:?}", (i + 1) * 500, vm.status());
    }

    println!("force_stop() call result: {:?}", vm.force_stop());
    for _ in 0..2 {
        pump_main_queue(0.5);
        println!("status after force_stop: {:?}", vm.status());
    }
}
