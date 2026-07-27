# Homebrew Tap Setup for TinyBridge

## Status: ⚠️ NOT YET CONFIGURED

Currently, TinyBridge documentation references Homebrew installation, but **no tap exists yet**.

```bash
# This command currently FAILS:
brew install tinybridge
# Error: No available formula with the name "tinybridge"
```

## Solution: Create Homebrew Tap

### Step 1: Create Tap Repository

Create a new GitHub repository: `github.com/Mullassery/homebrew-tinybridge`

```bash
# On your machine
mkdir -p ~/homebrew-tinybridge
cd ~/homebrew-tinybridge
git init
```

### Step 2: Set Up Formula Files

```
homebrew-tinybridge/
├─ Formula/
│  ├─ tinybridge.rb (CLI tool)
│  ├─ tinybridged.rb (Daemon)
│  └─ tinybridge-app.rb (macOS menu bar app)
├─ README.md
└─ .gitignore
```

### Step 3: Create Formulas

#### tinybridge.rb (CLI)

```ruby
# frozen_string_literal: true

class Tinybridge < Formula
  desc "Run Linux environments on macOS with zero configuration"
  homepage "https://github.com/Mullassery/tinybridge"
  url "https://github.com/Mullassery/tinybridge/releases/download/v0.4.0/tinybridge-0.4.0-x86_64-apple-darwin.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "Apache-2.0"
  
  depends_on "tinybridged" # Requires daemon
  
  def install
    bin.install "tinybridge"
    
    # Install shell completions (if available)
    bash_completion.install "completion/tinybridge.bash" if File.exist?("completion/tinybridge.bash")
    zsh_completion.install "completion/_tinybridge" if File.exist?("completion/_tinybridge")
  end
  
  def post_install
    # Ensure daemon is running
    system "launchctl", "load", "#{HOMEBREW_PREFIX}/Library/LaunchDaemons/com.tinybridge.daemon.plist" \
      if File.exist?("#{HOMEBREW_PREFIX}/Library/LaunchDaemons/com.tinybridge.daemon.plist")
  end
  
  test do
    assert_match "tinybridge", shell_output("#{bin}/tinybridge --version")
  end
end
```

#### tinybridged.rb (Daemon)

```ruby
# frozen_string_literal: true

class Tinybridged < Formula
  desc "TinyBridge daemon - Linux environment manager"
  homepage "https://github.com/Mullassery/tinybridge"
  url "https://github.com/Mullassery/tinybridge/releases/download/v0.4.0/tinybridged-0.4.0-x86_64-apple-darwin.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "Apache-2.0"
  
  service do
    run [opt_bin/"tinybridged", "--socket", "/tmp/tinybridge.sock"]
    keep_alive true
    log_path var/"log/tinybridged.log"
    error_log_path var/"log/tinybridged.error.log"
  end
  
  def install
    bin.install "tinybridged"
    
    # Install LaunchAgent for auto-start
    (prefix/"Library/LaunchDaemons").mkpath
    cp "LaunchAgents/com.tinybridge.daemon.plist", prefix/"Library/LaunchDaemons/"
  end
  
  def post_install
    # Ensure LaunchDaemons directory exists
    system "mkdir", "-p", "#{ENV['HOME']}/Library/LaunchDaemons"
    
    # Copy LaunchAgent
    system "cp", "#{prefix}/Library/LaunchDaemons/com.tinybridge.daemon.plist", 
           "#{ENV['HOME']}/Library/LaunchDaemons/"
    
    # Load the daemon
    system "launchctl", "load", "#{ENV['HOME']}/Library/LaunchDaemons/com.tinybridge.daemon.plist"
  end
  
  test do
    assert_match "tinybridge", shell_output("#{bin}/tinybridged --version")
  end
end
```

#### tinybridge-app.rb (Menu Bar App)

```ruby
# frozen_string_literal: true

class TinybridgeApp < Formula
  desc "TinyBridge menu bar app for macOS"
  homepage "https://github.com/Mullassery/tinybridge"
  url "https://github.com/Mullassery/tinybridge/releases/download/v0.4.0/TinyBridge-0.4.0.dmg"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "Apache-2.0"
  
  app "TinyBridge.app"
  
  depends_on "tinybridge"
  depends_on "tinybridged"
  
  post_install do
    system "open", "#{appdir}/TinyBridge.app" if system("command -v open")
  end
  
  test do
    assert_predicate appdir/"TinyBridge.app", :exist?
  end
end
```

### Step 4: Create LaunchAgent Plist

**File**: `Formula/templates/com.tinybridge.daemon.plist`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.tinybridge.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/tinybridged</string>
        <string>--socket</string>
        <string>/tmp/tinybridge.sock</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>StandardErrorPath</key>
    <string>/var/log/tinybridge.log</string>
    <key>StandardOutPath</key>
    <string>/var/log/tinybridge.log</string>
</dict>
</plist>
```

### Step 5: Create README for Tap

**File**: `homebrew-tinybridge/README.md`

```markdown
# Homebrew Tap for TinyBridge

Custom Homebrew tap for TinyBridge - run Linux environments on macOS.

## Installation

```bash
# Add the tap
brew tap Mullassery/tinybridge

# Install TinyBridge CLI
brew install tinybridge

# Install daemon (auto-runs at startup)
brew install tinybridged

# Install menu bar app (optional)
brew install tinybridge-app
```

## Quick Start

```bash
# Create first environment
tinybridge up myproject

# Enter the environment
tinybridge shell myproject

# View status
tinybridge status myproject
```

## Updating

```bash
brew upgrade tinybridge
brew upgrade tinybridged
```

## Uninstalling

```bash
brew uninstall tinybridge
brew uninstall tinybridged
brew untap Mullassery/tinybridge
```

## Troubleshooting

### Daemon won't start
```bash
launchctl load ~/Library/LaunchDaemons/com.tinybridge.daemon.plist
```

### Check daemon status
```bash
launchctl list | grep tinybridge
```

### View daemon logs
```bash
tail -f /var/log/tinybridge.log
```

## Development

To modify formulas locally:
```bash
brew tap-new Mullassery/tinybridge ~/Projects/homebrew-tinybridge
# Make changes to Formula/
brew install --build-from-source Mullassery/tinybridge/tinybridge
```

## Support

Report issues at: https://github.com/Mullassery/tinybridge/issues
```

### Step 6: Push to GitHub

```bash
cd ~/homebrew-tinybridge

# Add files
git add .
git commit -m "Initial tap setup for TinyBridge"

# Push to GitHub
git remote add origin https://github.com/Mullassery/homebrew-tinybridge.git
git push -u origin main
```

---

## Installation Instructions (For Users)

### After tap is created:

```bash
# Add tap once
brew tap Mullassery/tinybridge

# Install
brew install tinybridge

# Verify
tinybridge --version
# Output: tinybridge 0.4.0
```

---

## Checklist: Before Publishing Tap

### Build Artifacts
- [ ] Release binaries built for macOS (x86_64 + ARM64)
- [ ] Calculate SHA256 checksums for each binary
- [ ] Create GitHub release with artifacts
- [ ] Test download URLs work

### Formula Testing
```bash
# Test each formula locally
brew tap-new test/local ~/test-tap
cp Formula/tinybridge.rb ~/test-tap/Formula/

# Test install from source
brew install --build-from-source test/local/tinybridge

# Test binary install
brew install test/local/tinybridge

# Verify commands work
tinybridge --version
tinybridged --version
```

### Launch Agent
- [ ] LaunchAgent plist is valid
- [ ] Loads correctly: `launchctl load ~/Library/LaunchDaemons/com.tinybridge.daemon.plist`
- [ ] Daemon starts automatically on reboot
- [ ] Can be stopped/started: `launchctl stop/start com.tinybridge.daemon`

### Documentation
- [ ] README in tap repo
- [ ] Installation instructions clear
- [ ] Troubleshooting guide complete
- [ ] Updated main README with `brew install` instructions

---

## Multi-Platform Support

### Current: macOS only
```ruby
# tinybridge.rb
url "https://github.com/Mullassery/tinybridge/releases/download/v0.4.0/tinybridge-0.4.0-x86_64-apple-darwin.tar.gz"
```

### Future: Linux support
```ruby
# Once Linux binaries exist
on_macos do
  url "https://github.com/Mullassery/tinybridge/releases/download/v0.4.0/tinybridge-0.4.0-x86_64-apple-darwin.tar.gz"
  sha256 "..."
end

on_linux do
  url "https://github.com/Mullassery/tinybridge/releases/download/v0.4.0/tinybridge-0.4.0-x86_64-linux-gnu.tar.gz"
  sha256 "..."
end
```

---

## Release Process

### Step 1: Build Release Binaries

```bash
cd /tmp/tinybridge

# Build CLI
cargo build --release --bin tinybridge
TARGET_BINARY=target/release/tinybridge

# Build daemon
cargo build --release --bin tinybridged
DAEMON_BINARY=target/release/tinybridged

# Build menu bar app
cd crates/tinybridge-macos
swift build -c release
APP_BINARY=.build/release/TinyBridgeApp
```

### Step 2: Create Tarballs

```bash
# CLI tarball
tar -czf tinybridge-0.4.0-x86_64-apple-darwin.tar.gz \
  -C target/release tinybridge

# Daemon tarball
tar -czf tinybridged-0.4.0-x86_64-apple-darwin.tar.gz \
  -C target/release tinybridged

# App DMG (already built by xcode)
```

### Step 3: Calculate Checksums

```bash
# Get SHA256 for each
shasum -a 256 tinybridge-0.4.0-x86_64-apple-darwin.tar.gz
shasum -a 256 tinybridged-0.4.0-x86_64-apple-darwin.tar.gz
shasum -a 256 TinyBridge-0.4.0.dmg
```

### Step 4: Create GitHub Release

```bash
# Use GitHub CLI
gh release create v0.4.0 \
  tinybridge-0.4.0-x86_64-apple-darwin.tar.gz \
  tinybridged-0.4.0-x86_64-apple-darwin.tar.gz \
  TinyBridge-0.4.0.dmg \
  --title "TinyBridge v0.4.0" \
  --notes "Release notes here"
```

### Step 5: Update Formulas

```bash
# In homebrew-tinybridge repo
# Update Formula/tinybridge.rb with new URL and SHA256
# Update Formula/tinybridged.rb with new URL and SHA256
# Update Formula/tinybridge-app.rb with new URL and SHA256

git add Formula/
git commit -m "chore: Update formulas for v0.4.0"
git push
```

### Step 6: Test Installation

```bash
# Fresh macOS VM or Docker
brew tap Mullassery/tinybridge
brew install tinybridge
tinybridge --version
# Should output: tinybridge 0.4.0
```

---

## Verification Checklist

| Item | Status | Notes |
|------|--------|-------|
| GitHub tap repo created | ❌ | Need to create `homebrew-tinybridge` |
| Formulas written | ⚠️ | Drafted but need checksums |
| Release binaries built | ❌ | Need CI/CD pipeline |
| Checksums calculated | ❌ | Waiting for binaries |
| GitHub releases created | ❌ | Waiting for binaries |
| Formulas tested locally | ❌ | Need to test `brew install` |
| LaunchAgent works | ❌ | Need to verify daemon auto-start |
| Users can install | ❌ | Tap needs to be public |
| Documentation complete | ⚠️ | README written but needs updates to main repo |

---

## Next Steps (Priority Order)

### Immediate (This Week)
1. ✅ Create `homebrew-tinybridge` GitHub repository
2. ⚠️ Create and test Formula files locally
3. ⚠️ Build release binaries in CI/CD
4. ⚠️ Generate checksums for formulas
5. ✅ Update main README with installation instructions

### Short Term (Next Week)
6. Create GitHub releases with binaries
7. Publish formulas to tap
8. Test installation on clean macOS
9. Test daemon auto-start via LaunchAgent
10. Document troubleshooting

### Medium Term (Within 2 Weeks)
11. Add to Homebrew community tap (if eligible)
12. Set up CI/CD for automatic releases
13. Monitor installation metrics
14. Gather user feedback

---

## Why This Matters

**Current state**: Users see `brew install tinybridge` in docs but it fails

**After tap setup**: 
```bash
brew tap Mullassery/tinybridge
brew install tinybridge
# Works perfectly!
```

This is the difference between:
- ❌ "CLI tool that requires manual setup"
- ✅ "Native macOS application that works like Docker"

Homebrew installation is critical for adoption.
