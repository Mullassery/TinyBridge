# Contributing

TinyBridge is a one-person project (see [README.md](README.md) for current status). External
contributions are welcome, but there's no dedicated maintainer team, so response times are
best-effort.

## Before you start

Read the root [README.md](README.md), specifically the "Honest status" and "Known debt"
sections, and [ROADMAP_HONEST.md](ROADMAP_HONEST.md). This project is deliberately blunt
about what is and isn't built/working — please keep that standard in any PR that touches
documentation or status claims. Do not describe untested code as working, and do not use
hedge language ("planned", "may be added") for things that are simply broken or missing —
say so plainly.

## Development setup

This is a Rust + Swift project targeting macOS only (see README's "Honest status" for why).

```bash
git clone https://github.com/Mullassery/TinyBridge.git
cd TinyBridge
just build   # or see README's "Building from source" for the equivalent manual steps
```

Requirements: macOS 13+, Rust (see `rust-toolchain.toml`), Xcode Command Line Tools, and
optionally `just`.

## Before opening a PR

Run the same checks CI runs (`.github/workflows/ci.yml`):

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
DYLD_LIBRARY_PATH="$(pwd)/target/swift-libs" cargo test --workspace
```

Note: `DYLD_LIBRARY_PATH` must be an **absolute** path for `cargo test` to find the built
Swift dylib — a relative path silently fails at test-binary load time even though
`build`/`clippy` don't need it.

## Scope guidance

- Small, focused PRs are much easier to review than large ones.
- If you're fixing a real bug or filling in a `TODO`, link to the specific line(s) in the
  PR description.
- If you're touching a crate under `crates/` that isn't wired into the daemon/CLI (see
  `ROADMAP_HONEST.md` §3 for the current list of dead/orphaned code), say so explicitly in
  the PR — either you're wiring it in for real, or the PR should explain why it should stay
  as-is.
- Windows/Linux hypervisor backends are unimplemented scaffolding and can't realistically be
  built or tested from this maintainer's macOS-only development environment — PRs adding
  real Hyper-V/KVM support are welcome but should come with your own testing evidence, since
  it can't be verified here.

## Reporting bugs / requesting features

Use the issue templates under `.github/ISSUE_TEMPLATE/`. For security issues, see
[SECURITY.md](SECURITY.md) instead of opening a public issue.

## License

By contributing, you agree your contributions are licensed under this project's
[Apache License 2.0](LICENSE).
