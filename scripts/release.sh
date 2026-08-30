#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

VERSION=$(grep '^version' Cargo.toml | head -1 | sed -E 's/.*"(.+)".*/\1/')
ARCH=$(uname -m)
case "$ARCH" in
  arm64)   TARGET="aarch64-apple-darwin" ;;
  x86_64)  TARGET="x86_64-apple-darwin" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

RELEASE_NAME="tinybridge-$VERSION-$TARGET"
RELEASE_DIR="release"
STAGE_DIR="$RELEASE_DIR/$RELEASE_NAME"

echo "Building TinyBridge $VERSION for $TARGET"

echo "==> Building Swift bridge dylib"
swift build --package-path swift/ -c release
mkdir -p target/swift-libs
cp swift/.build/release/libTinyBridgeVZBridge.dylib target/swift-libs/

echo "==> Building Rust binaries"
cargo build --release --bin tinybridge --bin tinybridged --bin tinybridge-vmhost

echo "==> Codesigning tinybridge-vmhost with the virtualization entitlement"
codesign --force --sign - \
  --entitlements crates/tinybridge-vmhost/tinybridge-vmhost.entitlements \
  target/release/tinybridge-vmhost

echo "==> Verifying rpath is embedded (no DYLD_LIBRARY_PATH needed at install time)"
if ! otool -l target/release/tinybridge-vmhost | grep -q "@executable_path"; then
  echo "ERROR: tinybridge-vmhost has no @executable_path rpath — refusing to package a binary" >&2
  echo "that would dyld-crash on launch. Check crates/tinybridge-vmhost/build.rs." >&2
  exit 1
fi

echo "==> Assembling release directory"
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"
cp target/release/tinybridge "$STAGE_DIR/"
cp target/release/tinybridged "$STAGE_DIR/"
cp target/release/tinybridge-vmhost "$STAGE_DIR/"
cp target/swift-libs/libTinyBridgeVZBridge.dylib "$STAGE_DIR/"
cp crates/tinybridge-vmhost/tinybridge-vmhost.entitlements "$STAGE_DIR/"
cp LICENSE "$STAGE_DIR/"
cp README.md "$STAGE_DIR/"

cat > "$STAGE_DIR/INSTALL.txt" <<EOF
TinyBridge $VERSION - macOS (Apple Silicon, arm64)

This archive contains:
  tinybridge                  - CLI
  tinybridged                 - daemon
  tinybridge-vmhost           - per-VM host process (drives Virtualization.framework)
  libTinyBridgeVZBridge.dylib - Swift bridge to Virtualization.framework
  tinybridge-vmhost.entitlements

Install:
  1. Place libTinyBridgeVZBridge.dylib alongside the binaries (same
     directory). tinybridge-vmhost is built with an @executable_path rpath,
     so it finds the dylib there automatically — no DYLD_LIBRARY_PATH or
     install_name_tool patch needed.
  2. Codesign tinybridge-vmhost with the virtualization entitlement (ad-hoc
     signing is sufficient, no paid Apple Developer account required):

       codesign --force --sign - \\
         --entitlements tinybridge-vmhost.entitlements \\
         tinybridge-vmhost

  3. Put \`tinybridge\`, \`tinybridged\`, and \`tinybridge-vmhost\` on your PATH.
  4. Run \`tinybridged\` (the daemon), then use \`tinybridge\` (the CLI).

This is a macOS-only build. See README.md's "Honest status" and
"What's actually been verified" sections for exactly what has and hasn't
been confirmed working end-to-end.
EOF

echo "==> Generating checksums"
(cd "$STAGE_DIR" && shasum -a 256 tinybridge tinybridged tinybridge-vmhost libTinyBridgeVZBridge.dylib > SHA256SUMS)

echo "==> Packaging tarball"
tar -C "$RELEASE_DIR" -czf "$RELEASE_DIR/$RELEASE_NAME.tar.gz" "$RELEASE_NAME"
TARBALL_SHA=$(shasum -a 256 "$RELEASE_DIR/$RELEASE_NAME.tar.gz" | awk '{print $1}')

echo ""
echo "Release package ready: $RELEASE_DIR/$RELEASE_NAME.tar.gz"
echo "Tarball SHA256: $TARBALL_SHA"
echo ""
echo "Next steps:"
echo "  1. git tag v$VERSION && git push origin v$VERSION"
echo "  2. gh release create v$VERSION $RELEASE_DIR/$RELEASE_NAME.tar.gz --title \"TinyBridge v$VERSION\" --notes '...'"
echo "  3. Update homebrew-tinybridge Formula/*.rb url + sha256 to \"$TARBALL_SHA\""
