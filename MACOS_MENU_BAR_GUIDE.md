# TinyBridge Menu Bar App - Implementation Guide

## Quick Reference: Menu Bar App Goals

Transform TinyBridge from CLI-only to native macOS app with:
1. **Status visibility** - See running environments at a glance
2. **Quick actions** - Launch/stop VMs without Terminal
3. **Onboarding** - Guided first-run experience
4. **Notifications** - User feedback on completion
5. **Zero friction** - Works immediately after install

---

## Architecture Overview

### Current Stack
```
CLI (tinybridge) → Daemon (tinybridged) ← Commands
                        ↓
                    Device Manager
                    Policy Engine
                    Audit Logger
```

### With Menu Bar App
```
Menu Bar App (SwiftUI)
    ↓ JSON-RPC
Daemon (tinybridged) ← CLI still works
    ↓
Device Manager, Policy, Audit
```

**Key insight**: Menu Bar is just another JSON-RPC client (like CLI)

---

## Implementation Plan (Week 1)

### Day 1-2: Project Setup
**Create macOS app target in Cargo workspace**

```bash
cd /tmp/tinybridge

# Create Swift package for menu bar app
mkdir -p crates/tinybridge-macos
cd crates/tinybridge-macos

# Create package structure
swift package init --type executable --name TinyBridgeApp

# Create Xcode project for easier development
swift package generate-xcodeproj
```

### Day 2-3: Menu Bar Skeleton
**Build basic menu bar functionality**

```swift
// crates/tinybridge-macos/Sources/TinyBridgeApp/main.swift

import SwiftUI
import AppKit

@main
struct TinyBridgeApp: App {
    @StateObject var daemonClient = DaemonClient()
    
    var body: some Scene {
        MenuBarExtra("TinyBridge", systemImage: "desktopcomputer") {
            MenuBarView(client: daemonClient)
        }
        .windowStyle(.hiddenTitleBar)
    }
}

// MARK: - Menu Bar View
struct MenuBarView: View {
    @ObservedObject var client: DaemonClient
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Header
            Text("TinyBridge")
                .font(.headline)
            
            Divider()
            
            // Environment List
            if client.environments.isEmpty {
                Text("No environments yet")
                    .font(.caption)
                    .foregroundColor(.secondary)
            } else {
                ForEach(client.environments) { env in
                    EnvironmentRow(environment: env, client: client)
                }
            }
            
            Divider()
            
            // Quick Actions
            Button(action: { showNewEnvironmentSheet = true }) {
                Label("New Environment", systemImage: "plus.circle")
            }
            
            Button(action: { openPreferences() }) {
                Label("Preferences", systemImage: "gear")
            }
            
            Divider()
            
            Button(action: { NSApplication.shared.terminate(nil) }) {
                Label("Quit TinyBridge", systemImage: "power")
            }
        }
        .padding()
        .frame(minWidth: 280)
        .onAppear { client.startMonitoring() }
    }
}

// MARK: - Environment Row
struct EnvironmentRow: View {
    let environment: VMEnvironment
    @ObservedObject var client: DaemonClient
    
    var body: some View {
        HStack(spacing: 12) {
            // Status indicator
            Circle()
                .fill(environment.running ? Color.green : Color.gray)
                .frame(width: 8, height: 8)
            
            // Environment name
            VStack(alignment: .leading, spacing: 2) {
                Text(environment.name)
                    .font(.body)
                    .fontWeight(.medium)
                
                Text(environment.running ? 
                    "Running • \(environment.cpuCores) cores, \(environment.memory)GB" :
                    "Stopped")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Spacer()
            
            // Actions
            if environment.running {
                Button(action: { client.stopEnvironment(environment.name) }) {
                    Image(systemName: "stop.fill")
                        .foregroundColor(.red)
                }
                .buttonStyle(.plain)
                .help("Stop environment")
                
                Button(action: { client.openShell(environment.name) }) {
                    Image(systemName: "terminal.fill")
                        .foregroundColor(.blue)
                }
                .buttonStyle(.plain)
                .help("Open shell")
            } else {
                Button(action: { client.startEnvironment(environment.name) }) {
                    Image(systemName: "play.fill")
                        .foregroundColor(.green)
                }
                .buttonStyle(.plain)
                .help("Start environment")
            }
        }
        .padding(.vertical, 4)
        .contextMenu {
            Button("View Resources") { }
            Button("Edit Configuration") { }
            Button("View Logs") { }
            Button("Delete...") { }
        }
    }
}

// MARK: - Daemon Communication
@MainActor
class DaemonClient: NSObject, ObservableObject {
    @Published var environments: [VMEnvironment] = []
    @Published var isConnected = false
    
    private var socket: URLSessionWebSocketTask?
    private var monitoringTimer: Timer?
    
    func startMonitoring() {
        // Connect to daemon via JSON-RPC
        guard let socket = URLSession.shared.webSocketTask(
            with: URL(string: "http://localhost:7890/tinybridge")!
        ) else { return }
        
        self.socket = socket
        socket.resume()
        
        // Poll daemon for status
        monitoringTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            self?.fetchEnvironmentStatus()
        }
    }
    
    func fetchEnvironmentStatus() {
        // Send JSON-RPC request to daemon
        let request = """
        {
            "jsonrpc": "2.0",
            "method": "environment.list",
            "params": {},
            "id": 1
        }
        """
        
        // Parse response and update @Published environments
    }
    
    func startEnvironment(_ name: String) {
        let request = """
        {
            "jsonrpc": "2.0",
            "method": "environment.up",
            "params": {"name": "\(name)"},
            "id": 1
        }
        """
        // Send to daemon
    }
    
    func stopEnvironment(_ name: String) {
        let request = """
        {
            "jsonrpc": "2.0",
            "method": "environment.down",
            "params": {"name": "\(name)"},
            "id": 1
        }
        """
        // Send to daemon
    }
    
    func openShell(_ name: String) {
        // Launch Terminal.app with: tinybridge shell <name>
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/usr/bin/open")
        task.arguments = ["-a", "Terminal", "--args", "tinybridge", "shell", name]
        try? task.run()
    }
}

// MARK: - Data Models
struct VMEnvironment: Identifiable {
    let id: String
    let name: String
    let running: Bool
    let cpuCores: Int
    let memory: String
    let uptime: Int
}
```

### Day 3-4: Onboarding Wizard
**New environment creation flow**

```swift
// crates/tinybridge-macos/Sources/TinyBridgeApp/Onboarding/OnboardingSheet.swift

import SwiftUI

struct OnboardingSheet: View {
    @ObservedObject var client: DaemonClient
    @Environment(\.dismiss) var dismiss
    
    @State var currentStep = 0
    @State var selectedTemplate = "python"
    @State var environmentName = ""
    @State var cpuCores = 4
    @State var memory = 8
    
    var body: some View {
        VStack {
            if currentStep == 0 {
                WelcomeView()
            } else if currentStep == 1 {
                TemplateSelectionView(selected: $selectedTemplate)
            } else if currentStep == 2 {
                ResourceConfiguratorView(cores: $cpuCores, memory: $memory)
            } else if currentStep == 3 {
                EnvironmentNameView(name: $environmentName)
            } else if currentStep == 4 {
                ReviewView(
                    template: selectedTemplate,
                    name: environmentName,
                    cores: cpuCores,
                    memory: memory
                )
            }
            
            Spacer()
            
            HStack {
                if currentStep > 0 {
                    Button("Back") { currentStep -= 1 }
                }
                Spacer()
                if currentStep < 4 {
                    Button("Next") { currentStep += 1 }
                        .keyboardShortcut(.return, modifiers: [])
                } else {
                    Button(action: {
                        client.createEnvironment(
                            name: environmentName,
                            template: selectedTemplate,
                            cores: cpuCores,
                            memory: memory
                        )
                        dismiss()
                    }) {
                        Text("Create")
                            .frame(minWidth: 100)
                    }
                    .keyboardShortcut(.return, modifiers: [])
                }
            }
            .padding()
        }
        .frame(minWidth: 500, minHeight: 400)
    }
}

// Template Selection
struct TemplateSelectionView: View {
    @Binding var selected: String
    
    let templates = [
        ("python", "🐍 Python", "Python 3.11 + NumPy + Pandas"),
        ("rust", "🦀 Rust", "Rust 1.70 + Cargo + common crates"),
        ("ros2", "🤖 ROS 2", "ROS 2 Humble + DDS networking"),
        ("ml", "🧠 Machine Learning", "PyTorch + Jupyter + CUDA ready"),
    ]
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            Text("Select Template")
                .font(.title2)
                .fontWeight(.bold)
            
            Text("Choose a pre-configured environment")
                .foregroundColor(.secondary)
            
            VStack(spacing: 12) {
                ForEach(templates, id: \.0) { id, emoji, description in
                    VStack(alignment: .leading) {
                        HStack {
                            Text(emoji)
                                .font(.title)
                            VStack(alignment: .leading) {
                                Text(id.capitalized)
                                    .fontWeight(.semibold)
                                Text(description)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                        .padding()
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(selected == id ? Color.blue.opacity(0.1) : Color.gray.opacity(0.05))
                        .cornerRadius(8)
                        .overlay(
                            RoundedRectangle(cornerRadius: 8)
                                .stroke(selected == id ? Color.blue : Color.clear, lineWidth: 2)
                        )
                    }
                    .onTapGesture { selected = id }
                }
            }
        }
        .padding()
    }
}

// Resource Configuration
struct ResourceConfiguratorView: View {
    @Binding var cores: Int
    @Binding var memory: Int
    
    var recommendedMemory: Int {
        cores * 2
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 20) {
            Text("Configure Resources")
                .font(.title2)
                .fontWeight(.bold)
            
            VStack(alignment: .leading, spacing: 12) {
                Text("CPU Cores: \(cores)")
                    .fontWeight(.semibold)
                Slider(value: .init(get: { Double(cores) }, set: { cores = Int($0) }),
                       in: 1...8,
                       step: 1)
                Text("System has 8 cores available")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            VStack(alignment: .leading, spacing: 12) {
                Text("Memory: \(memory)GB")
                    .fontWeight(.semibold)
                Slider(value: .init(get: { Double(memory) }, set: { memory = Int($0) }),
                       in: 1...16,
                       step: 1)
                Text("Recommended: \(recommendedMemory)GB • System has 16GB available")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding()
    }
}
```

### Day 4-5: Daemon Integration
**Wire menu bar to daemon**

Key integration points:
```
Menu Bar → HTTP/JSON-RPC → Daemon Socket
   ↓                         ↓
Status requests         Process requests
Live monitoring          Device allocation
                         Policy enforcement
```

### Day 5: Polish & Testing
- Error handling
- Notification support
- Keyboard shortcuts (⌘K for new env, ⌘Q to quit)
- Accessibility labels

---

## Integration with Existing Daemon

### No Changes Required to Daemon!
Menu bar app is a JSON-RPC client, just like CLI:

```swift
// Similar to how CLI sends JSON-RPC
let request = """
{
    "jsonrpc": "2.0",
    "method": "environment.list",
    "id": 1
}
"""

// Daemon already handles this via DaemonServer
// Just responds with environment list
```

### Required: HTTP/JSON-RPC Endpoint
Currently daemon uses Unix socket. Consider adding:

```rust
// In tinybridged/src/server.rs
pub async fn start_http_server(port: u16) -> Result<()> {
    let app = Router::new()
        .route("/tinybridge", post(handle_json_rpc))
        .with_state(Arc::new(daemon_state));
    
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_json_rpc(
    State(state): State<Arc<DaemonState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    // Process request, return response
}
```

**Effort**: 200 LOC Rust

---

## Deployment: DMG Installer

### Build DMG Package
```bash
# Build release binary
cargo build --release --bin tinybridge --bin tinybridged

# Build macOS app
cd crates/tinybridge-macos
swift build -c release

# Create .dmg
# Resources/create-dmg.sh handles:
# - Code signing
# - Notarization
# - DMG creation with nice UI
```

### DMG Contents
```
TinyBridge.dmg
├─ TinyBridge.app (Menu bar app)
├─ tinybridge (CLI binary)
├─ tinybridged (Daemon binary)
├─ Installation script
└─ ReadMe.rtf
```

### Installer Script
```bash
#!/bin/bash

# 1. Copy binaries to /usr/local/bin or /opt/tinybridge
# 2. Install LaunchAgent for auto-start daemon
# 3. Create symlinks for CLI
# 4. Launch menu bar app
# 5. Show welcome notification
```

---

## User Experience Flow

### First Launch
```
1. Download TinyBridge.dmg
2. Drag TinyBridge.app to Applications
3. Open TinyBridge (from Applications or Launchpad)
4. Daemon auto-starts (LaunchAgent)
5. Menu bar icon appears
6. Welcome notification: "Click to get started"
7. Click notification → Onboarding wizard
8. Create first environment (Python template, 4 cores, 8GB)
9. Click "Launch" → Environment starts
10. Menu bar shows: ✅ myenv (Running, 4 cores, 8GB)
11. Click "Shell" → Terminal opens with `tinybridge shell myenv`
```

**Time from download to working environment: 90 seconds**

### Regular Use
```
1. User wants to work on ML project
2. Clicks menu bar TinyBridge icon
3. Sees "ml-training" in list (Running)
4. Clicks "Shell" button
5. Terminal opens in environment
6. Works on code
```

**Zero Terminal commands needed**

---

## Metrics: Before vs After

| Metric | CLI-Only | With Menu Bar |
|--------|----------|--------------|
| Onboarding time | 5+ minutes | 90 seconds |
| Terminal required | Yes | No |
| Visual feedback | Minimal | Rich |
| Quick access | `tinybridge` commands | Menu bar icon |
| Error clarity | Technical | User-friendly |
| Resource visibility | `tinybridge status` | Menu bar shows live |

---

## Summary: Menu Bar MVP (Week 1)

### Deliverables
- ✅ Menu bar app (SwiftUI)
- ✅ Environment status view
- ✅ Quick launch/stop actions
- ✅ Onboarding wizard (4 steps)
- ✅ Native notifications
- ✅ Shell launcher
- ✅ Preferences shortcut
- ✅ DMG installer

### Code Locations
```
crates/tinybridge-macos/
├─ Sources/
│  ├─ main.swift (Entry point)
│  ├─ MenuBarApp/ (Menu bar views)
│  ├─ Onboarding/ (Wizard views)
│  ├─ DaemonClient.swift (JSON-RPC client)
│  └─ Models/ (Data structures)
└─ Package.swift
```

### Dependencies
- SwiftUI (built-in)
- AppKit (built-in)
- URLSession (built-in)
- No external dependencies needed!

### Build & Ship
```bash
# Build
swift build -c release

# Test locally
open .build/release/TinyBridgeApp.app

# Sign & notarize for distribution
# Create DMG package
# Upload to GitHub releases
```

---

## Success Criteria

After menu bar launch:
- ✅ Zero Terminal commands for new user setup
- ✅ Visual VM status always visible
- ✅ 90-second time to first environment
- ✅ "Works like Docker Desktop" user feedback
- ✅ 80%+ of actions available from menu bar

Next: Spotlight integration (Phase 4.0.5)
