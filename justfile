default: build

build:
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

run-daemon:
    RUST_LOG=debug cargo run --bin tinybridged

run-cli-list:
    cargo run --bin tinybridge -- list

run-cli-help:
    cargo run --bin tinybridge -- --help

check:
    cargo check --workspace

release:
    cargo build --release --workspace

watch:
    cargo watch -x "build --workspace" -x "test --workspace"
