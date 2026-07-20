# TinyBridge

**Run Linux on your Mac. Instantly. For free.**

Stop switching between macOS and Linux. Run a full Ubuntu environment on your MacBook. No Docker complexity. No vendor lock-in. No waiting.

---

## What You Get

### 📝 Environment-as-Code
Define your Linux environment in one file. Share with your team. Everyone gets the same setup.

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
    - python@3.11
    - nodejs@20
    - docker
```

Check `env.yaml` into git. Your teammates run `tinybridge up my-project` and instantly get the identical environment.

### ⚡ Instant Linux Shell
Once your environment is defined, boot it in one command.

```bash
tinybridge up my-project
# ✓ Running (SSH ready)

ssh vm@my-project
ubuntu@my-project:~$
```

Your Linux environment is ready to use with optimized boot architecture.

### 🔄 Automatic File Sync
Your Mac's files are automatically available in Linux. Edit on macOS, run on Linux.

```bash
# On macOS
$ echo "hello" > ~/myproject/test.txt

# In Linux (automatic)
$ cat ~/test.txt
hello
```

No mounting. No configuration. Just works.

### 🚀 Run Multiple Environments
Work on multiple projects in parallel. Each environment is isolated, fast, and independent.

```bash
tinybridge up frontend
tinybridge up backend  
tinybridge up database

# All three running simultaneously
```

### 📈 Scales Naturally From Dev to Team
Start with a single environment on your Mac. When your team grows, scaling is built-in:

```bash
# Single developer: define your env.yaml locally
tinybridge up myproject

# Team scale: commit env.yaml to git
git add env.yaml
git push

# Your teammates: identical environments instantly
git pull
tinybridge up myproject

# Organization scale: templates for common stacks
tinybridge create --template backend  # Python + Postgres
tinybridge create --template ml       # PyTorch + Jupyter
tinybridge create --template robotics # ROS 2 + tools
```

Because environments are declarative YAML files in git, sharing, templating, and scaling from personal to team/org deployments requires no additional infrastructure.

### 🎯 Match Production Exactly
Your production runs Ubuntu 24.04? Your local environment runs Ubuntu 24.04. Same OS, same tools, same behavior. No "works on my Mac" surprises.

Supports: Ubuntu, Debian, Alpine, Fedora (any version).

### 🔐 Open Source, Forever Free
Apache 2.0 licensed. No subscriptions. No license costs. Read the code. Fork it. Run it forever.

---

## Install

### Homebrew (Recommended)
```bash
brew install --cask tinybridge
tinybridge --version
```

### Manual Download
1. Download from [GitHub Releases](https://github.com/Mullassery/tinybridge/releases)
2. Drag TinyBridge to Applications
3. Run it

### Python Projects
```bash
uv tool install tinybridge
```

---

## Get Started

Create an environment file:

```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: demo
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 4
  memory: 8GB
  disk: 50GB
```

Start your environment:

```bash
tinybridge up demo
tinybridge shell demo
```

You're now in Ubuntu:

```bash
ubuntu@demo:~$ python3 --version
Python 3.11.9
```

That's it. See [Getting Started](GETTING_STARTED.md) for a complete walkthrough.

---

## Real-World Use Cases

### Backend Developers
Stop context-switching between your Mac and production Linux. Develop locally in the exact Linux your code runs on.

### DevOps Teams
Share environment configurations via git. No more "it works on my machine" when deploying.

### ML Engineers
Run Python on Linux with access to Linux-only packages. Mount your training data directly.

### Robotics Teams
ROS 2 development environment that matches your robot's OS.

### Data Scientists
Run Spark, Postgres, and Kafka locally on Linux while writing code on macOS.

---

## Why TinyBridge vs. Alternatives?

| | TinyBridge | Docker Desktop | Lima |
|---|---|---|---|
| **Speed** | Seconds to shell | 10-20s | 8-15s |
| **Price** | Free | $7/month | Free |
| **Config** | One YAML file | Multiple files | YAML + scripts |
| **Open Source** | ✅ Yes | Partial | ✅ Yes |
| **Mac Integration** | ✅ Native app | Heavy | CLI only |
| **Parallel Envs** | ✅ Easy | Complex | Difficult |
| **File Sync** | Automatic | Slow | Manual |

---

## Quick Command Reference

| What | Command |
|------|---------|
| Start environment | `tinybridge up myenv` |
| Enter shell | `tinybridge shell myenv` |
| Run command | `tinybridge exec myenv "python train.py"` |
| Check status | `tinybridge status myenv` |
| Stop environment | `tinybridge down myenv` |
| List all | `tinybridge list` |

---

## Next Steps

1. **[Getting Started](GETTING_STARTED.md)** — Step-by-step walkthrough
2. **[Full Documentation](USER_README.md)** — Complete command reference
3. **[Git & Deployment Guide](docs/GIT_DEPLOYMENT_GUIDE.md)** — Version control and scaling
4. **[See the Roadmap](PRODUCT_VISION.md)** — What's coming next

---

## Frequently Asked Questions

**Q: Is this like Docker?**  
A: Similar idea, but simpler and faster. One environment per YAML file. Real Linux VM, not containers. No images, no layers.

**Q: Do I need Docker installed?**  
A: No. TinyBridge is independent and doesn't require Docker, VirtualBox, or any other tool.

**Q: Can I run GUI applications?**  
A: Phase 1 focuses on CLI/server development. GUI support coming in Phase 5.

**Q: What about GPU support?**  
A: Coming in Phase 4 (2027). For now, use remote GPU services via transparent routing.

**Q: Can my team share environments?**  
A: Yes. Check `env.yaml` into git. Your teammates run `tinybridge up`. Everyone gets identical setup.

**Q: Will my code run the same on Linux as on my Mac?**  
A: For Python, Go, Node: yes, exactly the same. For C/C++, minor differences due to Linux vs macOS libraries.

**Q: Is this production-ready?**  
A: Phase 1 is stable for development. Not yet recommended for running live services.

**Q: How much does this cost?**  
A: Free. Apache 2.0 license. No subscriptions, ever.

**Q: Can I contribute?**  
A: Yes! Open source on GitHub. Issues and PRs welcome.

---

## Status

🚧 **Phase 1: Foundations** (Stable for development)
- Linux environment with SSH access
- File syncing via VirtioFS  
- Environment-as-Code configuration
- Multi-distro support (Ubuntu, Debian, Alpine, Fedora)

🗓️ **Coming Next:**
- Phase 2: Smart environment templates
- Phase 3: Advanced networking (ROS 2 DDS native)
- Phase 4: GPU support
- Phase 5: Plugin ecosystem

---

## Get Help

- **[Getting Started Guide](GETTING_STARTED.md)** ⭐ Start here
- **[Troubleshooting](USER_README.md#troubleshooting)** Common issues
- **[Issues](https://github.com/Mullassery/tinybridge/issues)** Report a bug
- **[Discussions](https://github.com/Mullassery/tinybridge/discussions)** Ask questions

---

## License

Apache License 2.0 — Completely open source. Read the [LICENSE](LICENSE) file.

---

**TinyBridge — Linux development on macOS. No pain. No waiting.**
