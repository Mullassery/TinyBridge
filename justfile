default: build

vz-bridge:
    swift build -c release --package-path swift/ --product TinyBridgeVZBridge
    mkdir -p target/swift-libs
    cp swift/.build/release/libTinyBridgeVZBridge.dylib target/swift-libs/

build: vz-bridge
    cargo build --workspace

test:
    cargo test --workspace

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

watch:
    cargo watch -x "build --workspace" -x "test --workspace"
