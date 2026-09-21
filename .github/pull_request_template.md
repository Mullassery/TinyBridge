## What does this change?

<!-- Describe the change and why it's needed. -->

## Testing

<!-- What did you actually run to verify this? Paste real command output, not "should work." -->

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `DYLD_LIBRARY_PATH="$(pwd)/target/swift-libs" cargo test --workspace` (absolute path —
      see CONTRIBUTING.md)

## Honesty checklist

- [ ] If this touches README/docs, no hedge language ("planned", "may be added") for
      things that are actually broken or missing — state it plainly instead.
- [ ] If this wires previously-dead code into a real path (see `ROADMAP_HONEST.md` §3 for
      the current list), the PR description says so explicitly.
- [ ] No new `TODO`/`unimplemented!()` presented as if the feature works.
