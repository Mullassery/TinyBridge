# Phase 3 Week 1-2 OKR: Daemon Integration & Error Propagation

**Date:** 2026-07-25  
**Status:** Starting  
**Goal:** Wire error handling and diagnostics into daemon for structured error reporting

## Objectives

### 1. Error Propagation Layer
- [ ] Create error serialization for JSON-RPC responses
- [ ] Extend daemon error types to use BridgeError
- [ ] Add error context in daemon responses
- [ ] Implement structured error logging with OTel
- [ ] Test end-to-end error flow (daemon → CLI → user)

### 2. Health Check Endpoints
- [ ] Add `/health` JSON-RPC endpoint
- [ ] Implement resource monitoring (memory, CPU, disk)
- [ ] Add VM state health checks
- [ ] Integrate diagnostics into health checks
- [ ] Create health status enum (Healthy, Degraded, Unhealthy)

### 3. Daemon Logging Enhancement
- [ ] Replace basic logging with structured OTel
- [ ] Add error context to all log messages
- [ ] Implement log filtering by severity
- [ ] Add trace correlation IDs for debugging
- [ ] Export logs to configured backend (Jaeger, Datadog, etc.)

### 4. Graceful Error Handling
- [ ] Implement daemon signal handling (SIGTERM, SIGINT)
- [ ] Add cleanup on shutdown
- [ ] Implement connection draining
- [ ] Add error recovery mechanisms

## Key Files to Create/Modify

**Create:**
- `crates/tinybridge-daemon/src/error_propagation.rs` (error serialization)
- `crates/tinybridge-daemon/src/health.rs` (health check endpoints)
- `crates/tinybridge-daemon/src/structured_logging.rs` (OTel integration)

**Modify:**
- `crates/tinybridge-daemon/src/main.rs` (signal handling, shutdown)
- `crates/tinybridge-daemon/src/rpc_handler.rs` (error wrapping)
- `crates/tinybridge-daemon/Cargo.toml` (add tinybridge-error dependency)

## Success Criteria

- [ ] 20+ tests for error propagation
- [ ] Daemon returns structured errors in JSON-RPC
- [ ] Health endpoint implemented and tested
- [ ] OTel export working (at least logging backend)
- [ ] Graceful shutdown under load (tested)
- [ ] No regressions in Phase 1-2 metrics

## Test Plan

- Unit tests for error serialization
- Integration tests for error flow
- Daemon health check tests
- Load testing with error scenarios
- OTel export verification

## Timeline

**Week 1:**
- Error propagation layer (Days 1-2)
- Health check endpoints (Days 2-3)
- Integration testing (Days 3-5)

**Week 2:**
- Daemon logging enhancement (Days 1-2)
- Graceful shutdown (Days 2-3)
- End-to-end testing (Days 3-5)

## Blocking Dependencies

- Phase 2 complete (tinybridge-error, diagnostics) ✅
- Daemon RPC handler stable ✅
- OTel backend available ✅

## Notes

This phase bridges Phase 2 (CLI DX improvements) with daemon infrastructure. The goal is to make errors flow cleanly from daemon through CLI to users, with full context and recovery suggestions.

Key insight: Phase 2 added user-friendly error types to CLI layer. Phase 3 extends this to daemon, enabling bidirectional error reporting and structured logging.
