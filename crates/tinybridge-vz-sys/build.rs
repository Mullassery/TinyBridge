use bindgen::Builder;
use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let header_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../swift/TinyBridgeVZBridge/include/tinybridge_vz.h");

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
    println!(
        "cargo:rustc-link-search={}",
        out_path.join("../../../target/swift-libs").display()
    );
}
