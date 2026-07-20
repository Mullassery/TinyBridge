# TinyBridge: Getting Started

**Get a working Linux shell on your Mac.**

---

## What You're About to Do

1. Install TinyBridge
2. Create your first environment
3. SSH into Linux
4. Run commands native in Linux

That's it. You'll have a real Ubuntu system running on your Mac, accessible via SSH, with your files automatically synced.

---

## Prerequisites

- **macOS 14+** (Sonoma or newer)
- **Apple Silicon or Intel Mac** (both supported)
- **~2 GB disk space** for the Linux image
- Homebrew (optional, for easy installation)

> **Status Note:** TinyBridge is in Phase 1 development. Core functionality works. Performance benchmarks (boot time, file I/O) are architectural targets being verified. See [Testing Report](TESTING_REPORT.md) for details.

---

## Installation

### Option 1: Homebrew (Recommended)

```bash
brew install --cask tinybridge
```

Then verify:
```bash
tinybridge --version
```

### Option 2: UV (for Python projects)

If you're using UV to manage your Python environment:

```bash
uv tool install tinybridge
```

Or add to your Python project's requirements:
```bash
uv pip install tinybridge
```

### Option 3: Manual Download

1. Go to [GitHub Releases](https://github.com/Mullassery/tinybridge/releases)
2. Download the latest `.dmg` file
3. Drag `TinyBridge.app` to Applications
4. The CLI is automatically added to your PATH

### Verify Installation

```bash
tinybridge --version
```

You should see a version number. If not, restart Terminal.

### How TinyBridge Gets Detected on macOS

When you install TinyBridge:

1. **Homebrew** (if using `brew install --cask tinybridge`):
   - Installs the `tinybridge` CLI to `/usr/local/bin/tinybridge`
   - Adds the TinyBridge.app to `/Applications`
   - Launches the daemon via LaunchAgent on login

2. **Manual Installation** (from `.dmg`):
   - Drag `TinyBridge.app` to `/Applications`
   - Open it once to register the CLI
   - The daemon runs in background automatically

3. **First Run**:
   - `tinybridge` CLI automatically starts the daemon if not running
   - Daemon stores state at `~/.tinybridge/`
   - Creates socket at `~/.tinybridge/tinybridge.sock` for CLI communication

You can verify the daemon is running:
```bash
ps aux | grep tinybridged
# Should show: /Applications/TinyBridge.app/Contents/MacOS/tinybridged
```

---

## Your First Environment (5 Minutes)

### Step 1: Create `env.yaml`

In your project directory, create a file called `env.yaml`:

```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: my-first-env
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 4
  memory: 8GB
  disk: 50GB
```

**That's your entire environment definition.** Save it in your project root.

**Important:** The `metadata.name` (here: `my-first-env`) must match what you pass to `tinybridge up`.

### Step 2: Start the Environment

```bash
# The name must match metadata.name from env.yaml
tinybridge up my-first-env
```

TinyBridge looks for `env.yaml` in the current directory. If your file is in a subdirectory, use:
```bash
tinybridge up my-first-env --file path/to/env.yaml
```

You'll see:
```
✓ Preparing environment...
✓ Downloading Linux image (one-time, ~500MB)
✓ Starting VM...
✓ Running (SSH ready)
```

First boot downloads the Linux image. Subsequent boots reuse the cached image.

### Step 3: Enter the Shell

```bash
tinybridge shell my-first-env
```

You're now in Ubuntu:
```bash
vm@ubuntu:~$ uname -a
Linux ubuntu 6.12.4-generic #1 SMP ... x86_64 GNU/Linux
vm@ubuntu:~$ 
```

Welcome to Linux. You can run any Linux command here.

### Step 4: Try File Sync

On your Mac (in a new Terminal window):
```bash
echo "Hello from macOS" > ~/my-first-env/test.txt
```

In your Linux shell:
```bash
vm@ubuntu:~$ cat ~/test.txt
Hello from macOS
```

Files sync instantly. No mounting. No complexity.

### Step 5: Stop the Environment

```bash
tinybridge down my-first-env
```

The environment stops. Your files stay. Run `tinybridge up` again to resume.

---

## Common Workflows

### Run a Command Without Entering Shell

```bash
tinybridge exec my-first-env "python3 --version"
```

Output:
```
Python 3.11.9
```

### Check Status

```bash
tinybridge status my-first-env
```

Output:
```
Name:           my-first-env
Status:         Running
IP:             192.168.105.2
Memory:         4.2GB / 8GB
CPU:            12% (was 8% yesterday)
Uptime:         3m 42s
```

### List All Environments

```bash
tinybridge list
```

### SSH Directly (No Shell Command)

```bash
ssh vm@192.168.105.2
```

Same as `tinybridge shell`, but you can pass SSH flags:
```bash
ssh -L 8000:localhost:8000 vm@192.168.105.2
```

### Use a Different Linux Distro

Change your `env.yaml`:

```yaml
substrate:
  os: debian          # Was: ubuntu
  version: "12"       # Debian 12 (bookworm)
```

Stop and restart:
```bash
tinybridge down my-first-env
tinybridge up my-first-env
```

New environment with Debian instead of Ubuntu.

### Install Tools in Your Environment

Edit `env.yaml`:

```yaml
native:
  tools:
    - python@3.11
    - rust@1.87
    - nodejs@20
    - docker
```

Restart the environment. Tools will be pre-installed.

Or install manually after boot:
```bash
vm@ubuntu:~$ apt update && apt install -y python3-pip
```

### Run Multiple Environments in Parallel

Create two YAML files:

**frontend/env.yaml:**
```yaml
metadata:
  name: frontend
substrate:
  os: ubuntu
resources:
  cpu: 4
  memory: 8GB
  disk: 50GB
```

**backend/env.yaml:**
```yaml
metadata:
  name: backend
substrate:
  os: ubuntu
resources:
  cpu: 4
  memory: 8GB
  disk: 50GB
```

Start both:
```bash
tinybridge up frontend
tinybridge up backend
```

Each boots in parallel. Each has its own isolated filesystem and network.

---

## Troubleshooting

### "SSH connection refused"

The VM is still booting. Wait and try again:
```bash
sleep 5 && tinybridge shell my-first-env
```

### "File changes aren't syncing"

File sync is automatic but can cache. Force a sync inside the shell:
```bash
vm@ubuntu:~$ sync
```

### "Out of disk space"

Check usage:
```bash
tinybridge status my-first-env
```

Expand disk:
```bash
tinybridge scale my-first-env --disk 100GB
```

Restart the environment. The disk expands automatically.

### "I want to reinstall the Linux image"

The image is cached at `~/.tinybridge/assets/`. Delete it to re-download:
```bash
rm -rf ~/.tinybridge/assets/
tinybridge up my-first-env  # Downloads fresh image
```

### "Stuck or hung?"

Force stop:
```bash
tinybridge down my-first-env --force
```

Then restart:
```bash
tinybridge up my-first-env
```

---

## Next Steps

### 1. Use Your Real Project

Create `env.yaml` in your project root:

```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: my-project
  version: "1.0.0"
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 6
  memory: 12GB
  disk: 100GB
native:
  tools:
    - rust@latest
    - python@3.11
    - nodejs@20
```

Commit to git:
```bash
git add env.yaml
git commit -m "Add TinyBridge environment config"
```

Your team runs `tinybridge up my-project`. Everyone gets the same environment.

### 2. Integrate with Your Workflow

**Running tests:**
```bash
tinybridge exec my-project "pytest tests/"
```

**Building code:**
```bash
tinybridge exec my-project "cargo build --release"
```

**Debugging:**
```bash
tinybridge shell my-project
# Then run your app with debugger
```

### 3. Match Production

If your production runs Debian 12:
```yaml
substrate:
  os: debian
  version: "12"
```

If you need specific kernel version:
```yaml
substrate:
  os: ubuntu
  version: "22.04"
  kernel: "6.1"
```

### 4. Learn More

- **[Product Vision](PRODUCT_VISION.md)** — Why TinyBridge exists
- **[User Documentation](USER_README.md)** — Complete reference
- **[Architecture](docs/ARCHITECTURE.md)** — How it works under the hood

---

## Command Quick Reference

| Task | Command |
|------|---------|
| Start environment | `tinybridge up NAME` |
| Enter shell | `tinybridge shell NAME` |
| Run command | `tinybridge exec NAME "command"` |
| Check status | `tinybridge status NAME` |
| Stop environment | `tinybridge down NAME` |
| List all | `tinybridge list` |
| Scale resources | `tinybridge scale NAME --cpu 8 --memory 16GB` |
| Direct SSH | `ssh vm@192.168.105.2` |

---

## FAQ

**Q: Is this like Docker?**  
A: Similar idea, but simpler. One environment per `env.yaml`. No images, no layers, no complexity. Real Linux VM, not containers.

**Q: Can I run multiple services?**  
A: Yes. Use the Linux environment like you would any Linux machine. Systemd, background processes, daemons all work.

**Q: How much does this cost?**  
A: Free. Apache 2.0 open-source license. No subscriptions, no license fees.

**Q: What if I want to share my environment with my team?**  
A: Check `env.yaml` into git. Your team clones the repo and runs `tinybridge up`. Everyone gets the identical environment.

**Q: Can I use different Linux distros?**  
A: Yes. Ubuntu, Debian, Alpine, Fedora all supported. Change the `os` field in `env.yaml` and restart.

**Q: How do I access the Linux box from another Mac on my network?**  
A: Coming in Phase 3. For now, SSH tunneling works if you need it.

**Q: Will this work on an M1/M2/M3 Mac?**  
A: Yes, optimized for Apple Silicon. Intel Macs also work (Linux runs via Rosetta 2).

**Q: What if I have an issue?**  
A: Open an issue on [GitHub](https://github.com/Mullassery/tinybridge/issues). Include output of:
```bash
tinybridge status <name> --json
```

---

## You're Ready

You now know:
- How to install TinyBridge ✅
- How to create your first environment ✅
- How to access Linux from your Mac ✅
- How to sync files ✅
- How to stop and restart ✅

**Next:** Create `env.yaml` in your project and run `tinybridge up`. Start using Linux as naturally as macOS.

Questions? Check the [User Documentation](USER_README.md) or open an issue on GitHub.

---

**TinyBridge — Linux development on macOS. No pain.**
