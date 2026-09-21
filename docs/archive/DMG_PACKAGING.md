# .dmg Packaging & Notarization Guide

**Create production-ready macOS installer packages**

---

## Overview

TinyBridge ships as a code-signed, notarized `.dmg` file. This guide covers:

1. Building the .dmg from compiled binaries
2. Codesigning for distribution
3. Notarizing with Apple
4. Automated CI/CD packaging

---

## Prerequisites

- Compiled binaries (Rust CLI + daemon, Swift library)
- Developer ID certificate (for production)
- Apple ID and app-specific password (for notarization)
- macOS 11+

---

## Directory Structure

```
TinyBridge.dmg
└── TinyBridge.app/
    ├── Contents/
    │   ├── MacOS/
    │   │   ├── TinyBridgeApp          (Swift UI application)
    │   │   ├── tinybridge             (Rust CLI binary)
    │   │   └── tinybridged            (Rust daemon)
    │   ├── Frameworks/
    │   │   └── libTinyBridgeVZBridge.dylib
    │   ├── Resources/
    │   │   ├── AppIcon.icns
    │   │   └── LaunchAgent.plist
    │   ├── Info.plist
    │   └── _CodeSignature/
└── Install TinyBridge.lnk             (Link to /Applications)
```

---

## Build Steps

### 1. Create App Bundle

```bash
# Create structure
mkdir -p TinyBridge.app/Contents/{MacOS,Frameworks,Resources}

# Copy binaries
cp target/release/tinybridge TinyBridge.app/Contents/MacOS/
cp target/release/tinybridged TinyBridge.app/Contents/MacOS/
cp target/swift-libs/libTinyBridgeVZBridge.dylib TinyBridge.app/Contents/Frameworks/

# Copy app resources
cp packaging/AppIcon.icns TinyBridge.app/Contents/Resources/
cp packaging/Info.plist TinyBridge.app/Contents/
cp packaging/LaunchAgent.plist TinyBridge.app/Contents/Resources/
```

### 2. Create Info.plist

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    
    <key>CFBundleExecutable</key>
    <string>TinyBridgeApp</string>
    
    <key>CFBundleIdentifier</key>
    <string>com.mullassery.tinybridge</string>
    
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    
    <key>CFBundleName</key>
    <string>TinyBridge</string>
    
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    
    <key>CFBundleVersion</key>
    <string>1</string>
    
    <key>LSMinimumSystemVersion</key>
    <string>14.0</string>
    
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    
    <key>NSRequiresIPhoneOS</key>
    <false/>
</dict>
</plist>
```

### 3. Code Sign

For development:

```bash
# Sign app bundle
codesign --deep --force --verify --verbose \
  --sign - TinyBridge.app

# Verify signature
codesign -v TinyBridge.app
```

For production (with Developer ID):

```bash
# Get certificate details
security find-identity -v -p codesigning

# Sign with Developer ID
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Name (ID)" \
  TinyBridge.app

# Verify
codesign -v --detailed TinyBridge.app
```

### 4. Create .dmg

```bash
# Create temporary DMG
hdiutil create -volname "TinyBridge" \
  -srcfolder TinyBridge.app \
  -ov -format UDRW \
  TinyBridge-dev.dmg

# Add Applications link (optional)
mkdir -p /Volumes/TinyBridge
hdiutil mount TinyBridge-dev.dmg
ln -s /Applications /Volumes/TinyBridge/Applications
hdiutil unmount /Volumes/TinyBridge

# Convert to compressed read-only
hdiutil convert TinyBridge-dev.dmg \
  -format UDZO \
  -o TinyBridge.dmg

# Clean up
rm TinyBridge-dev.dmg
```

---

## Notarization (Production)

Required for distribution through non-App-Store channels.

### 1. Prepare for Notarization

```bash
# Staple the notarization ticket
xcrun stapler staple TinyBridge.dmg

# Verify staple
xcrun stapler validate TinyBridge.dmg
```

### 2. Submit for Notarization

```bash
# Create app-specific password at https://appleid.apple.com/account/security
# Store in Keychain:
xcrun notarytool store-credentials "TinyBridge" \
  --apple-id your@email.com \
  --password APP_SPECIFIC_PASSWORD \
  --team-id TEAM_ID

# Submit DMG
xcrun notarytool submit TinyBridge.dmg \
  --keychain-profile "TinyBridge" \
  --wait

# On success, staple the ticket
xcrun stapler staple TinyBridge.dmg
```

### 3. Verify Notarization

```bash
spctl -a -v TinyBridge.dmg
# Output should show: accepted (developer id)

# Mount and check
hdiutil mount TinyBridge.dmg
spctl -a -v /Volumes/TinyBridge/TinyBridge.app
hdiutil unmount /Volumes/TinyBridge
```

---

## Automated Packaging Script

Save as `packaging/build-dmg.sh`:

```bash
#!/bin/bash
set -e

VERSION=${1:-1.0.0}
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "Building TinyBridge $VERSION..."

# Compile binaries (assumes build already done)
echo "Binaries already compiled"

# Create app structure
APP_PATH="$TEMP_DIR/TinyBridge.app"
mkdir -p "$APP_PATH/Contents/MacOS"
mkdir -p "$APP_PATH/Contents/Frameworks"
mkdir -p "$APP_PATH/Contents/Resources"

# Copy files
cp target/release/tinybridge "$APP_PATH/Contents/MacOS/"
cp target/release/tinybridged "$APP_PATH/Contents/MacOS/"
cp target/swift-libs/libTinyBridgeVZBridge.dylib "$APP_PATH/Contents/Frameworks/"
cp packaging/AppIcon.icns "$APP_PATH/Contents/Resources/"
cp packaging/Info.plist "$APP_PATH/Contents/"

# Update version in plist
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" \
  "$APP_PATH/Contents/Info.plist"

# Code sign
codesign --deep --force --sign - "$APP_PATH"

# Create DMG
hdiutil create -volname "TinyBridge" \
  -srcfolder "$APP_PATH" \
  -ov -format UDRW \
  TinyBridge-temp.dmg

# Convert to compressed
hdiutil convert TinyBridge-temp.dmg \
  -format UDZO \
  -o "TinyBridge-$VERSION.dmg"

rm TinyBridge-temp.dmg

echo "✓ Created TinyBridge-$VERSION.dmg"
```

Usage:

```bash
chmod +x packaging/build-dmg.sh
./packaging/build-dmg.sh 1.0.0
```

---

## Distribution

### Manual Distribution

1. Sign .dmg with Developer ID certificate
2. Notarize with Apple (required since 2019)
3. Staple notarization ticket
4. Host on GitHub releases or website
5. Provide checksum for verification

### GitHub Releases

```bash
# Create release
gh release create v1.0.0 TinyBridge-1.0.0.dmg \
  --title "TinyBridge 1.0.0" \
  --notes "See CHANGELOG.md for details"
```

### Homebrew Cask

Create `homebrew-cask/Casks/tinybridge.rb`:

```ruby
cask "tinybridge" do
  version "1.0.0"
  sha256 "abc123..."
  url "https://github.com/Mullassery/tinybridge/releases/download/v#{version}/TinyBridge-#{version}.dmg"
  
  name "TinyBridge"
  desc "Linux development substrate for macOS"
  homepage "https://github.com/Mullassery/tinybridge"
  
  app "TinyBridge.app"
  
  postflight do
    system_command "/usr/local/bin/tinybridge", args: ["--version"]
  end
end
```

---

## Troubleshooting

### "Certificate not found"

```bash
# List available certificates
security find-identity -v -p codesigning
```

### "DMG is not notarized"

```bash
# Check notarization status
spctl -a -v TinyBridge.dmg

# If missing, resubmit:
xcrun notarytool submit TinyBridge.dmg \
  --keychain-profile "TinyBridge" \
  --wait
xcrun stapler staple TinyBridge.dmg
```

### "Library not found at runtime"

Verify `@rpath`:

```bash
otool -L TinyBridge.app/Contents/MacOS/tinybridged
# Check that libTinyBridgeVZBridge path is correct

# Fix if needed:
install_name_tool -change \
  "@rpath/libTinyBridgeVZBridge.dylib" \
  "@loader_path/../Frameworks/libTinyBridgeVZBridge.dylib" \
  TinyBridge.app/Contents/MacOS/tinybridged
```

---

## Next Steps

1. Build .dmg using this guide
2. Test installation on clean macOS
3. Verify app functionality
4. Notarize for distribution
5. Publish to GitHub releases

---

## References

- [Creating a macOS App Bundle](https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html)
- [Codesigning Guide](https://developer.apple.com/documentation/technicalqnotes/tn3127-inside-code-signing-and-notarization)
- [Notarization Overview](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
