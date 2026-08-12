use bindgen::Builder;
use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let header_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../swift/Sources/CTinyBridgeVZ/tinybridge_vz.h");
    println!("cargo:rerun-if-changed={}", header_path.display());

    let bindings = Builder::default()
        .header(header_path.to_string_lossy().to_string())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("TB.*")
        .allowlist_function("tb_.*")
        .allowlist_var("TB_.*")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rustc-link-lib=dylib=TinyBridgeVZBridge");

    // Use absolute path to swift-libs
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let swift_libs = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("target/swift-libs"))
        .expect("Could not determine swift-libs path");

    if swift_libs.exists() {
        println!("cargo:rustc-link-search={}", swift_libs.display());
    }
}
