# TinyBridge: Homebrew Distribution Strategy

## Overview

TinyBridge distributes exclusively via Homebrew for Phase 1-3. This keeps distribution simple while reaching the target developer audience effectively.

## Distribution Flow

```
GitHub Actions CI/CD
    ├── Cargo build (Rust CLI + daemon)
    ├── Xcode build (Swift app)
    ├── Sign with Developer ID
    ├── Notarize with Apple
    └── Create .dmg
            ↓
    GitHub Release (draft)
            ↓
    Homebrew Cask Formula (GitHub PR)
            ↓
    User: brew install --cask tinybridge
```

## Cask Formula Structure

**File:** `homebrew-cask/Casks/t/tinybridge.rb`

```ruby
cask "tinybridge" do
  version "1.0.0"
  sha256 "abc123..."

  url "https://github.com/Mullassery/tinybridge/releases/download/v#{version}/tinybridge.dmg"
  name "TinyBridge"
  desc "Native macOS Linux development substrate"
  homepage "https://github.com/Mullassery/tinybridge"

  app "TinyBridge.app"
  binary "#{appdir}/TinyBridge.app/Contents/MacOS/tinybridge"

  zap trash: [
    "#{ENV['HOME']}/Library/Caches/com.mullassery.tinybridge",
    "#{ENV['HOME']}/Library/Preferences/com.mullassery.tinybridge*",
  ]
end
```

## GitHub Actions Workflow

**File:** `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  build-and-release:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      # Build Rust (CLI + daemon)
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: aarch64-apple-darwin,x86_64-apple-darwin
      
      - name: Build Rust (universal binary)
        run: |
          cargo build --release --target aarch64-apple-darwin
          cargo build --release --target x86_64-apple-darwin
          # lipo to combine into universal binary
      
      # Build Swift (macOS app)
      - name: Build Swift app
        run: |
          xcode-select --install || true
          xcodebuild -scheme TinyBridge -configuration Release build
      
      # Sign + Notarize
      - name: Sign application
        env:
          DEVELOPER_ID_APPLICATION: ${{ secrets.DEVELOPER_ID_APPLICATION }}
          DEVELOPER_ID_PASSWORD: ${{ secrets.DEVELOPER_ID_PASSWORD }}
        run: |
          codesign -s "$DEVELOPER_ID_APPLICATION" TinyBridge.app
      
      - name: Notarize with Apple
        run: |
          xcrun notarytool submit tinybridge.dmg \
            --apple-id ${{ secrets.APPLE_ID }} \
            --password ${{ secrets.APPLE_ID_PASSWORD }} \
            --team-id ${{ secrets.APPLE_TEAM_ID }} \
            --wait
      
      # Create release
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: tinybridge.dmg
          draft: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Submission Process

### First Release (v1.0.0)

1. **Fork Homebrew/homebrew-cask** (or maintain in tinybridge repo as tap)
2. **Submit PR** with cask formula
3. **Homebrew maintainers review** (usually 1-2 weeks)
4. **Merged** → Available via `brew install --cask tinybridge`

### Subsequent Releases

1. **Create GitHub Release** with .dmg
2. **Update SHA256** in formula
3. **Update version** number
4. **Submit PR** to homebrew-cask
5. **Auto-merge** (usually quick)

## Alternative: Custom Tap (Faster)

If Homebrew review is slow, create a custom tap:

```bash
# Users install via:
brew tap mullassery/tinybridge
brew install --cask tinybridge

# Maintain in separate repo:
github.com/Mullassery/homebrew-tinybridge
```

This gives full control over release timing.

## Certificate & Signing Setup

**Required:**
- Developer ID Application certificate (from Apple Developer Program)
- Apple ID + app-specific password (for notarization)
- `DEVELOPER_ID_APPLICATION` secret (email of cert)

**Setup:**
1. Enroll in Apple Developer Program ($99/year)
2. Create Developer ID Application certificate in Xcode
3. Export certificate to `.p8` format
4. Add to GitHub Secrets: `DEVELOPER_ID_APPLICATION`, `DEVELOPER_ID_PASSWORD`
5. Add Apple ID secrets: `APPLE_ID`, `APPLE_ID_PASSWORD`, `APPLE_TEAM_ID`

## Metrics

Track via:
- GitHub download stats (`tinybridge.dmg` releases)
- `brew analytics` (Homebrew tracks cask installs)
- Self-hosted: add telemetry (optional, respect privacy)

## Timeline

| Phase | Timeline | Distribution |
|-------|----------|--------------|
| Phase 1-2 | Alpha/Beta | GitHub Releases (manual) |
| Phase 2 end | v1.0.0 | Homebrew submission |
| Phase 3+ | Stable | Homebrew + custom tap (optional) |
