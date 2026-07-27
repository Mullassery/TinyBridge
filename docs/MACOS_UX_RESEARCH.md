# TinyBridge macOS User Experience Research

## Current State
- CLI-first tool (typed commands)
- JSON-RPC daemon (background service)
- Keyboard-based interactive mode
- Terminal output
- Manual env.yaml editing

## macOS User Expectations
macOS users expect:
1. **Native GUI** - System Preferences, Menu Bar app, native dialogs
2. **Zero Configuration** - Works out of the box
3. **Visual Feedback** - Spinners, progress bars, notifications
4. **Spotlight Integration** - Quick search + launch
5. **Finder Integration** - Drag-and-drop, right-click context menu
6. **System Integration** - Auto-start, system tray, menubar
7. **Consistency** - Follows macOS design language
8. **Discoverability** - Help, onboarding, tooltips

---

## Recommended macOS UX Improvements

### Tier 1: Essential (High Impact, Medium Effort)

#### 1. Menu Bar App
**Problem**: Users forget daemon is running; no quick access  
**Solution**: Menu bar application showing:
- VM status (running/stopped)
- Quick launch buttons
- Real-time resource usage
- Recent commands

**Components**:
```
TinyBridge Menu Bar
├─ Status
│  └─ robotics-sim: ▶️ Running (4 cores, 8GB)
│  └─ ml-training: ⏸️ Stopped
├─ Quick Actions
│  ├─ Launch robotics-sim
│  ├─ Open Shell
│  └─ View Logs
├─ Settings
│  └─ Preferences...
└─ Quit
```

**Implementation**: SwiftUI with AppKit (NSStatusBar)  
**Effort**: 300 LOC (Swift)

#### 2. Onboarding Wizard
**Problem**: New users don't know where to start  
**Solution**: First-run interactive wizard
- Welcome screen
- Template selection (Rust, Python, ROS 2, etc.)
- Environment name input
- Resource allocation (CPU/memory sliders)
- Review & create

**Components**:
- Welcome view (SwiftUI)
- Template picker (Grid)
- Resource configurator (Sliders)
- Terminal auto-open

**Effort**: 400 LOC (Swift)

#### 3. Native Notifications
**Problem**: No user feedback on completion  
**Solution**: Use macOS Notification Center
- VM started/stopped
- Shell ready
- Errors with actionable suggestions
- Resource warnings (low disk, high memory)

**Implementation**: UserNotificationCenter  
**Effort**: 100 LOC (Swift)

#### 4. Spotlight Integration
**Problem**: Users must open Terminal to use TinyBridge  
**Solution**: Add to Spotlight search
- Search "launch robotics"
- Search "status ml-training"
- Quick actions appear in Spotlight

**Implementation**: Spotlight plugin or CLI shortcuts  
**Effort**: 150 LOC (Swift)

#### 5. Finder Integration
**Problem**: Users can't drag env.yaml to create environments  
**Solution**: Add Finder right-click context menu
- "Open with TinyBridge"
- Auto-creates environment from env.yaml

**Implementation**: Finder Quick Actions  
**Effort**: 200 LOC (Swift)

---

### Tier 2: Polish (Medium Impact, Low-Medium Effort)

#### 6. System Preferences Pane
**Problem**: Configuration buried in CLI  
**Solution**: Native System Preferences integration
- Default resource allocation
- Auto-start settings
- Networking options (DDS domain, port ranges)
- Performance tuning
- Logging level

**Effort**: 350 LOC (Swift)

#### 7. Smart Terminal Integration
**Problem**: Terminal output is technical/cryptic  
**Solution**: Colorized, emoji-enhanced output
```
✅ Environment ml-training started
   • Boot time: 2.3s
   • SSH ready on port 2222
   • 4 cores, 8GB memory allocated
```

**Effort**: 100 LOC (Rust - crossterm)

#### 8. Keychain Integration
**Problem**: Users can't save credentials safely  
**Solution**: Store SSH keys, Docker credentials in Keychain
- Auto-login to environments
- Secure credential storage
- Per-environment secrets

**Implementation**: Security framework  
**Effort**: 200 LOC (Swift)

#### 9. Activity Monitor Integration
**Problem**: Can't see resource usage  
**Solution**: Register with Activity Monitor
- Show CPU/memory per environment
- Appear in Activity Monitor's process list

**Effort**: 150 LOC (Swift)

#### 10. Drag-and-Drop env.yaml
**Problem**: Manual file editing  
**Solution**: Drag env.yaml onto Menu Bar app
- Auto-creates environment
- Visual validation (shows what will be created)
- One-click launch

**Effort**: 150 LOC (Swift)

---

### Tier 3: Advanced (Lower Priority)

#### 11. SwiftUI Settings App
**Problem**: Preferences scattered  
**Solution**: Beautiful native Settings window
- Tabbed interface
- Real-time validation
- Preview of changes

**Effort**: 300 LOC (Swift)

#### 12. Dock Integration
**Problem**: App disappears from Dock  
**Solution**: Persistent Dock icon with menu
- Shows active environments
- Quick launch from Dock

**Effort**: 100 LOC (Swift)

#### 13. Quick Look Preview
**Problem**: Can't preview env.yaml before opening  
**Solution**: macOS Quick Look plugin
- Preview YAML structure
- Show resource requirements
- Display template info

**Effort**: 200 LOC (Swift)

#### 14. Accessibility (VoiceOver)
**Problem**: Blind users can't use TinyBridge  
**Solution**: Full VoiceOver support
- Descriptive labels
- Navigation hints
- Screen reader friendly

**Effort**: 150 LOC (Swift)

#### 15. Continuous Auto-Update
**Problem**: Manual `brew upgrade` required  
**Solution**: Built-in auto-update (Sparkle framework)
- Background checks
- Gentle notifications
- One-click update

**Effort**: 100 LOC (Swift)

---

## UX Flow: New User (Current vs. Improved)

### Current Flow (CLI-First)
```
1. User opens Terminal
2. Types: brew install tinybridge
3. Creates env.yaml in text editor
4. Types: tinybridge up myproject
5. Types: tinybridge shell myproject
6. Edits environment later via CLI
```
**Pain Points**: Terminal knowledge required, no visual feedback

### Improved Flow (GUI + CLI)
```
1. User downloads .dmg
2. Drags TinyBridge to Applications
3. Launches TinyBridge → Welcomes them
4. Wizard opens → Select template (Rust)
5. Click "Create" → Environment appears in menu bar
6. Click "Launch" → Progress spinner
7. Click "Shell" → Terminal auto-opens
8. Menu bar shows: ✅ Running (4 cores, 8GB)
```
**Benefits**: No Terminal knowledge, visual feedback, fast

---

## Recommended Implementation Path

### Phase 4.0.4: macOS UX Foundation (3 weeks)
**Focus: Menu bar + onboarding wizard**

Priority order:
1. Menu bar app (400 LOC Swift) - **Most visible**
2. Onboarding wizard (400 LOC Swift) - **Removes friction**
3. Native notifications (100 LOC) - **Feedback**
4. Smart Terminal output (100 LOC Rust) - **Polish**

**Total**: ~1,000 LOC Swift, 100 LOC Rust  
**Effort**: 3 weeks (1 person)

### Phase 4.0.5: System Integration (2 weeks)
**Focus: Spotlight, Finder, Preferences**

1. Spotlight integration (150 LOC)
2. Finder quick actions (200 LOC)
3. System Preferences pane (350 LOC)
4. Keychain integration (200 LOC)

**Total**: ~900 LOC Swift  
**Effort**: 2 weeks

### Phase 4.0.6: Polish (1 week)
**Focus: Accessibility + Auto-update**

1. VoiceOver support (150 LOC)
2. Sparkle auto-update (100 LOC)
3. Dock integration (100 LOC)

**Total**: ~350 LOC Swift

---

## Technical Architecture for macOS UX

### New Module Structure
```
tinybridge-macos/
├─ Sources/
│  ├─ TinyBridgeApp.swift (Main app entry)
│  ├─ MenuBarApp/
│  │  ├─ MenuBarManager.swift
│  │  ├─ StatusMenuController.swift
│  │  └─ VMStatusView.swift
│  ├─ Onboarding/
│  │  ├─ WelcomeView.swift
│  │  ├─ TemplateSelector.swift
│  │  ├─ ResourceConfigurator.swift
│  │  └─ ReviewView.swift
│  ├─ Preferences/
│  │  ├─ PreferencesWindow.swift
│  │  ├─ GeneralPreferences.swift
│  │  └─ AdvancedPreferences.swift
│  ├─ Integration/
│  │  ├─ SpotlightIntegration.swift
│  │  ├─ FinderIntegration.swift
│  │  ├─ NotificationCenter.swift
│  │  └─ KeychainHelper.swift
│  └─ Utils/
│     ├─ DaemonManager.swift
│     ├─ FileWatcher.swift
│     └─ TerminalHelper.swift
└─ Resources/
   └─ Assets.xcassets
```

### Communication Flow
```
┌─────────────────────────┐
│  macOS GUI (SwiftUI)    │
│  • Menu Bar             │
│  • Preferences          │
│  • Wizard               │
└────────────┬────────────┘
             │ JSON-RPC
             ↓
┌─────────────────────────┐
│  Daemon (tinybridged)   │
│  • Device management    │
│  • Policy engine        │
│  • Audit logging        │
└─────────────────────────┘
```

### Daemon Enhancement for GUI
```rust
// Add to daemon for GUI support
pub struct UIManager {
    /// Notify GUI of VM status changes
    pub fn notify_status_change(env: &str, status: VMStatus) {
        // Send OSC or UNS notification
    }
    
    /// Get real-time stats for menu bar
    pub fn get_environment_stats(env: &str) -> EnvironmentStats {
        // Boot time, CPU %, memory %, uptime
    }
    
    /// Provide GUI-friendly error messages
    pub fn get_user_friendly_error(error: &BridgeError) -> String {
        // "Low disk space - only 2GB remaining"
        // vs "ENOSPC: No space left on device"
    }
}
```

---

## User Research Questions

Before building, validate with actual users:

1. **Installation**
   - Do users prefer Homebrew or GUI installer (.dmg)?
   - How many users know about Homebrew?

2. **Discovery**
   - Where do users look first? (Dock, Spotlight, Applications folder?)
   - Do they expect an app icon or just CLI?

3. **Mental Model**
   - Do users think of "environments" as files or applications?
   - Should VMs appear in Activity Monitor?

4. **Frequency of Use**
   - How often do users create new environments? (Daily? Weekly?)
   - How often do they switch between environments?

5. **Preferences**
   - Would users prefer menu bar or window always visible?
   - How much detail in notifications vs minimal?

---

## Quick Win: Menu Bar MVP (Week 1)

Minimum viable product - highest user impact:

```swift
// MenuBarApp.swift
import SwiftUI

@main
struct TinyBridgeApp: App {
    @StateObject var vmMonitor = VMStatusMonitor()
    
    var body: some Scene {
        MenuBarExtra("TinyBridge", systemImage: "desktopcomputer") {
            VStack {
                // Active VMs
                ForEach(vmMonitor.environments) { env in
                    HStack {
                        Circle()
                            .fill(env.running ? .green : .gray)
                            .frame(width: 8)
                        Text(env.name)
                        Spacer()
                        Text(env.running ? "Running" : "Stopped")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding(.vertical, 4)
                }
                
                Divider()
                
                Button("New Environment...") {
                    showOnboarding = true
                }
                
                Button("Quit") {
                    NSApplication.shared.terminate(nil)
                }
            }
            .padding()
            .frame(width: 300)
        }
    }
}
```

**Provides**:
- Quick status overview
- Launch/stop actions
- Create new environment
- Clean quit

**Time to implement**: 2-3 days  
**Impact**: Users see VM status at a glance

---

## Success Metrics

After implementing macOS UX improvements:

| Metric | Before | Target |
|--------|--------|--------|
| Time to first VM | 5 min | 60 sec |
| Terminal required | Yes | No |
| Discoverability | Via docs | Spotlight search |
| Error clarity | Technical | User-friendly |
| Visual feedback | Minimal | Rich (spinners, notifications) |
| Menu bar awareness | None | Always visible |

---

## Competitive Analysis

### Docker Desktop
✅ Native menu bar with status  
✅ Preferences window  
✅ One-click actions  
❌ Resource-heavy  
❌ Requires Docker Hub account

### OrbStack
✅ Fast boot  
✅ Minimal UI (menu bar only)  
❌ Limited customization  

### Lima
❌ CLI-only  
❌ No menu bar  
❌ Requires manual configuration  

**TinyBridge Opportunity**: Combine Docker Desktop's UX polish with TinyBridge's lightweight efficiency

---

## Summary: macOS UX Roadmap

### Phase 4.0.4 (3 weeks) - Core UX
- [ ] Menu bar app with VM status
- [ ] Onboarding wizard
- [ ] Native notifications
- [ ] Smart terminal output

### Phase 4.0.5 (2 weeks) - Integration
- [ ] Spotlight search
- [ ] Finder context menu
- [ ] System Preferences pane
- [ ] Keychain support

### Phase 4.0.6 (1 week) - Polish
- [ ] VoiceOver accessibility
- [ ] Auto-update support
- [ ] Dock integration

### Total Effort: 6 weeks, ~2,250 LOC (Swift)

---

## Recommendation

**Start with Menu Bar App** (Phase 4.0.4 Week 1)

Why:
1. **Highest impact** - Most visible to users
2. **Quick win** - Implementable in 3-5 days
3. **Foundation** - Other features build on it
4. **Safe** - Daemon unchanged, just adds GUI layer
5. **Familiar** - Similar to Docker Desktop, users expect it

This single feature transforms TinyBridge from "CLI tool" to "native macOS application" in users' minds.
