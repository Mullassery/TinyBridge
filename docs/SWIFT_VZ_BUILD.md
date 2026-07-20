# Swift VZ Bridge Build Guide

**Complete guide to compiling and integrating the Swift virtualization wrapper**

---

## Overview

The Swift VZ Bridge (`TinyBridgeVZBridge`) wraps macOS's Virtualization.framework in a C interface callable from Rust. This guide covers compilation, linking, and integration.

---

## Prerequisites

- Xcode 15.0+ with Swift 5.10+
- macOS 14.0+ (Sonoma or newer)
- Apple Silicon or Intel Mac
- Command line tools: `swift`, `swiftc`, `clang`

Verify installation:

```bash
swift --version
xcode-select --print-path
```

---

## Build Steps

### 1. Compile Swift Library

```bash
cd swift/

# Development build (debug symbols, no optimization)
swift build

# Release build (optimized, stripped)
swift build -c release --product TinyBridgeVZBridge
```

Output locations:
- Debug: `.build/debug/libTinyBridgeVZBridge.dylib`
- Release: `.build/release/libTinyBridgeVZBridge.dylib`

### 2. Verify Library

```bash
# Check architecture
lipo -info .build/release/libTinyBridgeVZBridge.dylib
# Output: Mach header:
#         magic cputype cpusubtype ...

# List exported symbols
nm -gU .build/release/libTinyBridgeVZBridge.dylib | head -20
# Should show: tb_vm_create, tb_vm_start, tb_vm_stop, etc.
```

### 3. Copy to Rust Workspace

```bash
# Create library directory
mkdir -p target/swift-libs

# Copy release build
cp .build/release/libTinyBridgeVZBridge.dylib target/swift-libs/

# Create link for Cargo
ln -sf $(pwd)/.build/release/libTinyBridgeVZBridge.dylib \
  ../crates/tinybridge-vz-sys/libTinyBridgeVZBridge.dylib
```

### 4. Update Cargo Configuration

In `crates/tinybridge-vz-sys/.cargo/config.toml`:

```toml
[build]
rustflags = [
  "-l", "dylib=TinyBridgeVZBridge",
  "-L", "dependency=../../target/swift-libs",
  "-L", "dependency=../../.build/release",
]
```

### 5. Build Rust Crate

```bash
cd crates/tinybridge-vz-sys/
cargo build --release
```

If linking fails, verify:
- Swift library exists and is readable
- Architecture matches (arm64 or x86_64)
- Cargo can find the library path

---

## Using the Makefile

A `Makefile` automates the build process:

```bash
# Build Swift library only
make swift-build

# Build release version
make swift-release

# Build Rust wrapper
make rust-build

# Full build (Swift + Rust)
make build

# Clean Swift artifacts
make swift-clean

# Clean everything
make clean
```

### Makefile Targets

```makefile
.PHONY: swift-build swift-release rust-build build clean swift-clean

SWIFT_DIR := swift
RUST_DIR := crates/tinybridge-vz-sys
BUILD_DIR := target/swift-libs

swift-build:
	cd $(SWIFT_DIR) && swift build

swift-release:
	cd $(SWIFT_DIR) && swift build -c release --product TinyBridgeVZBridge
	mkdir -p $(BUILD_DIR)
	cp $(SWIFT_DIR)/.build/release/libTinyBridgeVZBridge.dylib $(BUILD_DIR)/

rust-build:
	mkdir -p $(BUILD_DIR)
	cargo build --release -p tinybridge-vz-sys

build: swift-release rust-build
	@echo "Build complete: $(BUILD_DIR)/libTinyBridgeVZBridge.dylib"

clean: swift-clean
	cargo clean

swift-clean:
	cd $(SWIFT_DIR) && rm -rf .build
```

---

## Verification

After successful build, verify integration:

```bash
# Check Rust can link
cargo build --release -p tinybridge-daemon

# Verify symbols
nm -u target/release/tinybridged | grep tb_
# Should list: _tb_vm_create, _tb_vm_start, etc.

# Runtime check (requires running daemon)
./target/release/tinybridged --version
# Should start without linker errors
```

---

## Code Signing (for Distribution)

For production .dmg distribution, sign the library:

```bash
# Sign with development identity
codesign -s - target/swift-libs/libTinyBridgeVZBridge.dylib

# Verify signature
codesign -v target/swift-libs/libTinyBridgeVZBridge.dylib

# For production, use Developer ID
codesign -s "Developer ID Application: Name (ID)" \
  target/swift-libs/libTinyBridgeVZBridge.dylib
```

---

## Troubleshooting

### "dyld: Library not loaded: @rpath/libTinyBridgeVZBridge.dylib"

**Solution:** Update `@rpath` in library:

```bash
install_name_tool -change \
  "@rpath/libTinyBridgeVZBridge.dylib" \
  "@loader_path/../Frameworks/libTinyBridgeVZBridge.dylib" \
  target/release/tinybridged
```

### "symbol not found in flat namespace"

**Solution:** Ensure Swift library was built successfully:

```bash
cd swift && swift build -c release
# If fails, check Xcode installation
xcode-select --install
```

### "Swift module not found"

**Solution:** Verify Xcode tools are installed:

```bash
swift --version
# If fails, reinstall:
xcode-select --reset
xcode-select --install
```

### Architecture mismatch (x86_64 vs arm64)

**Solution:** Build for correct architecture:

```bash
# For Apple Silicon (arm64)
swift build -c release -Xswiftc -target -Xswiftc arm64-apple-macos14

# For Intel (x86_64)
swift build -c release -Xswiftc -target -Xswiftc x86_64-apple-macos14

# Create universal binary
lipo -create \
  .build/release-arm64/libTinyBridgeVZBridge.dylib \
  .build/release-x86_64/libTinyBridgeVZBridge.dylib \
  -output libTinyBridgeVZBridge.dylib
```

---

## CI/CD Integration

In GitHub Actions:

```yaml
name: Swift VZ Bridge Build

on: [push, pull_request]

jobs:
  build:
    runs-on: macos-15
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Swift Library
        run: make swift-release
      
      - name: Build Rust Wrapper
        run: make rust-build
      
      - name: Verify Symbols
        run: |
          nm -u target/release/tinybridged | grep tb_
          echo "Integration complete"
```

---

## Next Steps

Once Swift library is built:

1. **Create .dmg installer** (see DMG_PACKAGING.md)
2. **Run benchmarks** (see BENCHMARK_RUNNER.md)
3. **Test on real M5** (kernel + rootfs from BUILD_ASSETS_GUIDE.md)
4. **Document results** (update TESTING_REPORT.md)

---

## Reference

- [Swift Package Manager](https://swift.org/package-manager/)
- [Virtualization.framework](https://developer.apple.com/documentation/virtualization)
- [FFI Best Practices](https://doc.rust-lang.org/nomicon/ffi.html)
