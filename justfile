default: build

vz-bridge:
    swift build -c release --package-path swift/ --product TinyBridgeVZBridge
    mkdir -p target/swift-libs
    cp swift/.build/release/libTinyBridgeVZBridge.dylib target/swift-libs/

build: vz-bridge
    cargo build --workspace
    just sign-vmhost debug

test:
    cargo test --workspace

# Ad-hoc codesign tinybridge-vmhost with the com.apple.security.virtualization entitlement.
# Without this, VZVirtualMachine construction fails at runtime with "The process doesn't
# have the 'com.apple.security.virtualization' entitlement." Ad-hoc signing (no paid Apple
# Developer account required) is sufficient - verified locally against a real ARM64 Linux
# kernel boot.
sign-vmhost profile="release":
    codesign --force --sign - \
        --entitlements crates/tinybridge-vmhost/tinybridge-vmhost.entitlements \
        target/{{profile}}/tinybridge-vmhost

lint:
    cargo fmt --check
    cargo clippy --workspace -- -D warnings

fmt:
    cargo fmt --all

clean:
    cargo clean
    swift package clean --package-path swift/

run-daemon:
    RUST_LOG=debug cargo run --bin tinybridged

run-cli-list:
    cargo run --bin tinybridge -- list

run-cli-help:
    cargo run --bin tinybridge -- --help

check:
    cargo check --workspace

release: vz-bridge
    cargo build --release --workspace
    just sign-vmhost release

watch:
    cargo watch -x "build --workspace" -x "test --workspace"
