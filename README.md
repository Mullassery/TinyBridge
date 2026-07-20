# TinyBridge

**Run Linux on your Mac. Instantly. For free.**

Stop switching between macOS and Linux. Run a full Ubuntu environment on your MacBook. No Docker complexity. No vendor lock-in. No waiting.

---

## Installation (30 seconds)

### Option 1: Homebrew (Recommended)

```bash
brew install --cask tinybridge
```

### Option 2: Manual Download

1. Download latest `.dmg` from [GitHub Releases](https://github.com/Mullassery/tinybridge/releases)
2. Open the file and drag `TinyBridge.app` to Applications
3. Open TinyBridge.app once (registers the CLI)

### Option 3: Python Projects (with UV)

```bash
uv tool install tinybridge
```

---

## Verify Installation

```bash
# Check CLI is accessible
tinybridge --version
# Should output: tinybridge 0.1.0

# Verify daemon starts automatically
ps aux | grep tinybridged
# Should show: /Applications/TinyBridge.app/Contents/MacOS/tinybridged
```

If `tinybridge --version` fails, restart Terminal. The daemon starts automatically on first run.

---

## Get Started in 3 Steps

### Step 1: Create `env.yaml`

In your project directory, create a file named `env.yaml`. **Replace `myprojectname` with your actual project name:**

```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: myprojectname
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
```

Save this file in your project root. That's your entire environment definition.

**Critical:** The `metadata.name` in env.yaml must exactly match the name you use in `tinybridge up <name>`. If your project is called "backend", use `name: backend`. If it's "ml-training", use `name: ml-training`.

### Step 2: Start Your Environment

```bash
# Replace 'myprojectname' with YOUR actual project name
tinybridge up myprojectname
```

First run downloads the Linux image (~500MB, one-time). You'll see:
```
⟳ Starting environment: myprojectname
✓ Environment myprojectname is ready
```

### Step 3: Enter the Linux Shell

```bash
# Use the SAME name as your project
tinybridge shell myprojectname
```

You're now in Ubuntu Linux:
```bash
ubuntu@myprojectname:~$ uname -a
Linux myprojectname 6.12.4-generic #1 SMP ... x86_64 GNU/Linux
ubuntu@myprojectname:~$ python3 --version
Python 3.11.9
```

All your files from macOS are available at `~/`:
```bash
ubuntu@myprojectname:~$ ls
# Lists your macOS home directory
```

---

## What You Get

### 📝 Environment-as-Code
Define your entire Linux setup in one YAML file. Check it into git. Everyone on your team gets the identical environment.

```bash
# In your project (e.g., myprojectname):
git add env.yaml
git push

# Your teammates get the same environment:
git pull
tinybridge up myprojectname  # Uses metadata.name from env.yaml
```

No more "works on my machine". Production matches development exactly.

### ⚡ Instant Linux Shell
Full Ubuntu with optimized boot. SSH-ready in seconds. All your tools pre-configured. Replace `myprojectname` with your actual project name:

```bash
tinybridge up myprojectname
tinybridge shell myprojectname
ubuntu@myprojectname:~$ docker run ubuntu:24.04 bash
```

### 🔄 Automatic File Sync
Files on macOS instantly appear in Linux. Edit on your Mac, run in Linux. No mounting. No configuration. Replace `myprojectname` with your project name:

```bash
# On macOS
$ echo "hello" > ~/myprojectname/test.txt

# In Linux (automatic, via: tinybridge shell myprojectname)
$ cat ~/test.txt
hello
```

### 🚀 Multiple Parallel Environments
Run multiple projects simultaneously. Each isolated and independent. Replace these with YOUR actual project names:

```bash
tinybridge up frontend        # Your frontend project name
tinybridge up backend         # Your backend project name
tinybridge up database        # Your database project name
# All three running at once
```

### 📈 Scales Naturally: Dev → Team → Organization

**Single developer:** (replace `myprojectname` with your project)
```bash
# Define locally in env.yaml with metadata.name: myprojectname
tinybridge up myprojectname
```

**Team scale:** (all teammates use the same project name)
```bash
# Commit env.yaml to git, teammates run:
git pull
tinybridge up myprojectname  # Same name as metadata.name in env.yaml
```

**Organization scale:**
```bash
# Use templates for common stacks
tinybridge create --template backend   # Python + Postgres
tinybridge create --template ml        # PyTorch + Jupyter
tinybridge create --template robotics  # ROS 2 + tools
```

No infrastructure changes. Same `env.yaml` approach at every scale.

### 🎯 Match Production Exactly

Your production runs Ubuntu 24.04? Your development environment runs Ubuntu 24.04. Same OS, same tools, same behavior.

Supports: Ubuntu, Debian, Alpine, Fedora (any version).

### 🛡️ Built-In Anomaly Detection

Automatic monitoring detects issues before they break workflows:
- **Boot regression** — track boot time changes
- **Resource spikes** — detect CPU/memory anomalies  
- **Availability breaches** — know when SSH stops responding
- **Error trends** — identify unusual error patterns
- **Intrusion detection** — suspicious activity patterns

All logged to an immutable audit trail for reproducibility.

### 🔒 Enterprise-Grade Security

Self-aware security built on OpenTelemetry:
- **Tamper-evident logs** — all environment changes tracked
- **Forensics support** — replay and debug incidents
- **Complete observability** — every environment action recorded
- **Compliance ready** — structured event logging for audits

### 📊 OpenTelemetry Integration (Zero Vendor Lock-in)

Full observability with choice of backends. Switch providers anytime without code changes:

```bash
# In env.yaml
observability:
  backend: datadog    # or: prometheus, jaeger, honeycomb, newrelic, splunk, dynatrace, grafana
  sample_rate: 1.0
```

**Standard OTel metrics:**
- Traces (distributed tracing of environment lifecycle)
- Metrics (boot time, resource usage, I/O latency, error rates)
- Logs (structured event logging with full context)

**Supported backends (pick any):**

| Backend | Cost | Use Case |
|---------|------|----------|
| Prometheus | Free | Self-hosted, on-premise |
| Jaeger | Free | Distributed tracing focus |
| Datadog | Paid | Enterprise, multi-cloud |
| New Relic | Paid | Full-stack monitoring |
| Honeycomb | Paid | Observability-first |
| Splunk | Paid | Log aggregation + analysis |
| Dynatrace | Paid | AI-driven insights |
| Grafana Stack | Free | Open source complete stack |

**No lock-in:** Start with free Prometheus. Scale to Datadog later. Migrate to Jaeger next month. All metrics are standard OTel format—no agent changes required.

### 🔐 Open Source, Forever Free

Apache 2.0 licensed. No subscriptions. No license costs. Read the code. Fork it. Run it forever.

---

## Commands Reference

Replace `<projectname>` with your actual project name (must match `metadata.name` in env.yaml):

| Command | Purpose | Example |
|---------|---------|---------|
| `tinybridge up <projectname>` | Start an environment | `tinybridge up myprojectname` |
| `tinybridge shell <projectname>` | Open bash in environment | `tinybridge shell myprojectname` |
| `tinybridge exec <projectname> "cmd"` | Run command in environment | `tinybridge exec myprojectname "python train.py"` |
| `tinybridge list` | Show all environments | `tinybridge list` |
| `tinybridge status <projectname>` | Check environment status | `tinybridge status myprojectname` |
| `tinybridge down <projectname>` | Stop environment | `tinybridge down myprojectname` |

---

## Real-World Use Cases

### Backend Developers
Stop context-switching between your Mac and production Linux. Develop locally in the exact OS your code runs on.

### DevOps Teams
Share environment configurations via git. No more "it works on my machine" when deploying. Identical setups for everyone.

### ML Engineers
Run Python on Linux with all Linux-only packages. Mount training data directly. Full GPU support coming in Phase 4.

### Robotics Teams
ROS 2 development environment that matches your robot's OS. DDS multicast networking works out of the box.

### Data Scientists
Run Spark, Postgres, Kafka locally on Linux while writing code on macOS. No Docker complexity.

---

## Why TinyBridge?

| Feature | TinyBridge | Docker Desktop | Lima |
|---------|---|---|---|
| **Price** | Free | $7/month | Free |
| **Speed** | Optimized | Slower | Similar |
| **Setup** | One YAML file | Multiple configs | YAML + scripts |
| **Open Source** | ✅ Yes | Partial | ✅ Yes |
| **Native Mac App** | ✅ Yes | Heavy | CLI only |
| **Parallel Envs** | ✅ Easy | Complex | Difficult |
| **File Sync** | Automatic | Slow | Manual |
| **GPU Support** | Phase 4 | Limited | No |
| **Observability** | Built-in OTel | Addon | No |

---

## Learn More

- **[Getting Started Guide](GETTING_STARTED.md)** — Step-by-step walkthrough
- **[User Reference](USER_README.md)** — Complete command reference
- **[Architecture](docs/ARCHITECTURE.md)** — Technical deep dive
- **[Testing Report](TESTING_REPORT.md)** — Performance benchmarks (targets pending verification)
- **[GitHub Repository](https://github.com/Mullassery/tinybridge)**

---

## Status

**Phase 1 (Current):** Core VM + CLI + daemon. Boot-optimized Linux environments on macOS.

- ✅ Environment-as-Code (env.yaml)
- ✅ CLI with full keyboard support
- ✅ Automatic file sync (VirtioFS)
- ✅ Multiple parallel environments
- ✅ OpenTelemetry integration
- 🔄 Performance benchmarking (architecture validated, metrics collection in progress)

**Future Phases:**
- Phase 2: Execution routing + templates
- Phase 3: Hardware passthrough + DDS networking
- Phase 4: Remote GPU routing
- Phase 5: GPU bridge + plugin ecosystem

---

## Support

- **[Issues](https://github.com/Mullassery/tinybridge/issues)** — Report bugs or request features
- **[Discussions](https://github.com/Mullassery/tinybridge/discussions)** — Ask questions, share workflows
- **[Email](mailto:mullassery@gmail.com)** — Direct support

---

**Apache 2.0 License** | Built with Rust + Swift | For macOS 14+
