# TinyBridge

**Run Linux on your Mac. Instantly. No hassle.**

---

## The Problem

You're a macOS developer. Your code runs on Linux. So you're stuck:

- **Docker Desktop** is bloated, slow, and costs $100/year
- **Lima** is bare-bones—no GUI, no snapshots, constant frustration
- **UTM** has a GUI but no daemon—you need the app open to keep VMs running
- **Dual booting** wastes disk space and time switching

You want **a real Linux VM** that:
- Boots in seconds, not minutes
- Has a desktop you can actually see
- Runs headless when you need it (same VM, no restart)
- Stays running even after you close the app
- Doesn't require a PhD to set up

**TinyBridge solves this.** One command. Full Linux. On demand.

---

## What You Get

### ⚡ **Get Started in 30 Seconds**

```bash
brew install tinybridge tinybridged
tinybridge up myproject
ssh vm@myproject
```

Done. You have a fully-functional Ubuntu environment. No YAML wizardry. No configuration files. No waiting.

### 🖥️ **Desktop When You Want It**

Launch with a desktop:
```bash
tinybridge up myproject --gui
```

No desktop? Run headless:
```bash
tinybridge up myproject
```

Want to switch later? Zero restart:
```bash
tinybridge gui myproject        # Show desktop
tinybridge headless myproject   # Hide it (VM keeps running)
```

### 🔧 **Resources You Control**

Create with custom specs:
```bash
tinybridge up myproject --cpu 8 --memory 16 --disk 100
```

Change them later without stopping:
```bash
tinybridge update myproject --cpu 16 --memory 32
```

### 💾 **Snapshots for Sanity**

Save state before risky experiments:
```bash
tinybridge snapshot myproject create before-update
# ... do something dangerous ...
tinybridge snapshot myproject restore before-update
```

Back to square one. Instantly.

### 🛑 **Pause, Don't Kill**

Suspend your VM (preserves memory, all running processes):
```bash
tinybridge suspend myproject
```

Resume exactly where you left off:
```bash
tinybridge resume myproject
```

---

## Common Use Cases

### **Local Development**
```bash
tinybridge up backend --cpu 4 --memory 8
tinybridge ssh backend
# Code natively on macOS, deploy tests in real Linux

ubuntu@backend:~$ ./run-tests.sh
```

### **Robotics / ROS 2**
```bash
tinybridge up robot --cpu 8 --memory 16
tinybridge ssh robot
ubuntu@robot:~$ ros2 run my_package my_node
# DDS multicast networking works out of the box
```

### **Data Science / ML**
```bash
tinybridge up ml-training --cpu 16 --memory 64 --disk 200
tinybridge snapshot ml-training create baseline
# Train model, experiment, restore baseline to retry
tinybridge snapshot ml-training restore baseline
```

### **Team Onboarding**
```bash
# Commit env.yaml to your repo
git add env.yaml
git commit -m "Development environment"

# Your teammate:
git clone <your-repo>
tinybridge up myproject
# Identical setup, zero config
```

### **CI/CD Testing**
```bash
tinybridge up test-env
tinybridge ssh test-env "cd /path/to/tests && ./run.sh"
tinybridge down test-env
# Repeatable, isolated, fast
```

---

## Installation

### Homebrew (Recommended)

```bash
brew tap Mullassery/tinybridge https://github.com/Mullassery/homebrew-tinybridge.git
brew install tinybridge tinybridged
```

That's it. Daemon auto-starts on login.

### Manual (Build from Source)

```bash
git clone https://github.com/Mullassery/tinybridge
cd tinybridge
cargo build --release

# Install
sudo cp target/release/tinybridge /usr/local/bin/
sudo cp target/release/tinybridged /usr/local/bin/

# Verify
tinybridge --version
```

---

## Quick Start

### 1️⃣ Create an Environment

```bash
tinybridge up myproject
```

First run: Downloads Linux image (~500MB, one-time only)

### 2️⃣ Access It

**Option A: SSH (Recommended)**
```bash
ssh vm@myproject
```

**Option B: Shell**
```bash
tinybridge shell myproject
```

**Option C: Desktop**
```bash
tinybridge gui myproject
```

### 3️⃣ Do Your Work

```bash
ubuntu@myproject:~$ python3 app.py
ubuntu@myproject:~$ npm test
ubuntu@myproject:~$ cargo build
```

All your files from macOS are available. Zero setup.

### 4️⃣ When You're Done

**Stop it** (VM stops, disk preserved):
```bash
tinybridge down myproject
```

**Suspend it** (VM pauses, no resource use):
```bash
tinybridge suspend myproject
```

**Delete it** (full cleanup):
```bash
tinybridge destroy myproject
```

---

## Why Choose TinyBridge?

| | **TinyBridge** | **Docker Desktop** | **Lima** | **UTM** |
|---|---|---|---|---|
| **Cost** | Free | $100/year | Free | Free |
| **Boot time** | <5s | 30s+ | 10s+ | 20s+ |
| **GUI available** | ✅ Yes | ❌ No | ❌ No | ✅ Yes |
| **Headless mode** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ App required |
| **Snapshots** | ✅ Yes | ❌ No | ❌ No | ✅ Yes |
| **Suspend/Resume** | ✅ Yes | ❌ No | ❌ No | ✅ Yes |
| **Auto-start daemon** | ✅ Yes | ❌ No | ⚠️ Manual | ❌ No |
| **SSH ready** | ✅ Auto | ❌ Complex | ✅ Auto | ⚠️ Manual |
| **ROS 2 DDS** | ✅ Works | ❌ Broken | ❌ No | ✅ Works |

---

## All Commands at a Glance

```bash
# Lifecycle
tinybridge up myproject                          # Start
tinybridge down myproject                        # Stop
tinybridge suspend myproject                     # Pause
tinybridge resume myproject                      # Unpause
tinybridge restart myproject                     # Restart
tinybridge destroy myproject                     # Delete

# Access
tinybridge ssh myproject                         # SSH
tinybridge shell myproject                       # Interactive shell
tinybridge gui myproject                         # Show desktop
tinybridge headless myproject                    # Hide desktop

# Configuration
tinybridge up myproject --cpu 8 --memory 16      # Create with specs
tinybridge update myproject --cpu 16             # Change resources

# Snapshots
tinybridge snapshot myproject create latest      # Save state
tinybridge snapshot myproject restore latest     # Load state
tinybridge snapshot myproject list               # Show all
tinybridge snapshot myproject delete latest      # Remove

# Status
tinybridge status myproject                      # Check state
tinybridge list                                  # All environments
```

---

## FAQ

### Q: Do I lose data when I close my laptop?
**A:** No. TinyBridge uses a daemon—VMs keep running in the background. Your data is always safe. You can even restart your Mac.

### Q: Can I suspend a VM and come back to it exactly as it was?
**A:** Yes. `tinybridge suspend` pauses the VM. All processes stay in memory. `tinybridge resume` wakes it up instantly.

### Q: Do I need to pay for a subscription?
**A:** No. TinyBridge is completely free. No trials, no upsells, no vendor lock-in.

### Q: What if I need to scale resources (more CPU/RAM)?
**A:** Just run `tinybridge update myproject --cpu 16`. No restart needed.

### Q: Can my team share environments?
**A:** Yes. Commit `env.yaml` to your repo. Your teammates run `tinybridge up myproject` and get the exact same setup.

### Q: Does it work with ROS 2?
**A:** Yes. DDS multicast networking works out of the box—no iptables hacks required.

### Q: What about GPU support?
**A:** Phase 2 (coming soon). For now, you can use local CPU or route to remote GPUs via SSH.

---

## Roadmap

- **v0.3.0** (Now): Full VM lifecycle, snapshots, dual-mode GUI, SSH
- **v0.4.0** (Aug): GPU routing, advanced networking
- **v0.5.0** (Sep): Kubernetes integration, distributed VMs
- **v1.0.0** (Oct): Production-ready, enterprise support

---

## Support

- **Issues**: [GitHub Issues](https://github.com/Mullassery/tinybridge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Mullassery/tinybridge/discussions)
- **Email**: mullassery@gmail.com

---

## License

Proprietary. Source code available on GitHub (private).

---

**Stop configuring. Start developing.**

[Get Started Now →](https://github.com/Mullassery/tinybridge#quick-start)
