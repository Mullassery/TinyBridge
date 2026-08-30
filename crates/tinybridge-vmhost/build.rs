fn main() {
    // tinybridge-vmhost links libTinyBridgeVZBridge.dylib (via tinybridge-vz-sys) but the
    // release tarball ships it alongside the binary rather than in a fixed system path.
    // Embed the rpath at link time so dyld finds it without DYLD_LIBRARY_PATH or a
    // post-hoc `install_name_tool -add_rpath` patch (which needs headerpad room we don't
    // reserve at link time anyway).
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
}
