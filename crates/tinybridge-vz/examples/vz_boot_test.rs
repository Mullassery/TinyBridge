//! Real guest-boot verification harness.
//!
//! Unlike `vz_smoke.rs` (which intentionally drops `root=` and uses a fake
//! zero-filled disk to prove only that the hypervisor mechanism itself works),
//! this example uses the *real* default cmdline (`root=/dev/vda1 rw
//! console=hvc0 quiet`) against a real kernel and a real converted-to-raw
//! Ubuntu cloud disk image, with a real serial console attached and its
//! output tailed to a log file, so the actual boot progress (or failure) can
//! be observed rather than assumed.
//!
//! Run it:
//! ```sh
//! DYLD_LIBRARY_PATH=../../target/swift-libs ../../target/debug/examples/vz_boot_test \
//!     /path/to/vmlinuz /path/to/disk.raw /path/to/console.log [/path/to/initrd.img] \
//!     [/path/to/cloud-init-seed.iso]
//! ```
//!
//! The initrd argument is optional: a kernel with virtio-blk/virtio-console
//! built in (not as modules) doesn't need one, but a stock distro kernel
//! (e.g. Ubuntu's `linux-image-generic`, which builds virtio drivers as
//! modules) does - without a matching initrd it can't load the module that
//! finds `root=`, and won't get far enough to init the console either.
//!
//! The seed-image argument is also optional: a raw cloud disk image with no
//! cloud-init datasource boots to a real login prompt but has no usable
//! credentials (no password, no SSH key) - a NoCloud seed ISO (volume label
//! "cidata", containing `user-data`/`meta-data`) attached as a second,
//! read-only disk lets cloud-init set one on first boot.

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
        .expect("usage: vz_boot_test <kernel-image-path> <disk-image-path> <serial-log-path>");
    let disk = std::env::args()
        .nth(2)
        .expect("usage: vz_boot_test <kernel-image-path> <disk-image-path> <serial-log-path>");
    let serial_log = std::env::args()
        .nth(3)
        .expect("usage: vz_boot_test <kernel-image-path> <disk-image-path> <serial-log-path>");
    let initrd = std::env::args().nth(4);
    let seed_image = std::env::args().nth(5);

    let resources = tinybridge_core::Resources {
        cpu: 2,
        memory_bytes: 2 * 1024 * 1024 * 1024,
        disk_bytes: 0,
        gpu: None,
    };
    // Real default cmdline (VmConfig::new's default), kept explicit here so
    // the intent is visible: root=/dev/vda1 matches the first partition of
    // the converted raw disk image.
    let mut config = tinybridge_vz::VmConfig::new(kernel, disk, resources)
        .with_cmdline("root=/dev/vda1 rw console=hvc0".to_string())
        .with_serial_log_path(serial_log)
        // Headless: skip the VirtIO graphics device entirely so VM start
        // doesn't require a WindowServer session (see TinyBridgeVZ.swift) -
        // this test only needs the serial console to observe boot progress.
        .with_display(0, 0);
    if let Some(initrd) = initrd {
        println!("using initrd: {initrd}");
        config = config.with_initrd(initrd);
    }
    if let Some(seed_image) = seed_image {
        println!("using cloud-init seed image: {seed_image}");
        config = config.with_seed_image(seed_image);
    }

    // Real IP detection (see resolveGuestIP in TinyBridgeVZ.swift) matches
    // this name against the DHCP lease file's "name=" field, which comes
    // from the guest's own DHCP host-name option - so this must match
    // whatever hostname the guest actually reports (e.g. a cloud-init
    // seed's `hostname:` field).
    let result = tinybridge_vz::VirtualMachine::new("tinybridge-vm".to_string(), config);
    println!("create result: {result:?}");

    let Ok(vm) = result else {
        return;
    };

    println!("status before start: {:?}", vm.status());
    println!("start() call result: {:?}", vm.start());

    // Give it real time to actually get through the kernel + init + getty
    // sequence rather than the 5s smoke-test window. Bumped from 120 (60s)
    // to 320 (160s): a real Ubuntu boot spends up to ~120s in
    // systemd-networkd-wait-online.service before its own timeout releases
    // it (no cloud-init/NoCloud seed means the guest never gets network
    // config), and 60s wasn't enough to observe that it does eventually
    // continue past that to a real login prompt.
    for i in 0..700 {
        pump_main_queue(0.5);
        if i % 10 == 0 {
            println!("[t={:>6}ms] status: {:?}", (i + 1) * 500, vm.status());
        }
    }

    println!("force_stop() call result: {:?}", vm.force_stop());
    for _ in 0..2 {
        pump_main_queue(0.5);
        println!("status after force_stop: {:?}", vm.status());
    }
}
