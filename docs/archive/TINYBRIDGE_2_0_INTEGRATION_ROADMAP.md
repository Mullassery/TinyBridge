# TinyBridge 2.0 Integration Roadmap

**Status:** Planning  
**Date:** 2026-07-25  
**Vision:** Developer Experience First Architecture

---

## Executive Summary

TinyBridge transitions from **engineering-focused** (Phase 1-3: boot perf, OTel, compliance) to **developer-focused** (2.0: zero friction, opinionated defaults, intelligent diagnostics).

**Key Principle:** Incremental evolution, not rewrite. Phase 1-3 remains the performance/observability foundation. 2.0 adds the DX layer on top.

---

## Current State Assessment (Phase 1-3)

### What Works Well ✅
- **Boot optimization:** Multi-tier lazy loading (1.5s SSH, 5s usable, 120s complete)
- **Observability:** OTel framework ready, extensible backends (Prometheus, Jaeger, Datadog)
- **Architecture:** Clean modular crates, 80 tests passing, 100% CI passing
- **Foundation:** VM lifecycle, network management, device passthrough policy

### What's Missing for 2.0 ⚠️
- **CLI UX:** Current CLI is functional but not polished (no progress indicators, no helpful error messages)
- **Defaults:** Users must configure (CPU, RAM, disk, networking) — should be automatic
- **Diagnostics:** No health check system (`doctor` command doesn't exist)
- **Templates:** No pre-built environments (rust, python, ros2, etc.)
- **Error Handling:** Stack traces instead of actionable guidance
- [x] **Orphaned-process registry** — DONE. `tinybridge-core::pid_lock::PidLock` writes a `<socket_path>.pid` file and checks it on startup via `ps -p <pid>` (no new `unsafe`/libc dependency): a live owning process refuses cleanup and errors out instead of having its socket yanked; a dead one's stale socket+lock is cleaned up automatically (logged). Wired into both `daemon.rs` and `tinybridge-vmhost/socket_server.rs`, replacing their blind `remove_file` calls. `Drop` removes the lock file on clean shutdown.
- [x] **Runtime nested-virt/CPU-extension diagnostics** — DONE. `VirtualizationCheck::run` now also checks `sysctl kern.hv_support` at runtime (the same signal Hypervisor.framework/Virtualization.framework consult internally) — catches cases pure `cfg!(target_arch)` can't, e.g. running nested inside another hypervisor without virtualization passthrough, or virtualization disabled by an MDM profile. Reports `Fail` with a specific message when hardware virtualization is confirmed unavailable, regardless of architecture.
- Note: two other external-critique items were checked and don't apply — there's no tap/tun/bridge networking code at all (uses Apple's unprivileged `VZNATNetworkDeviceAttachment`, so no root-prompt/sandboxing issue), and there's no multi-hypervisor capability-flag problem today since only the Apple VZ backend (`tinybridge-vz`) actually ships — the Hyper-V/KVM adapter files are explicitly dead code per a doc comment in `macos_adapter.rs`.

---

## TinyBridge 2.0 Vision Goals

### 1. Zero-Friction Onboarding
**Goal:** `brew install tinybridge` → `tinybridge launch` → working Linux environment in <60 seconds

**Requirements:**
- Auto-detect system capabilities (CPU, RAM, storage, architecture)
- Download correct Linux image automatically
- Intelligent default allocation (CPU/RAM based on system, disk based on available space)
- Configure networking without user intervention
- Mount workspace directories automatically
- SSH ready immediately

### 2. Intelligent Defaults
**Goal:** 95%+ of users never need config files

**Mechanism:**
- Detect available memory/storage
- Profile-based allocation (dev, workstation, enterprise)
- Opinionated but overridable configurations
- Smart resource allocation based on host capacity

### 3. Comprehensive Diagnostics
**Goal:** `tinybridge doctor` → complete health report

**Coverage:**
- Virtualization capability check
- Resource availability (memory, disk, CPU)
- Networking health (connectivity, DNS, bridge config)
- Storage integrity (corruption detection)
- Guest health (SSH, services, resource consumption)
- Actionable recommendations for failures

### 4. Developer Templates
**Goal:** `tinybridge launch rust` → complete Rust dev environment

**Templates:**
- **Base:** Ubuntu 24.04 LTS, Fedora, Debian, Arch
- **Dev:** Rust, Python, Node.js, Go, C++
- **Robotics:** ROS2 Humble, ROS2 Jazzy, Nav2, Gazebo, RViz
- **AI/ML:** Jupyter, PyTorch, TensorFlow, LLM stacks
- **Enterprise:** Custom templates from org YAML

### 5. Exceptional Error Messages
**Goal:** Replace stack traces with guidance

**Pattern:**
```
Unable to start Linux environment.

Reason: Insufficient disk space.
Available: 3.2 GB | Required: 10 GB

Suggested Fix:
- Delete unused files (freed space will appear here)
- Increase storage allocation
- Use external SSD (path: /mnt/external)

Learn more:
  tinybridge doctor
  tinybridge logs --errors
```

---

## Architecture Integration Strategy

### Phase Approach
Rather than rewrite, add 2.0 layer incrementally:

```
Phase 1-3 (Current)          Phase 2.0 (Incremental)
├─ Boot Optimization          ├─ CLI Layer (new)
├─ OTel Infrastructure         ├─ UX Improvements (wrapped)
├─ Device Passthrough          ├─ Diagnostics (new)
├─ Compliance/OKF              ├─ Templates (new)
└─ RPC Daemon                  └─ Error Handling (enhanced)
     ↓
    v (wire together)
   
   TinyBridge 2.0 = 1.0 Core + 2.0 Experience Layer
```

### Component Mapping

| Current | 2.0 Layer | Status |
|---------|-----------|--------|
| `tinybridge-daemon` | RPC backend | Keep as-is (no breaking changes) |
| `tinybridge-cli` | New CLI UX | Redesign around 2.0 commands |
| `tinybridge-core` | Config + models | Extend for templates, defaults |
| `tinybridge-vz` | VM lifecycle | Keep as-is |
| (new) | Diagnostics system | Build new crate `tinybridge-diagnostics` |
| (new) | Template engine | Build new crate `tinybridge-templates` |
| (new) | Error formatter | Build new crate `tinybridge-errors` |

### Backward Compatibility
- Existing JSON-RPC daemon API unchanged
- Existing configs still work (v1 + v2 coexist)
- Default behavior becomes v2 (smart defaults), but v1 behavior available via flags
- Migration path clear and documented

---

## Phase-by-Phase Roadmap

### Phase 2.0.1: CLI Restructuring (4 weeks)
**Goal:** Establish new command structure and UX patterns

**Deliverables:**
1. New CLI command structure
   ```bash
   tinybridge launch [env-name] [template]      # New primary command
   tinybridge stop [env-name]
   tinybridge restart [env-name]
   tinybridge status [env-name]
   tinybridge logs [options]
   tinybridge doctor
   tinybridge destroy [env-name]
   tinybridge templates [search]
   tinybridge images [search]
   ```

2. Smart defaults engine
   - Auto-detect system specs (RAM, CPU, disk)
   - Calculate optimal allocation
   - Generate config with recommendations

3. Progress indicators
   - Spinner for long operations
   - Step-by-step progress (downloading image, creating VM, starting services)
   - ETA for operations

4. Tests: 20+ CLI integration tests

**Files to Create/Modify:**
- `crates/tinybridge-cli/src/main.rs` (restructure)
- `crates/tinybridge-cli/src/commands/` (new module)
- `crates/tinybridge-cli/src/config_generator.rs` (new - smart defaults)
- `crates/tinybridge-cli/src/progress.rs` (new - progress indicators)

**Acceptance Criteria:**
- All commands work without explicit config
- Help text is clear and actionable
- 20+ tests pass
- Backward compatible with v1.0 configs

---

### Phase 2.0.2: Diagnostics System (3 weeks)
**Goal:** Comprehensive health checking and recommendations

**Deliverables:**
1. Diagnostics framework
   ```rust
   // Checks virtualization, resources, networking, storage, guest
   pub struct DiagnosticRunner { ... }
   pub enum DiagnosticCheck { Virtualization, Resources, Network, Storage, Guest }
   pub struct DiagnosticResult { passed, warnings, recommendations }
   ```

2. Diagnostic checks
   - **Virtualization:** VZ availability, architecture, hypervisor status
   - **Resources:** Memory (threshold 2GB min), disk (threshold 10GB min), CPU
   - **Networking:** Connectivity, DNS resolution, bridge config, port conflicts
   - **Storage:** Disk corruption, missing images, permission issues
   - **Guest:** SSH availability, service health, resource consumption

3. Error recovery suggestions
   - Recommend actions for each failure
   - Provide terminal commands users can copy/paste
   - Link to documentation

4. Tests: 15+ diagnostic tests

**Files to Create:**
- `crates/tinybridge-diagnostics/Cargo.toml` (new crate)
- `crates/tinybridge-diagnostics/src/lib.rs`
- `crates/tinybridge-diagnostics/src/checks/` (module per check type)
- `crates/tinybridge-diagnostics/src/recommendations.rs`

**CLI Integration:**
```bash
tinybridge doctor                    # Full diagnostic report
tinybridge doctor --virtualization   # Specific check
tinybridge doctor --fix              # Auto-remediate (if possible)
```

**Acceptance Criteria:**
- All check types implemented and tested
- Output is clear and actionable
- Recommendations are accurate
- 95%+ of failures have suggested fixes

---

### Phase 2.0.3: Template System (3 weeks)
**Goal:** One-command setup for dev environments

**Deliverables:**
1. Template definition format
   ```yaml
   name: "Rust Development"
   description: "Complete Rust dev environment"
   base: "ubuntu-24.04"
   packages:
     - build-essential
     - rustc
     - cargo
   post_install: |
     # Custom setup script
   ports: [8000-9000]  # Range for local forwarding
   workspace_mount: /workspace
   ```

2. Template engine
   - Parse YAML templates
   - Download base image
   - Apply customizations
   - Handle dependencies

3. Built-in templates
   - Base: Ubuntu, Fedora, Debian, Arch
   - Dev: Rust, Python, Node, Go, C++
   - Robotics: ROS2, Nav2, Gazebo, RViz
   - AI: Jupyter, PyTorch, TensorFlow

4. Custom template support
   - Users can define ~/.tinybridge/templates/custom.yaml
   - Org-wide templates (git URL or local path)
   - Template marketplace (future)

5. Tests: 10+ template tests

**Files to Create:**
- `crates/tinybridge-templates/src/template_engine.rs`
- `crates/tinybridge-templates/data/templates/*.yaml`
- `crates/tinybridge-templates/src/marketplace.rs` (placeholder)

**CLI Integration:**
```bash
tinybridge launch --template rust        # Use built-in template
tinybridge launch --template ~/my.yaml   # Use custom template
tinybridge templates                     # List all templates
tinybridge templates search python       # Search templates
```

**Acceptance Criteria:**
- All built-in templates work end-to-end
- Users can create custom templates
- Template marketplace API defined (Phase 2.0.4+)

---

### Phase 2.0.4: Enhanced Error Handling (2 weeks)
**Goal:** Replace stack traces with guidance

**Deliverables:**
1. Error type hierarchy
   ```rust
   pub enum TinyBridgeError {
       InsufficientDisk { available, required },
       NoVirtualization,
       NetworkUnreachable { reason, suggestion },
       // ...
   }
   ```

2. Error formatter
   - Convert errors to user-friendly messages
   - Include recovery suggestions
   - Link to `tinybridge doctor` output

3. Structured logging
   - Trace errors through daemon
   - Helpful debug output in logs
   - No stack traces in user-facing output

**Acceptance Criteria:**
- All error types have helpful messages
- User sees actionable guidance, not stack traces
- 100% error path coverage in tests

---

### Phase 2.0.5: Integration & Polish (2 weeks)
**Goal:** Tie everything together, end-to-end testing

**Deliverables:**
1. End-to-end flows tested
   - Fresh install → launch ubuntu → SSH ready
   - Fresh install → launch ros2 → ROS2 ready
   - Auto-remediate common errors via doctor

2. Performance validation
   - Boot time still <5s to SSH
   - No regression on Phase 1 metrics

3. Documentation
   - User guide for new commands
   - Template creation guide
   - Troubleshooting guide

4. Beta release candidate
   - All 2.0 features present
   - Ready for v2.0.0 release

---

## Risk Assessment & Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Breaking daemon API | Low | High | Use JSON-RPC versioning, test backward compat |
| Regression on boot time | Medium | High | Benchmark Phase 1 metrics, measure each phase |
| Template delivery latency | Medium | Medium | Cache templates locally, CDN for downloads |
| Error messages unhelpful | Medium | Medium | User testing, iterate on messaging |

---

## Success Metrics

### Phase 1-3 Metrics (Unchanged)
- ✅ Boot to SSH: 1.5s
- ✅ Boot to usable: 5s
- ✅ Tests passing: 80+

### 2.0 Metrics (New)
- ⚡ Time to "Linux ready" from fresh install: <60s (goal)
- 📊 User satisfaction with error messages: 4.5/5.0 (goal)
- 🔧 Diagnostic accuracy: 98%+ (goal)
- 🚀 Template launch success rate: 95%+ (goal)
- 🤝 Zero config required: 90%+ of users (goal)

---

## Timeline

```
Jul 2026   Phase 2.0.1 (CLI Restructuring) .......... 4 weeks
Aug 2026   Phase 2.0.2 (Diagnostics)................. 3 weeks
Aug 2026   Phase 2.0.3 (Templates) .................. 3 weeks
Sep 2026   Phase 2.0.4 (Error Handling).............. 2 weeks
Sep 2026   Phase 2.0.5 (Integration & Polish)........ 2 weeks
           ─────────────────────────────────────
Oct 2026   🎉 TinyBridge v2.0.0 Release
```

**Effort:** ~14 weeks, 2-3 engineers (or 1 eng × 14 weeks)

---

## Next Actions

1. **Immediate (This Week):**
   - ✅ Phase 1 completion + code quality
   - Begin Phase 2.0.1 CLI structure design
   - Set up task tracking for each phase

2. **Short-term (Next 2 Weeks):**
   - Implement Phase 2.0.1 (CLI layer)
   - Set up CI/CD for new commands
   - Write user guide draft

3. **Medium-term (Next 4 Weeks):**
   - Implement Phase 2.0.2 (Diagnostics)
   - Beta test with early users
   - Refine based on feedback

---

## Appendix: 2.0 vs 1.0 Comparison

| Aspect | 1.0 (Current) | 2.0 (Proposed) |
|--------|---------------|----------------|
| **Setup** | Manual config | Auto-detect defaults |
| **Time to Linux** | 5s+ | <60s (including download) |
| **Error Messages** | Stack traces | Actionable guidance |
| **CLI Commands** | Basic (up/down) | Rich (templates, doctor) |
| **Diagnostics** | None | Comprehensive |
| **Performance** | Fast ✅ | Unchanged ✅ |
| **Observability** | OTel framework | OTel + user-friendly logs |
| **Target Audience** | DevOps/Power Users | All developers |

---

**Document Status:** Ready for Phase 2.0.1 Implementation

