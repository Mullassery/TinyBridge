# Building TinyBridge Menu Bar App

## Quick Start

### Prerequisites
- macOS 13+ (Ventura or later)
- Xcode Command Line Tools: `xcode-select --install`
- Swift 5.9+

### Build Menu Bar App

```bash
cd crates/tinybridge-macos

# Build for development
swift build

# Run directly
.build/debug/TinyBridgeApp

# Or build release
swift build -c release
.build/release/TinyBridgeApp

# Generate Xcode project (for IDE development)
swift package generate-xcodeproj
open TinyBridgeApp.xcodeproj
```

## Architecture: Menu Bar App

### Component Breakdown

```
TinyBridgeApp.swift (450 LOC)
├─ @main struct TinyBridgeApp
│  └─ MenuBarExtra (native macOS menu bar)
│
├─ DaemonClient (JSON-RPC communication)
│  ├─ startDaemonIfNeeded()
│  ├─ fetchEnvironmentStatus()
│  ├─ startEnvironment()
│  ├─ stopEnvironment()
│  ├─ createEnvironment()
│  └─ openShell()
│
├─ MenuBarView (UI container)
│  ├─ Status indicator
│  ├─ Environment list
│  ├─ Quick action buttons
│  └─ Preferences/Quit
│
├─ EnvironmentRow (Environment status card)
│  ├─ Status dot (green/gray)
│  ├─ Name + resources
│  └─ Action buttons (play/stop/shell)
│
├─ OnboardingSheet (5-step wizard)
│  ├─ WelcomeStepView
│  ├─ TemplateStepView
│  ├─ ResourcesStepView
│  ├─ NameStepView
│  └─ ReviewStepView
│
└─ VMEnvironment (data model)
```

### Key Design Decisions

1. **JSON-RPC over HTTP**
   - Menu bar talks to daemon via HTTP (not Unix socket)
   - Easier for GUI apps to communicate
   - Daemon needs `/tinybridge` HTTP endpoint
   - Fallback: starts daemon if not running

2. **SwiftUI Only**
   - No storyboards
   - No XIB files
   - Pure code = easier to version control
   - Native macOS controls (MenuBarExtra, Slider, etc.)

3. **Minimal Daemon Changes**
   - Daemon already handles JSON-RPC
   - Just needs HTTP wrapper
   - Menu bar is just another client
   - No architectural complexity

## Integration with Daemon

### Required: HTTP Endpoint

The daemon currently uses Unix socket. Add HTTP endpoint:

```rust
// In tinybridged/src/main.rs

// Start HTTP server alongside socket
#[tokio::main]
async fn main() -> Result<()> {
    // Existing code...
    
    // New: HTTP server for GUI
    tokio::spawn(async {
        start_http_server(7890).await
    });
    
    // Existing socket server
    daemon::run(socket).await
}

async fn start_http_server(port: u16) -> Result<()> {
    use axum::{routing::post, Router, Json};
    
    let app = Router::new()
        .route("/tinybridge", post(handle_rpc_request));
    
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_rpc_request(
    Json(request): Json<JsonRpcRequest>
) -> Json<JsonRpcResponse> {
    // Process request like DDS handler does
    // Return JSON-RPC response
}
```

**Effort**: 100-150 LOC Rust

### Testing Connection

```bash
# Start daemon manually
tinybridged --socket /tmp/tinybridge.sock

# Test HTTP endpoint
curl -X POST http://127.0.0.1:7890/tinybridge \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"environment.list","id":1}'

# Should return:
# {"jsonrpc":"2.0","result":[...],"id":1}
```

## Building DMG Installer

### Create DMG Package

```bash
#!/bin/bash

# 1. Build app
swift build -c release

# 2. Create app bundle structure
mkdir -p dist/TinyBridge.app/Contents/{MacOS,Resources}

# 3. Copy binary
cp .build/release/TinyBridgeApp dist/TinyBridge.app/Contents/MacOS/

# 4. Create Info.plist
cat > dist/TinyBridge.app/Contents/Info.plist <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>TinyBridgeApp</string>
    <key>CFBundleIdentifier</key>
    <string>com.tinybridge.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>TinyBridge</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.4.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>NSUserNotificationAlertStyle</key>
    <string>alert</string>
</dict>
</plist>
EOF

# 5. Create DMG
hdiutil create -volname "TinyBridge" \
  -srcfolder dist \
  -ov -format UDZO \
  TinyBridge-0.4.0.dmg

echo "✅ DMG created: TinyBridge-0.4.0.dmg"
```

### DMG Contents

```
TinyBridge-0.4.0.dmg
├─ TinyBridge.app (Menu bar app)
├─ Install.sh (Installation script)
├─ ReadMe.rtf (Quick start guide)
└─ LaunchAgent.plist (Auto-start configuration)
```

### Installation Script

```bash
#!/bin/bash
# Install.sh - Run after mounting DMG

set -e

# Install app
sudo cp -r TinyBridge.app /Applications/

# Install binaries (if not via Homebrew)
sudo cp tinybridge /usr/local/bin/
sudo cp tinybridged /usr/local/bin/

# Make executable
chmod +x /usr/local/bin/tinybridge
chmod +x /usr/local/bin/tinybridged

# Install LaunchAgent for daemon auto-start
mkdir -p ~/Library/LaunchAgents
cp com.tinybridge.daemon.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.tinybridge.daemon.plist

echo "✅ TinyBridge installed successfully!"
echo "Launch from Applications or Spotlight search 'TinyBridge'"
```

## LaunchAgent Configuration

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

## Development Workflow

### Run in Xcode

```bash
# Generate project
cd crates/tinybridge-macos
swift package generate-xcodeproj
open TinyBridgeApp.xcodeproj

# In Xcode:
# 1. Select "My Mac" as target
# 2. Press Cmd+R to run
# 3. Menu bar icon appears at top right
# 4. Click to open menu
```

### Debug Mode

```bash
# Build with debug symbols
swift build -v

# Run with LLDB debugger
lldb .build/debug/TinyBridgeApp

(lldb) run
(lldb) b main  # Set breakpoint
(lldb) po client.environments  # Print object
```

### Hot Reload During Development

While developing UI, you can:
1. Make code changes
2. Kill app (Cmd+Q or menu → Quit)
3. Rebuild: `swift build`
4. Run: `.build/debug/TinyBridgeApp`

This is faster than Xcode for iterative UI development.

## Troubleshooting

### App doesn't appear in menu bar
```bash
# Check if running
pgrep TinyBridgeApp

# Check console output
# Menu bar apps may not show console, use log:
log stream --predicate 'process == "TinyBridgeApp"'
```

### Daemon connection fails
```bash
# Check daemon running
lsof -i :7890

# Check socket exists
ls -la /tmp/tinybridge.sock

# Start daemon manually
/usr/local/bin/tinybridged --socket /tmp/tinybridge.sock

# Test HTTP endpoint
curl -X POST http://127.0.0.1:7890/tinybridge \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"environment.list","id":1}'
```

### Menu bar icon stuck
```bash
# Force quit
killall TinyBridgeApp

# Or via Activity Monitor
# Find "TinyBridgeApp" → Force Quit
```

### Permissions issues
```bash
# If `/usr/local/bin` doesn't exist
sudo mkdir -p /usr/local/bin

# Make files executable
chmod +x /usr/local/bin/tinybridge
chmod +x /usr/local/bin/tinybridged

# Or install via Homebrew (recommended)
brew tap tinybridge/tinybridge
brew install tinybridge
```

## Release Checklist

Before shipping:

- [ ] `swift build -c release` succeeds
- [ ] Menu bar appears
- [ ] Daemon auto-starts
- [ ] Can create environment
- [ ] Can start/stop environment
- [ ] Can open shell
- [ ] Notifications appear
- [ ] Quit works
- [ ] DMG builds
- [ ] DMG installs cleanly
- [ ] Uninstall leaves no trace

## Next Steps

After menu bar MVP (Phase 4.0.4):
- [ ] Add Spotlight integration (Phase 4.0.5)
- [ ] Add Finder quick actions
- [ ] Add System Preferences pane
- [ ] Add Keychain integration
- [ ] Add auto-update support

See `MACOS_UX_RESEARCH.md` for full roadmap.
