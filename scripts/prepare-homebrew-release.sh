#!/bin/bash
set -e

VERSION="0.3.0"
TEMP_DIR=$(mktemp -d)
RELEASE_DIR="homebrew-release"

echo "🔨 Building TinyBridge $VERSION for Homebrew..."

# Create release directory
mkdir -p "$RELEASE_DIR"

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
  arm64)
    TARGET="aarch64-apple-darwin"
    BINARY_ARCH="aarch64"
    ;;
  x86_64)
    TARGET="x86_64-apple-darwin"
    BINARY_ARCH="x86_64"
    ;;
  *)
    echo "❌ Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

echo "📦 Target: $TARGET"

# Build CLI
echo "Building tinybridge CLI..."
cargo build --release --bin tinybridge

# Build daemon
echo "Building tinybridged daemon..."
cargo build --release --bin tinybridged

# Build vmhost
echo "Building tinybridge-vmhost..."
cargo build --release --bin tinybridge-vmhost

# Package CLI
echo "Packaging tinybridge..."
TINYBRIDGE_DIR="$TEMP_DIR/tinybridge-$VERSION-$BINARY_ARCH-apple-darwin"
mkdir -p "$TINYBRIDGE_DIR"
cp target/release/tinybridge "$TINYBRIDGE_DIR/"
tar -C "$TEMP_DIR" -czf "$RELEASE_DIR/tinybridge-$VERSION-$BINARY_ARCH-apple-darwin.tar.gz" "tinybridge-$VERSION-$BINARY_ARCH-apple-darwin"
TINYBRIDGE_SHA=$(shasum -a 256 "$RELEASE_DIR/tinybridge-$VERSION-$BINARY_ARCH-apple-darwin.tar.gz" | awk '{print $1}')
echo "✅ tinybridge: $TINYBRIDGE_SHA"

# Package daemon
echo "Packaging tinybridged..."
TINYBRIDGED_DIR="$TEMP_DIR/tinybridged-$VERSION-$BINARY_ARCH-apple-darwin"
mkdir -p "$TINYBRIDGED_DIR"
cp target/release/tinybridged "$TINYBRIDGED_DIR/"
tar -C "$TEMP_DIR" -czf "$RELEASE_DIR/tinybridged-$VERSION-$BINARY_ARCH-apple-darwin.tar.gz" "tinybridged-$VERSION-$BINARY_ARCH-apple-darwin"
TINYBRIDGED_SHA=$(shasum -a 256 "$RELEASE_DIR/tinybridged-$VERSION-$BINARY_ARCH-apple-darwin.tar.gz" | awk '{print $1}')
echo "✅ tinybridged: $TINYBRIDGED_SHA"

# Package vmhost
echo "Packaging tinybridge-vmhost..."
VMHOST_DIR="$TEMP_DIR/tinybridge-vmhost-$VERSION-$BINARY_ARCH-apple-darwin"
mkdir -p "$VMHOST_DIR"
cp target/release/tinybridge-vmhost "$VMHOST_DIR/"
tar -C "$TEMP_DIR" -czf "$RELEASE_DIR/tinybridge-vmhost-$VERSION-$BINARY_ARCH-apple-darwin.tar.gz" "tinybridge-vmhost-$VERSION-$BINARY_ARCH-apple-darwin"
VMHOST_SHA=$(shasum -a 256 "$RELEASE_DIR/tinybridge-vmhost-$VERSION-$BINARY_ARCH-apple-darwin.tar.gz" | awk '{print $1}')
echo "✅ tinybridge-vmhost: $VMHOST_SHA"

# Generate formula updates
echo ""
echo "📄 Update Formula SHA256 hashes:"
echo ""
echo "Formula/tinybridge.rb:"
echo "  sha256 \"$TINYBRIDGE_SHA\""
echo ""
echo "Formula/tinybridged.rb:"
echo "  sha256 \"$TINYBRIDGED_SHA\""
echo ""

# Cleanup
rm -rf "$TEMP_DIR"

echo ""
echo "✅ Release packages ready in $RELEASE_DIR/"
echo ""
echo "Next steps:"
echo "  1. Upload to GitHub releases: https://github.com/Mullassery/tinybridge/releases"
echo "  2. Update Homebrew formula SHA256 hashes"
echo "  3. Test: brew tap Mullassery/tinybridge && brew install tinybridge"
