#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
VERSION="${1:-0.1.0}"
DIST_DIR="$REPO_DIR/dist"
APP_PATH="$DIST_DIR/TinyBridge.app"

echo "Building TinyBridge.app v$VERSION..."

# Clean previous build
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Generate icon if it doesn't exist
if [ ! -f "$SCRIPT_DIR/AppIcon.icns" ]; then
    echo "Generating icon..."
    "$SCRIPT_DIR/generate-icon.sh"
fi

# Build universal Rust binaries
echo "Building Rust binaries (universal)..."
cd "$REPO_DIR"
cargo build --release --target aarch64-apple-darwin -p tinybridge-cli -p tinybridge-daemon 2>&1 | grep -E "(Compiling|Finished)" | tail -5 || true
cargo build --release --target x86_64-apple-darwin -p tinybridge-cli -p tinybridge-daemon 2>&1 | grep -E "(Compiling|Finished)" | tail -5 || true

# Create universal binaries with lipo
echo "Creating universal binaries..."
for binary in tinybridge tinybridged; do
    lipo -create \
        "target/aarch64-apple-darwin/release/$binary" \
        "target/x86_64-apple-darwin/release/$binary" \
        -output "$DIST_DIR/$binary"
done

# Build Swift
echo "Building Swift app..."
swift build --package-path swift/ -c release 2>&1 | grep -E "(Compiling|Finished)" | tail -5 || true

# Copy Swift artifacts (native build, then lipo if cross-arch capable)
swift_exe_path=".build/release/TinyBridgeApp"
if [ -f "$swift_exe_path" ]; then
    cp "$swift_exe_path" "$DIST_DIR/TinyBridgeApp.native"
else
    # Try finding in build directory
    find swift -name TinyBridgeApp -type f -executable 2>/dev/null | head -1 | xargs -I {} cp {} "$DIST_DIR/TinyBridgeApp.native" || true
fi

# Copy dylib (native first, then lipo if available)
dylib_path=$(find swift -name "libTinyBridgeVZBridge.dylib" 2>/dev/null | head -1)
if [ -n "$dylib_path" ]; then
    cp "$dylib_path" "$DIST_DIR/libTinyBridgeVZBridge.dylib.native"
fi

# Create app bundle structure
echo "Creating app bundle..."
mkdir -p "$APP_PATH/Contents/"{MacOS,Resources,Frameworks}

# Copy binaries to MacOS
cp "$DIST_DIR/tinybridge" "$APP_PATH/Contents/MacOS/"
cp "$DIST_DIR/tinybridged" "$APP_PATH/Contents/MacOS/"
cp "$DIST_DIR/TinyBridgeApp.native" "$APP_PATH/Contents/MacOS/TinyBridgeApp"
chmod +x "$APP_PATH/Contents/MacOS"/*

# Copy frameworks
if [ -f "$DIST_DIR/libTinyBridgeVZBridge.dylib.native" ]; then
    cp "$DIST_DIR/libTinyBridgeVZBridge.dylib.native" "$APP_PATH/Contents/Frameworks/libTinyBridgeVZBridge.dylib"
fi

# Copy resources
cp "$SCRIPT_DIR/AppIcon.icns" "$APP_PATH/Contents/Resources/"
cp "$SCRIPT_DIR/com.mullassery.tinybridge.daemon.plist" "$APP_PATH/Contents/Resources/"

# Generate Info.plist
echo "Generating Info.plist..."
sed "s/__VERSION__/$VERSION/g" "$SCRIPT_DIR/Info.plist.template" > "$APP_PATH/Contents/Info.plist"

# Fix install names for dylib dependencies
echo "Fixing dylib dependencies..."
if [ -f "$APP_PATH/Contents/Frameworks/libTinyBridgeVZBridge.dylib" ]; then
    for binary in tinybridged; do
        otool -L "$APP_PATH/Contents/MacOS/$binary" 2>/dev/null | grep -q "libTinyBridgeVZBridge" && \
        install_name_tool -change "@rpath/libTinyBridgeVZBridge.dylib" \
            "@loader_path/../Frameworks/libTinyBridgeVZBridge.dylib" \
            "$APP_PATH/Contents/MacOS/$binary" || true
    done
fi

# Code sign (ad-hoc by default, can be overridden by CI with Developer ID)
echo "Code signing app..."
codesign --deep --force --sign - "$APP_PATH" 2>/dev/null || true

# Verify
echo "Verifying app bundle..."
if [ -f "$APP_PATH/Contents/Info.plist" ]; then
    echo "✓ Info.plist present"
fi
if [ -x "$APP_PATH/Contents/MacOS/TinyBridgeApp" ]; then
    echo "✓ Executable present"
fi
if [ -f "$APP_PATH/Contents/Resources/AppIcon.icns" ]; then
    echo "✓ Icon present"
fi

echo "✓ Created $APP_PATH"
