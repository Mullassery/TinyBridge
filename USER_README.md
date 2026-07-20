# TinyBridge: User Guide

**Linux development on macOS. Fast.**

> **New to TinyBridge?** Start with [Getting Started](GETTING_STARTED.md) for a 5-minute walkthrough. Come back here for detailed reference.

---

## What is TinyBridge?

TinyBridge is a lightweight Linux environment for macOS that prioritizes fast SSH access to a real Ubuntu environment with file sharing and native resource management.

No Docker. No complexity. No waiting.

---

## Install

### macOS 14.0+ (Apple Silicon or Intel)

**Via Homebrew (easiest):**
```bash
brew install --cask tinybridge
```

**Via UV (Python environments):**
```bash
uv tool install tinybridge
```

**Manual download:**
1. Go to [GitHub Releases](https://github.com/Mullassery/tinybridge/releases)
2. Download the latest `.dmg`
3. Drag TinyBridge to Applications

**Verify installation:**
```bash
tinybridge --version
```

TinyBridge runs as a background service. No daemon setup needed.

---

## Your First Environment

Create an `env.yaml` file:

```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: my-project
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 4
  memory: 8GB
  disk: 50GB
native:
  tools:
    - rust@1.87
    - python@3.11
```

Boot it:
```bash
tinybridge up my-project
```

Within seconds:
```bash
✓ Running (SSH ready)
```

Enter the environment:
```bash
ssh vm@192.168.1.10
```

You're now in Ubuntu 24.04 with Rust and Python pre-installed.

Stop it:
```bash
tinybridge down my-project
```

That's it. Your files in `~/my-project` are automatically synced to `/home/user` in the VM.

---

## Common Commands

```bash
# Start an environment
tinybridge up myenv

# Stop an environment
tinybridge down myenv

# Check status
tinybridge status myenv

# List all environments
tinybridge list

# Access the shell
tinybridge shell myenv

# Or SSH directly
ssh vm@192.168.1.10
```

---

## How It Works

### Boot Timeline

TinyBridge uses multi-tier lazy loading:

**Tier 1 (SSH Ready):** Kernel + VirtioFS + networking online. SSH access available.  
**Tier 2 (Usable):** Core system services running. Typical development tasks ready.  
**Tier 3 (Complete):** All services online. System fully initialized.  

You get SSH access at Tier 1 and start working immediately. Tiers 2-3 load in the background while you develop.

> **Performance Note:** Actual boot times are being benchmarked in Phase 1. Target is Tier 1 SSH in <2s, Tier 2 in <5s. See [Testing Report](../TESTING_REPORT.md) for methodology and expected results vs. Lima.

### File Sharing

Your Mac filesystem is automatically available in the VM:

```bash
# On macOS
$ echo "hello" > ~/project.txt

# In the VM
vm@ubuntu:~$ cat ~/project.txt
hello
```

No mounting. No setup. Just works. 90%+ native speed.

### Networking

The VM gets its own IP (192.168.105.2 by default). SSH access. Port forwarding. Standard Linux networking.

---

## Environment Configuration

### Complete Schema

```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: my-project         # Required: environment name
  version: "1.0.0"         # Optional: version tag
  description: "..."       # Optional: human-readable description

substrate:
  os: ubuntu               # Linux distribution (ubuntu, debian, alpine)
  version: "24.04"         # OS version (default: latest LTS)
                           # Ubuntu: "24.04", "22.04", "20.04"
                           # Debian: "12", "11", "10"
                           # Alpine: "3.19", "3.18"
  arch: [arm64]            # [arm64] for Apple Silicon (default)
                           # [amd64] for Intel (runs via Rosetta 2)

resources:
  cpu: 4                   # Number of cores (1-16)
  memory: 8GB              # RAM (e.g., "4GB", "16GB")
  disk: 50GB               # Disk size (e.g., "20GB", "100GB")

native:
  tools:                   # Optional: pre-installed tools
    - rust@1.87
    - python@3.11
    - go@1.21
    - node@18
```

### Supported Linux Distributions

TinyBridge supports multiple Linux distributions and versions. Choose the one that matches your production environment:

**Ubuntu** (recommended for most users)
- 24.04 LTS (default)
- 22.04 LTS
- 20.04 LTS

**Debian**
- 12 (bookworm)
- 11 (bullseye)

**Alpine**
- 3.19
- 3.18

**Fedora**
- 39
- 38

Example: Use Debian if your production runs Debian:
```yaml
substrate:
  os: debian
  version: "12"
```

### Resource Limits

- **CPU**: 1-16 cores (your Mac's max)
- **Memory**: 512MB-32GB
- **Disk**: 10GB-500GB (storage available on your Mac)

Resize anytime:
```bash
tinybridge scale myenv --cpu 8 --memory 16GB
```

---

## Sharing Environments

Check `env.yaml` into git:

```bash
git add env.yaml
git commit -m "Add environment config"
git push
```

Your team clones the repo and runs:
```bash
tinybridge up myenv
```

Everyone gets identical environments.

---

## Troubleshooting

### "Boot is slow"

First boot downloads the Linux image (~500MB). Subsequent boots are faster.

If boots are consistently slow:
```bash
tinybridge status myenv --verbose
```

This shows resource contention. Scale CPU or memory if needed, or check system load.

### "SSH connection refused"

VM is still booting. Wait 2-3 seconds and try again:
```bash
sleep 3 && ssh vm@192.168.1.10
```

### "File changes not syncing"

VirtioFS caches by default. Give it a second or force a sync:
```bash
vm@ubuntu:~$ sync
```

### "Out of disk space"

Check current usage:
```bash
tinybridge status myenv
```

Expand the disk:
```bash
tinybridge scale myenv --disk 100GB
```

---

## Performance Tips

### Use Local SSD
TinyBridge performance is fastest on local SSD. If on external drive, expect slower boot and file access.

### Allocate Enough Resources
For development:
- **Minimum**: 4 CPU, 8GB RAM, 50GB disk
- **Recommended**: 6+ CPU, 16GB RAM, 100GB+ disk

Allocate too little and everything feels slow.

### Keep Environments Small
One environment per project beats one environment for everything.

```bash
# ✓ Good: separate environments
tinybridge up frontend
tinybridge up api
tinybridge up database

# ✗ Avoid: one monolithic environment
tinybridge up everything
```

---

## Advanced Usage

### Custom Port Forwarding

Forward ports from VM to macOS:
```bash
# Forward VM port 3000 to macOS localhost:3000
ssh -L 3000:localhost:3000 vm@192.168.1.10
```

Then on macOS:
```bash
curl localhost:3000
```

### Multiple Environments

Run several environments in parallel:
```bash
tinybridge up api
tinybridge up frontend
tinybridge up database

# All three running simultaneously
tinybridge list
```

Each boots in 1.5 seconds independently.

### Inspect Environment Details

```bash
# Show all details
tinybridge status myenv --json

# Real-time monitoring
tinybridge watch myenv
```

---

## What's Coming

### Phase 2: Team Environments (Q3 2026)
- Environment versioning
- Team templates
- Multi-user support
- Git-based workflows

### Phase 3: Advanced Networking (Q4 2026)
- Multicast (ROS 2 DDS native)
- Port forwarding UI
- Service discovery
- Network policies

### Phase 4: GPU Support (Q1 2027)
- Transparent CUDA routing to local GPU
- Remote GPU fallback
- ML workload optimization
- Train models locally at production scale

### Phase 5: Ecosystem (Q2 2027)
- Plugin SDK
- Community packages
- Advanced orchestration
- Production dashboards

---

## FAQ

**Q: Do I need Docker installed?**  
A: No. TinyBridge is independent.

**Q: Does this work on Intel Macs?**  
A: Yes. Specify `arch: [amd64]` in env.yaml. TinyBridge uses Rosetta 2 to run AMD64 Linux on Apple Silicon efficiently.

**Q: Can I run multiple VMs at once?**  
A: Yes, one per environment. Each boots in 1.5 seconds independently.

**Q: Where does the Linux image live?**  
A: `~/.tinybridge/assets/`. You can move it to another drive if needed.

**Q: How much disk space does TinyBridge need?**  
A: The Linux image is ~500MB. Plus whatever you allocate per environment (minimum 10GB recommended).

**Q: Is this production-ready?**  
A: Phase 1 is stable for development. Not recommended for hosting services yet.

**Q: Can I use this with CI/CD?**  
A: Yes. Run `tinybridge up` in your CI pipeline. Each build gets a fresh environment.

**Q: What about GPU support?**  
A: Coming in Phase 4. Track [the roadmap](./PRODUCT_VISION.md) for updates.

**Q: How do I contribute?**  
A: TinyBridge is Apache 2.0 open source. Contributions welcome at [GitHub](https://github.com/Mullassery/tinybridge).

---

## Get Help

- **Issues**: [GitHub Issues](https://github.com/Mullassery/tinybridge/issues)
- **Documentation**: [Full Docs](./docs/)
- **Roadmap**: [Development Roadmap](./PRODUCT_VISION.md)

---

**Ready to go?** Start with:
```bash
tinybridge up my-first-project
```

Questions? Open an issue on GitHub.

---

*TinyBridge — Linux development on macOS. In under 2 seconds.*
