# Clipboard Bridging in TinyBridge

TinyBridge automatically synchronizes your clipboard between macOS and the Linux environment, enabling seamless copy-paste operations across the operating system boundary.

## How It Works

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    macOS Host                           │
│  ┌────────────────────────────────────────────────────┐ │
│  │           NSPasteboard (macOS Clipboard)           │ │
│  └────────────────┬─────────────────────────────────┘ │
│                   │                                     │
│  ┌────────────────▼─────────────────────────────────┐ │
│  │  TinyBridge Clipboard Bridge (Bidirectional)     │ │
│  │  - Monitors macOS pasteboard changes             │ │
│  │  - Syncs via SSH to Linux VM                     │ │
│  │  - Polls Linux clipboard for changes             │ │
│  └────────────────┬─────────────────────────────────┘ │
│                   │                                     │
│                   └────────SSH (Port 2222)─────────────┤
│                                                         │
└─────────────────────────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────┐
│                  Linux VM (Ubuntu)                      │
│  ┌────────────────────────────────────────────────────┐ │
│  │  X11 Clipboard / Wayland (xclip/xsel)             │ │
│  └────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

### Synchronization Flow

1. **macOS → Linux**
   - Monitor NSPasteboard for changes (every 1 second)
   - Read text content
   - SSH into VM and pipe to `xclip -selection clipboard`
   - Updates Linux clipboard instantly

2. **Linux → macOS**
   - Poll Linux clipboard via SSH (every 1 second)
   - Read from `xclip -selection clipboard -o`
   - Write to macOS NSPasteboard
   - Updates macOS clipboard instantly

### Default Behavior

Clipboard sync **starts automatically** when you start a VM:

```bash
$ tinybridge up
Clipboard sync enabled for environment 'default'
VM starting...
```

And stops when you stop the VM:

```bash
$ tinybridge down
Stopping environment 'default'
Clipboard sync disabled
```

## Usage Examples

### Copy from macOS, Paste in Linux

```bash
# On macOS
$ echo "Hello from macOS" | pbcopy

# In Linux VM
$ tinybridge shell
ubuntu$ xclip -selection clipboard -o
Hello from macOS
```

### Copy from Linux, Paste on macOS

```bash
# In Linux VM
$ echo "Hello from Linux" | xclip -selection clipboard

# On macOS
$ pbpaste
Hello from Linux
```

### Text Editors

Works seamlessly with:
- **VS Code** — copy in macOS, paste in Linux VM terminal
- **vim/nano** — copy output from macOS, paste into editor in Linux
- **Browser** — copy from macOS, paste in Linux web dev server

### Data Processing

Example: copy CSV data from macOS → process in Linux:

```bash
# On macOS, copy CSV to clipboard

# In Linux VM
$ xclip -selection clipboard -o > data.csv
$ python process_data.py data.csv
```

## Configuration

Clipboard sync is enabled by default with a 1-second polling interval. No configuration needed for most use cases.

### Advanced: Custom Sync Interval

(Future: when integrated with `env.yaml`)

```yaml
clipboard:
  enabled: true
  sync_interval_ms: 500  # Sync every 500ms
```

## Requirements

### On macOS
- NSPasteboard API (built into macOS)
- SSH client (built into macOS)

### On Linux VM
- `xclip` or `xsel` (installed by default in Ubuntu 24.04)
- SSH server running on port 2222

## Troubleshooting

### Clipboard sync not working

**Check 1: Is the VM running?**
```bash
$ tinybridge status
default          Running    192.168.64.2  1 min
```

**Check 2: Is SSH available?**
```bash
$ tinybridge shell
(should connect without errors)
```

**Check 3: Are xclip/xsel installed?**
```bash
$ tinybridge shell
ubuntu$ which xclip
/usr/bin/xclip
```

If not installed:
```bash
ubuntu$ sudo apt-get install -y xclip
```

### Clipboard sync is slow

Default poll interval is 1 second. This can be adjusted in future versions via `env.yaml`.

### Large clipboard content hangs

SSH has default buffer limits. For content >1MB, consider:
- Writing to a file and transferring instead
- Compressing before copying

## Performance

- **Latency**: ~500ms (SSH round-trip + SSH command overhead)
- **Size limit**: Practical limit ~10MB (depends on SSH buffer)
- **CPU impact**: <1% (minimal polling overhead)

## Security

Clipboard content is:
- Transmitted over **SSH (encrypted)**
- Never logged to disk
- Never cached beyond current session
- Tied to the running environment (stops when VM stops)

## Future Enhancements

- **Phase 2**: Configurable sync interval via `env.yaml`
- **Phase 2**: Disable/enable per-environment
- **Phase 3**: Support for rich content (images, formatted text)
- **Phase 4**: Clipboard history + search
- **Phase 5**: Cross-environment clipboard sharing (multiple VMs)

## Architecture Details

### Implementation

- **Module**: `crates/tinybridge-clipboard/`
- **Components**:
  - `MacosPasteboard` — NSPasteboard access via objc FFI
  - `LinuxClipboard` — SSH-based clipboard access
  - `ClipboardBridge` — Bidirectional sync orchestration
  - `ClipboardSyncManager` — Task lifecycle management in daemon

### Code Flow

```
EnvironmentManager::up()
  ├─ Create VM
  ├─ Start VM
  └─ ClipboardSyncManager::start_sync(env_id, ssh_host, ssh_port, ssh_user)
       └─ Spawn tokio task with ClipboardBridge
           └─ Loop every 1s:
              ├─ sync_macos_to_linux()
              └─ sync_linux_to_macos()

EnvironmentManager::down()
  └─ ClipboardSyncManager::stop_sync(env_id)
      └─ Abort clipboard sync task
```

---

**Note**: Clipboard bridging is a Phase 1 MVP feature. The implementation prioritizes simplicity and reliability over advanced features. Rich content support (images, formatted text) is deferred to Phase 3+.
