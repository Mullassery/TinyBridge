# TinyBridge macOS Packaging Redesign — Phase 1 Complete

**Date:** 2026-07-26  
**Status:** Phase 1 (Foundation) — COMPLETE  
**Remaining:** Phase 2 (CI/CD), Phase 3 (Release/Distribution)

---

## What Was Completed (Phase 1)

### ✅ Rust Path/Config Refactoring (DONE)

1. **File:** `crates/tinybridge-core/src/config.rs`
   - Replaced `/var/run/tinybridge.sock` with `~/Library/Application Support/TinyBridge/tinybridge.sock` (user-writable)
   - Removed flat `~/.tinybridge` directory, split into Apple-convention paths:
     - `data_dir()` → `~/Library/Application Support/TinyBridge` (keys, identity, state)
     - `cache_dir()` → `~/Library/Caches/TinyBridge` (downloadable assets, VM images)
     - `logs_dir()` → `~/Library/Logs/TinyBridge` (log files)
   - Updated `kernel_path`/`initrd_path` to use `cache_dir()/assets/` (they're downloadable, not executables)

2. **File:** `crates/tinybridge-daemon/src/manager.rs`
   - Updated `assets_dir` to use `TinyBridgeConfig::cache_dir()`
   - Updated `keys_dir` to use `TinyBridgeConfig::data_dir()`

3. **File:** `crates/tinybridge-ssh/src/audit.rs`, `key_manager.rs`
   - Updated default paths to use `TinyBridgeConfig::data_dir()`
   - Added `tinybridge-core` dependency to `crates/tinybridge-ssh/Cargo.toml`

### ✅ Migration Module (DONE)

**File:** `crates/tinybridge-core/src/migration.rs`
- New idempotent migration function that runs on daemon startup
- Cleans up legacy `~/Library/LaunchDaemons/com.tinybridge.daemon.plist` (old formula's mislabeled plist)
- Migrates data from `~/.tinybridge` → new Apple-convention paths
- Non-fatal errors (logged as warnings, doesn't block daemon startup)
- Integrated into `crates/tinybridge-daemon/src/main.rs`

### ✅ Swift Menu-Bar App (MVP BUILT)

**Directory:** `swift/TinyBridgeApp/` (7 new files)
- `TinyBridgeApp.swift` — `@main App` with `MenuBarExtra` showing daemon status
- `AppState.swift` — ObservableObject managing environment list, polling daemon every 3s
- `MenuBarContentView.swift` — SwiftUI view with environment list
- `EnvironmentRowView.swift` — Individual row with Start/Stop/Open Shell buttons
- `CLIBridge.swift` — Shells out to bundled `tinybridge binary` CLI for `list --json`, `up`, `down`, `shell` commands
- `DaemonModels.swift` — Codable structs (`EnvironmentSummary`, `ListResponse`) mirroring CLI JSON wire format
- `DaemonLauncher.swift` — First-run LaunchAgent registration (reads template, substitutes `__HOME__`, bootstraps via `launchctl`)

**Key decisions:**
- CLI-based polling (process-spawn + JSON parsing) vs native socket client — MVP simplification, flagged as fast-follow
- "Open Shell" opens Terminal.app running `tinybridge shell <env>` via osascript
- `MenuBarExtra` + `LSUIElement=false` → both menu bar icon + Dock icon simultaneously

**Build status:** `swift build --package-path swift/ -c release` ✅ succeeds

### ✅ Packaging Assets (DONE)

1. **File:** `packaging/Info.plist.template`
   - Correct bundle identifier: `com.mullassery.tinybridge`
   - Correct executable: `TinyBridgeApp`
   - Category: `public.app-category.developer-tools`
   - LSMinimumSystemVersion: `13.0` (matches Package.swift)
   - LSUIElement: `false` (appears in Dock while running)
   - Icon file: `AppIcon`

2. **File:** `packaging/com.mullassery.tinybridge.daemon.plist`
   - Per-user LaunchAgent template (not root LaunchDaemon)
   - Label: `com.mullassery.tinybridge.daemon`
   - Logs to `~/Library/Logs/TinyBridge/daemon{,.err}.log`
   - Placeholder `__HOME__` substituted at install time

3. **File:** `packaging/generate-icon.sh` ✅ (DONE, runs successfully)
   - Generates 1024×1024 placeholder PNG programmatically (no external image deps)
   - Builds .iconset via `sips` at all standard macOS sizes
   - Outputs `AppIcon.icns` (21KB)
   - Can swap source PNG later for real branding art

4. **File:** `packaging/AppIcon.icns` ✅ (DONE, generated)

### ✅ App Bundle Assembly Script (DONE)

**File:** `packaging/build-app.sh`
- Builds universal Rust binaries (aarch64 + x86_64 via `lipo`)
- Builds Swift app
- Assembles bundle structure: `TinyBridge.app/Contents/{MacOS,Resources,Frameworks}`
- Fixes dylib install names via `install_name_tool` (reuses exact invocation from docs)
- Generates `Info.plist` from template (version substitution)
- Ad-hoc code-signs app
- Verified: script creates `dist/TinyBridge.app` locally

---

## Rust Compilation Status

✅ `cargo check --workspace` — zero errors, warnings only (pre-existing)
✅ All crates compile successfully
✅ Migration module integrated and tested

---

## Swift Compilation Status

✅ `swift build --package-path swift/ -c release` — builds to completion
✅ App executable: `.build/release/TinyBridgeApp`
✅ All type checking passed

---

## Still TODO (Phase 2-3)

### Phase 2: CI/CD & Release Pipeline
- [ ] `.github/workflows/release.yml` — codesign (Developer ID gated) + notarize (if secrets present) + create .dmg + .zip
- [ ] Homebrew Cask implementation: `Casks/tinybridge.rb` in tap repo
- [ ] Deprecation of `Formula/tinybridge.rb` and `Formula/tinybridged.rb`

### Phase 3: CLI Commands & Testing
- [ ] `tinybridge uninstall [--purge]` CLI subcommand
- [ ] `tinybridge daemon install` subcommand (manual LaunchAgent registration fallback)
- [ ] `packaging/validate-install.sh` (post-install verification script)
- [ ] Manual smoke testing on clean macOS VM

---

## File Inventory

### New Files Created
```
packaging/
  ├── Info.plist.template
  ├── com.mullassery.tinybridge.daemon.plist
  ├── generate-icon.sh (executable)
  ├── build-app.sh (executable)
  ├── AppIcon-source.png (placeholder, regenerated each run)
  └── AppIcon.icns (21KB, generated)

swift/
  └── TinyBridgeApp/
      ├── TinyBridgeApp.swift
      ├── AppState.swift
      ├── MenuBarContentView.swift
      ├── EnvironmentRowView.swift
      ├── CLIBridge.swift
      ├── DaemonModels.swift
      └── DaemonLauncher.swift

crates/tinybridge-core/
  └── src/
      └── migration.rs (new module)
```

### Modified Files
```
swift/Package.swift (added TinyBridgeApp executable target)
crates/tinybridge-core/src/{config.rs, lib.rs}
crates/tinybridge-daemon/src/{manager.rs, main.rs}
crates/tinybridge-ssh/src/{audit.rs, key_manager.rs}
crates/tinybridge-ssh/Cargo.toml (added tinybridge-core dependency)
crates/tinybridge-core/Cargo.toml (added anyhow, tracing dependencies)
```

---

## Key Decisions Made

1. **SwiftPM over Xcode project** — keeps build CI-compatible with `swift build`, single package, no separate .xcodeproj needed
2. **MVP UI scope** — minimal MenuBarExtra (no creation wizard, no in-app logs) to ship fast, CLI-bridge polling as temporary measure
3. **Ad-hoc signing first** — pipeline supports Developer ID secrets but works unsigned initially (users use right-click→Open)
4. **Migration in daemon** — runs on every startup, idempotent, catches all install paths (Cask, leftover Formula, manual)
5. **Self-registering LaunchAgent** — app registers itself on first launch, not Homebrew postinstall (more reliable)

---

## Testing Checklist (Local)

- [x] `cargo check --workspace` compiles
- [x] `swift build --package-path swift/ -c release` builds
- [x] `packaging/build-app.sh 0.1.0` creates `dist/TinyBridge.app`
- [x] App bundle has Info.plist, executable, icon, dylib
- [x] `codesign -v dist/TinyBridge.app` passes (ad-hoc)
- [ ] Copy to `/Applications`, double-click launch, verify menu bar + Dock icon appear
- [ ] `tinybridge list --json` returns valid JSON
- [ ] LaunchAgent registers on first app launch
- [ ] Environment list displays correctly in menu bar
- [ ] Start/Stop/Open Shell buttons work against a real daemon

---

## Next Steps (Phase 2-3, outside this session)

1. **Push Phase 1 commits** to `main` branch
2. **Create release.yml** with secret-gated codesign/notarize pipeline
3. **Update tap repo** with new Cask formula, deprecate old Formulas
4. **Manual validation** on clean macOS VM with ad-hoc build
5. **Beta test** with real Developer ID secrets (optional, future)

---

**Created by:** Claude Haiku 4.5  
**Plan file:** `/Users/georgimullassery/.claude/plans/parallel-cuddling-cray.md`
