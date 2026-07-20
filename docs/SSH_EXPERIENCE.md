# TinyBridge SSH Experience Design

**Philosophy**: SSH should be invisible, automatic, and trustworthy. Users should never think about keys, configs, or passwords.

## User Experience

### The Ideal Flow

```bash
# Create a VM
$ tinybridge up myvm

# Within seconds, you can connect
$ ssh myvm
user@myvm:~$

# No passwords, no key setup, no configuration
# That's it. Ship your code.
```

### Key Design Principles

1. **Zero Configuration** — Users never manage SSH keys or configs manually
2. **Automatic Aliases** — `ssh myvm` works, not `ssh user@192.168.64.2 -i ~/.tinybridge/keys/...`
3. **Secure by Default** — Ed25519 keys, no password auth, audit everything
4. **Transparent** — SSH operations happen in the background
5. **Reliable** — Survives VM state changes, IP changes, network transitions
6. **Developer-Friendly** — One-click actions for Terminal, VS Code, Cursor, etc.

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                   TinyBridge SSH Layer                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  SSH Key Manager                                        │ │
│  │  - Generate Ed25519 keypairs per environment           │ │
│  │  - Store in ~/.tinybridge/keys/{env_id}/               │ │
│  │  - Rotate keys on demand                               │ │
│  │  - Support hardware-backed keys (future)               │ │
│  └─────────────────────────────────────────────────────────┘ │
│                           │                                   │
│  ┌────────────────────────▼─────────────────────────────────┐ │
│  │  SSH Provisioner                                        │ │
│  │  - Inject public key during first boot                 │ │
│  │  - Multi-distro support (Ubuntu, Fedora, Alpine, etc.) │ │
│  │  - Cloud-init, Ignition, custom script delivery       │ │
│  │  - Verify successful provisioning                      │ │
│  └─────────────────────────────────────────────────────────┘ │
│                           │                                   │
│  ┌────────────────────────▼─────────────────────────────────┐ │
│  │  SSH Config Manager                                     │ │
│  │  - Maintain ~/.ssh/config entries                      │ │
│  │  - Auto-generate aliases (ssh myvm)                    │ │
│  │  - Update on IP change                                 │ │
│  │  - Remove on environment deletion                      │ │
│  └─────────────────────────────────────────────────────────┘ │
│                           │                                   │
│  ┌────────────────────────▼─────────────────────────────────┐ │
│  │  SSH Session Manager                                    │ │
│  │  - Track active connections                            │ │
│  │  - Provide quick-launch shortcuts                      │ │
│  │  - Monitor session metrics                             │ │
│  └─────────────────────────────────────────────────────────┘ │
│                           │                                   │
│  ┌────────────────────────▼─────────────────────────────────┐ │
│  │  SSH Audit Logger                                       │ │
│  │  - Log all key operations                              │ │
│  │  - Track connection attempts                           │ │
│  │  - Record auth failures                                │ │
│  │  - Enterprise compliance ready                         │ │
│  └─────────────────────────────────────────────────────────┘ │
│                           │                                   │
│  ┌────────────────────────▼─────────────────────────────────┐ │
│  │  SSH Repair Service                                     │ │
│  │  - Detect broken configs                               │ │
│  │  - Auto-repair SSH entries                             │ │
│  │  - Handle IP transitions                               │ │
│  │  - Cleanup stale entries                               │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
└─────────────────────────────────────────────────────────────┘
        │
        │  Injects public key
        │  during first boot
        ▼
┌─────────────────────────────────────────────────────────────┐
│                   Linux Guest                               │
│  ~/.ssh/authorized_keys (populated automatically)           │
└─────────────────────────────────────────────────────────────┘
```

### Key Storage

```
~/.tinybridge/
├── keys/
│   ├── {env-id}/
│   │   ├── id_ed25519              (private key)
│   │   ├── id_ed25519.pub          (public key)
│   │   ├── fingerprint             (SSH fingerprint)
│   │   └── metadata.json           (key info, rotation history)
│   └── ...
├── ssh/
│   ├── config.d/
│   │   ├── tinybridge-auto.conf    (generated entries)
│   │   └── tinybridge-manual.conf  (user additions)
│   └── audit.log                   (audit trail)
└── sessions/
    └── active.json                 (current sessions)
```

## Implementation Plan

### Phase 2a: SSH Infrastructure (This Session)

**Crate: `tinybridge-ssh`**

1. **SSH Key Manager** (key_manager.rs)
   - Generate Ed25519 keypairs
   - Store with metadata
   - Manage key lifecycle
   - Support key rotation

2. **SSH Provisioner** (provisioner.rs)
   - Cloud-init integration
   - Multi-distro detection
   - Public key injection
   - Verification

3. **SSH Config Manager** (config_manager.rs)
   - Generate ~/.ssh/config entries
   - Auto-create aliases
   - Maintain consistency
   - Handle updates/deletions

4. **SSH Audit Logger** (audit.rs)
   - Log all operations
   - Track events chronologically
   - Query capabilities
   - Retention policies

### Phase 2b: Daemon Integration (Next Session)

1. Connect to EnvironmentManager lifecycle
2. Auto-generate keys on `up`
3. Inject during first boot
4. Cleanup on `down`
5. Track active sessions

### Phase 2c: CLI + UI (Phase 3)

1. `tinybridge ssh <env>`
2. `tinybridge ssh-key <env>`
3. `tinybridge ssh-audit`
4. One-click terminal/IDE actions
5. SSH completion

## Security Model

### Private Key Protection

```
Private Key Storage:
├── File permissions: 600 (owner read/write only)
├── Directory permissions: 700 (owner access only)
├── No backup to cloud (user responsibility)
├── Optional: Hardware Secure Enclave (Phase 3)
└── Never exposed to guest OS
```

### Public Key Delivery

```
Methods (in priority order):
1. Cloud-init (Ubuntu, Debian, Fedora, CentOS, RHEL, Rocky)
2. Ignition (CoreOS, Fedora CoreOS)
3. Custom script via VirtioFS (Alpine, Arch, others)
4. Manual injection (fallback)
```

### Audit Trail

```
Every operation logs:
├── Timestamp
├── Operation (KeyCreated, KeyRotated, ConnectionAttempt, etc.)
├── Environment ID
├── User (local macOS user)
├── Result (success/failure)
└── Details (fingerprint, IP, auth method, etc.)
```

## CLI Commands

```bash
# Connect to environment (just works)
tinybridge ssh myvm                    # SSH into myvm
tinybridge ssh myvm "ls -la"           # Execute command
tinybridge ssh myvm -c "bash"          # Interactive shell

# SSH key management
tinybridge ssh-key list                # List all keys
tinybridge ssh-key show <env>          # Show key info + fingerprint
tinybridge ssh-key rotate <env>        # Rotate key
tinybridge ssh-key export <env>        # Export public key (for external systems)

# SSH config management
tinybridge ssh-config show             # Show generated config
tinybridge ssh-config rebuild          # Rebuild from scratch
tinybridge ssh-config repair           # Fix broken entries

# Audit and monitoring
tinybridge ssh-audit log               # View audit log
tinybridge ssh-audit events <env>      # Events for specific environment
tinybridge ssh-audit export log.txt    # Export for compliance

# Sessions and tunnels (Phase 3)
tinybridge ssh-session list            # Active sessions
tinybridge ssh-tunnel create <env> 8000:localhost:3000  # Port forward
tinybridge ssh-proxy <env>             # SOCKS proxy
```

## Integration Examples

### VS Code Remote SSH

```bash
# User workflow:
1. tinybridge up training-cluster
2. Ctrl+K Ctrl+O in VS Code
3. Type: myvm
4. Connects instantly (no credential prompts)

# Behind the scenes:
- SSH key already in ~/.ssh/
- SSH config already has entry
- VS Code uses standard SSH (it just works)
```

### Terminal.app

```bash
# One-click action in TinyBridge UI:
"Open Terminal" button
  → Runs: open -a Terminal /usr/bin/ssh myvm
  → Terminal opens with SSH connection ready
```

### Docker Compose Remote

```yaml
# docker-compose.yml
services:
  web:
    build: .
    ports:
      - "3000:3000"

# User runs:
$ docker context create ssh myvm
$ docker compose -c ssh up
# Works seamlessly
```

## Multi-Environment Example

```bash
# User creates multiple environments
$ tinybridge up backend     # Creates backend environment
$ tinybridge up frontend    # Creates frontend environment
$ tinybridge up ml-train    # Creates ML training environment

# SSH config automatically populated
$ cat ~/.ssh/config
Host backend
  HostName 192.168.64.2
  User user
  IdentityFile ~/.tinybridge/keys/backend-uuid/id_ed25519
  StrictHostKeyChecking accept-new

Host frontend
  HostName 192.168.64.3
  User user
  IdentityFile ~/.tinybridge/keys/frontend-uuid/id_ed25519
  StrictHostKeyChecking accept-new

Host ml-train
  HostName 192.168.64.4
  User user
  IdentityFile ~/.tinybridge/keys/ml-train-uuid/id_ed25519
  StrictHostKeyChecking accept-new

# All work instantly
$ ssh backend
$ ssh frontend
$ ssh ml-train

# Even from Docker
$ docker run --rm alpine ssh backend "uname -a"

# Or from scripts
$ for host in backend frontend ml-train; do
    ssh $host "curl https://api.github.com" &
  done
```

## Reliability Features

### Automatic Recovery

```
Scenario: VM IP changes
→ Detect IP change during status check
→ Update ~/.ssh/config entry
→ Log event
→ Notify user (if configured)
→ SSH connection still works

Scenario: SSH directory permissions wrong
→ Detect on next operation
→ Auto-fix to 700
→ Log event

Scenario: Stale known_hosts entry
→ Detect connection failure
→ Offer to remove stale entry
→ Reconnect
→ Log event
```

## Observability

### Dashboard View

```
TinyBridge SSH Status:
├── backend (192.168.64.2:22)
│   ├── Key: ed25519 (SHA256:abc...)
│   ├── Status: Ready ✓
│   ├── Sessions: 0 active
│   └── Last connection: 2h ago
├── frontend (192.168.64.3:22)
│   ├── Key: ed25519 (SHA256:def...)
│   ├── Status: Ready ✓
│   ├── Sessions: 1 active (bash, 15m)
│   └── Last connection: 5m ago
└── ml-train (192.168.64.4:22)
    ├── Key: ed25519 (SHA256:ghi...)
    ├── Status: Degraded ⚠ (IP changed, recovering)
    ├── Sessions: 0 active
    └── Last connection: Never
```

### Audit Log Example

```
[2026-07-20 14:23:45] KeyCreated backend-env-uuid
  Fingerprint: SHA256:abc123...
  KeyType: ed25519
  
[2026-07-20 14:23:50] PublicKeyInjected backend-env-uuid
  OS: Ubuntu 24.04
  Method: cloud-init
  Verified: ✓

[2026-07-20 14:23:52] SshConfigCreated backend
  Alias: backend
  Hostname: 192.168.64.2
  User: user

[2026-07-20 14:24:00] ConnectionAttempt backend
  Status: success
  AuthMethod: publickey
  DurationMs: 250

[2026-07-20 16:45:30] IpAddressChanged backend-env-uuid
  OldIp: 192.168.64.2
  NewIp: 192.168.64.5
  Action: ConfigUpdated
  
[2026-07-20 19:30:00] ConnectionAttempt backend
  Status: success
  AuthMethod: publickey
  DurationMs: 245

[2026-07-20 20:15:00] EnvironmentDeleted backend-env-uuid
  KeyAction: archived
  ConfigAction: removed
```

## Enterprise Features (Phase 3+)

### SSH Certificates

```bash
# For large teams (optional)
$ tinybridge ssh-cert install /path/to/ca_key.pem

# Automatic certificate generation instead of keys
# Centralized key rotation
# Better audit trail
```

### Hardware-Backed Keys

```bash
# Secure Enclave support (macOS)
$ tinybridge ssh-key create --hardware-backed secure-enclave

# YubiKey support
$ tinybridge ssh-key create --hardware-backed yubikey

# TPM support (future)
```

### Centralized Audit

```bash
# Export to SIEM
$ tinybridge ssh-audit export --format syslog
$ tinybridge ssh-audit export --format json > audit.json
$ tinybridge ssh-audit export --format csv > audit.csv
```

## Success Criteria

1. ✅ Zero manual key management
2. ✅ Automatic alias generation (`ssh myvm` just works)
3. ✅ Secure (Ed25519, no passwords, audit logs)
4. ✅ Reliable (survives IP changes, VM state changes)
5. ✅ Developer-friendly (one-click terminal, VS Code integration)
6. ✅ Enterprise-ready (audit logs, recovery, compliance)

The experience should be so seamless that users forget SSH exists. It's just infrastructure.

---

**Status**: Design complete, ready for implementation (Phase 2a)
