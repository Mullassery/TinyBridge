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

---

## Adjusting Resources While Running

You can change CPU and memory allocation for a running environment without losing state. Changes take effect after restart.

### Method 1: Update env.yaml (Recommended)

Edit your `env.yaml` file and change the resource values:

```yaml
resources:
  cpu: 4              # Change from 4
  memory: 8GB         # Change from 8GB
  disk: 50GB
```

To:

```yaml
resources:
  cpu: 8              # Now 8 cores
  memory: 16GB        # Now 16GB RAM
  disk: 50GB
```

Then restart the environment:

```bash
# Stop the environment (preserves all files and state)
tinybridge down myenv

# Start with new resources
tinybridge up myenv
```

Your environment will boot with the new resource allocation. All files remain intact.

### Method 2: Command-Line Update (While Running)

Update resources without stopping the environment:

```bash
# Check current allocation
tinybridge status myenv

# Increase CPU to 8 cores
tinybridge update myenv --cpu 8

# Increase memory to 16GB
tinybridge update myenv --memory 16GB

# Both at once
tinybridge update myenv --cpu 8 --memory 16GB
```

Changes apply immediately to new processes. Running processes keep their original resources until restart.

For a clean restart with new resources:
```bash
tinybridge restart myenv
```

### Verifying Resource Changes

Check the new allocation:

```bash
# Show environment details
tinybridge status myenv

# Inside the environment
tinybridge shell myenv
ubuntu@myenv:~$ nproc           # Show CPU cores
8
ubuntu@myenv:~$ free -h         # Show memory
              total        used        free      shared  buff/cache   available
Mem:            16Gi       2.1Gi       12Gi       0.0Gi       1.8Gi       13Gi
```

### Resource Adjustment Best Practices

**When to increase resources:**
- Machine learning training is slow → increase CPU + memory
- Database operations are memory-constrained → increase memory
- Compilation takes too long → increase CPU cores
- Running multiple services → increase memory

**When to decrease resources:**
- Reduce macOS slowdown → free up unused cores
- Free up RAM for other Mac apps
- Testing resource-constrained deployments → simulate production limits

**Safe limits:**
- Keep at least 2 cores and 4GB RAM free on your Mac for macOS
- Don't allocate more than 75% of your Mac's resources to any one environment
- Example on 8-core, 16GB Mac: max `cpu: 6` and `memory: 12GB` per environment

### Example: Progressive Resource Allocation

Start small, then scale up as needed:

```bash
# Phase 1: Initial development (minimal resources)
tinybridge up myenv  # Uses default: 4 cores, 8GB

# Phase 2: Performance testing (increase resources)
tinybridge update myenv --cpu 8 --memory 16GB

# Phase 3: Production simulation (match production specs)
tinybridge update myenv --cpu 16 --memory 32GB

# Phase 4: Back to development (reduce resources)
tinybridge update myenv --cpu 4 --memory 8GB
```

### Multiple Environments with Different Resources

You can run multiple environments with different allocations:

```bash
# Environment 1: ML training (high resources)
tinybridge up ml-training    # Has: cpu: 8, memory: 16GB (from env.yaml)

# Environment 2: Web development (standard resources)
tinybridge up backend         # Has: cpu: 4, memory: 8GB (from env.yaml)

# Environment 3: Database testing (memory-heavy)
tinybridge up database        # Has: cpu: 2, memory: 24GB (from env.yaml)

# All three running simultaneously with their own allocations
tinybridge list
```

See [Running Multiple Environments](README.md#-multiple-parallel-environments) for more details.

---

## Running Multiple Parallel Environments

TinyBridge lets you run multiple isolated Linux environments simultaneously on the same Mac. Each environment has its own:
- CPU cores and memory allocation
- Filesystem (~/myprojectname is independent from ~/anotherproject)
- Network (separate IP addresses)
- SSH access
- Running processes

### Quick Start: Run 3 Environments in Parallel

Create 3 separate `env.yaml` files:

**Project 1: Web Backend** (`backend/env.yaml`)
```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: backend
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 4
  memory: 8GB
  disk: 50GB
```

**Project 2: ML Training** (`ml/env.yaml`)
```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: ml-training
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 8              # More cores for ML
  memory: 16GB        # More memory for models
  disk: 100GB         # Larger disk for datasets
```

**Project 3: Database** (`database/env.yaml`)
```yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: database
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 2
  memory: 12GB        # Memory-optimized
  disk: 50GB
```

Start all three:

```bash
# Terminal 1: Start backend
cd backend && tinybridge up backend

# Terminal 2: Start ML training
cd ml && tinybridge up ml-training

# Terminal 3: Start database
cd database && tinybridge up database

# All three are now running simultaneously
```

Verify all are running:

```bash
tinybridge list
```

Output:
```
Name              Status          Uptime    IP Address
backend           ✓ Running       2m30s     192.168.105.2
ml-training       ✓ Running       1m45s     192.168.105.3
database          ✓ Running       30s       192.168.105.4
```

### Working with Multiple Environments

Each environment is independent. Operate on them individually:

```bash
# Open shell in backend
tinybridge shell backend
ubuntu@backend:~$ python app.py

# Open shell in ML training (different terminal)
tinybridge shell ml-training
ubuntu@ml-training:~$ python train.py

# Open shell in database (different terminal)
tinybridge shell database
ubuntu@database:~$ psql postgres
```

Files are isolated per environment:

```bash
# Files in ~/backend are only in backend environment
echo "backend data" > ~/backend/data.txt
tinybridge shell backend
ubuntu@backend:~$ cat ~/data.txt
backend data

# Files in ~/ml-training are only in ML environment
echo "ml data" > ~/ml-training/data.txt
tinybridge shell ml-training
ubuntu@ml-training:~$ cat ~/data.txt
ml data
```

### Managing Multiple Environments

Stop individual environments:

```bash
# Stop just the backend
tinybridge down backend

# ml-training and database still running
tinybridge list
# Shows: ml-training ✓ Running, database ✓ Running

# Start backend again
tinybridge up backend
```

Stop all at once:

```bash
tinybridge down backend && tinybridge down ml-training && tinybridge down database
```

Update resources for one environment:

```bash
# Increase ML training memory (while it's running)
tinybridge update ml-training --memory 24GB

# Or restart with new resources
tinybridge restart ml-training
```

### Resource Planning for Multiple Environments

**On an 8-core, 16GB Mac:**

Good configuration:
```yaml
backend:
  cpu: 3
  memory: 6GB

ml-training:
  cpu: 3
  memory: 6GB

database:
  cpu: 2
  memory: 4GB

# Total: 8 cores, 16GB RAM (leaves Mac at minimum)
```

**On a 10-core, 32GB Mac:**

Aggressive configuration:
```yaml
backend:
  cpu: 4
  memory: 8GB

ml-training:
  cpu: 4
  memory: 16GB

database:
  cpu: 2
  memory: 8GB

# Total: 10 cores, 32GB RAM
# Safe: leaves some headroom
```

### Networking Between Environments

Environments have separate IP addresses but can communicate:

```bash
# Find IPs
tinybridge list

# In backend environment
tinybridge shell backend
ubuntu@backend:~$ ping 192.168.105.3  # ML training IP
ubuntu@backend:~$ curl http://192.168.105.4:5432  # Database IP
```

### Use Cases for Multiple Environments

**Microservices Development:**
```bash
tinybridge up api-service      # 4 cores, 8GB
tinybridge up frontend-service # 2 cores, 4GB
tinybridge up database         # 2 cores, 8GB
# Run and test all services together
```

**Machine Learning Pipeline:**
```bash
tinybridge up data-prep        # 4 cores, 8GB
tinybridge up model-training   # 8 cores, 16GB
tinybridge up inference        # 4 cores, 8GB
# Run data pipeline, training, and inference separately
```

**Testing Against Multiple OS Versions:**
```bash
tinybridge up ubuntu-24.04  # Latest Ubuntu
tinybridge up ubuntu-22.04  # LTS version
tinybridge up debian-12     # Debian testing
# Test code across different distributions
```

**Team Collaboration:**
```bash
tinybridge up alice-env    # Alice's project environment
tinybridge up bob-env      # Bob's project environment
# Different devs on same Mac, isolated setups
```

### Permanently Removing Environments

There are different ways to remove an environment, depending on whether you want to keep the data:

#### Option 1: Soft Stop (Preserves All Data)

Stop the environment but keep files and data:

```bash
tinybridge down myprojectname
```

The environment is stopped but:
- ✅ All files in `~/myprojectname/` are preserved
- ✅ Environment state is saved
- ✅ You can restart anytime: `tinybridge up myprojectname`
- ✅ env.yaml still controls the environment

**Use this when:** You're pausing development but might come back to it.

#### Option 2: Hard Delete (Remove Everything)

Completely delete an environment and all its data:

```bash
# First stop the environment
tinybridge down myprojectname

# Then delete it permanently
tinybridge delete myprojectname --force
```

This removes:
- ✗ The environment VM
- ✗ All environment metadata
- ✗ Associated system files
- ✗ Everything in `~/.tinybridge/data/myprojectname`

But **preserves:**
- ✅ Files in `~/myprojectname/` on your Mac (these stay safe)
- ✅ Your env.yaml file (if in git or elsewhere)
- ✅ Version control history

**Use this when:** You're completely done with a project and want to free resources.

#### Option 3: Clean Up All Environments

Remove all stopped environments at once:

```bash
# List all environments
tinybridge list

# Remove all (after confirming which ones to delete)
tinybridge cleanup --all
```

This only removes stopped environments. Running environments are not affected.

### Comparison: Stop vs. Delete

| Action | Command | Environment State | Mac Files | Can Restart? |
|--------|---------|-------------------|-----------|--------------|
| **Pause** | `tinybridge down name` | Stopped (saved) | Preserved | ✅ Yes |
| **Delete** | `tinybridge delete name --force` | Deleted | Preserved | ❌ No (must recreate) |
| **Cleanup** | `tinybridge cleanup --all` | Deleted (stopped only) | Preserved | ❌ No |

### Saving and Pausing Environments for Later

You can pause an environment and resume it exactly where you left off—all files, processes, and configuration preserved.

#### Simple Pause: Stop and Resume Later

This is the easiest way to save an environment:

```bash
# Stop the environment (saves all state)
tinybridge down myprojectname

# Environment is now paused
# All files are preserved
# No changes needed to resume

# ... days or weeks later ...

# Resume where you left off
tinybridge up myprojectname

# Entire environment boots with all files intact
```

**What's preserved:**
- ✅ All files in `~/myprojectname/`
- ✅ Environment configuration (env.yaml)
- ✅ Installed packages and tools
- ✅ System state
- ✅ SSH access

**What's NOT preserved:**
- Running processes (new ones start fresh)
- Memory/RAM (apps need to reload data)
- Temporary files in `/tmp`

**Disk usage while paused:**
- Environment still uses disk space (~50GB for default Ubuntu)
- Mac files are unaffected

#### Checkpointing: Save Progress at Specific Points

Create snapshots at important milestones:

```bash
# Before running long training
tinybridge checkpoint myprojectname --name "pre-training"

# Run your ML training
tinybridge shell myprojectname
ubuntu@myprojectname:~$ python train.py    # Takes 4 hours

# Training complete - save checkpoint
tinybridge checkpoint myprojectname --name "training-complete"

# Experiment with new hyperparameters
# Something goes wrong...

# Restore to last good checkpoint
tinybridge restore myprojectname --from "training-complete"

# Now you're back to the successful training state
```

List saved checkpoints:

```bash
tinybridge checkpoints myprojectname
```

Output:
```
Environment: myprojectname
Checkpoints:
  pre-training           Created: 2 hours ago
  training-complete      Created: 1 hour ago
  experiment-v1          Created: 30 minutes ago
```

#### Backup Strategy: Save Important Files

Before pausing long-term, back up critical files:

```bash
# Backup to Mac
cp -r ~/myprojectname ~/myprojectname-backup-$(date +%Y%m%d)

# Or backup to external drive
cp -r ~/myprojectname /Volumes/ExternalDrive/backups/

# Or backup to cloud
rsync -av ~/myprojectname ~/Dropbox/backups/
```

Then safely pause:

```bash
tinybridge down myprojectname
```

#### Long-Term Storage: Archive Before Deletion

If you need to free disk space but want to preserve everything:

```bash
# 1. Back up the entire environment to external storage
tar -czf ~/myprojectname-archive.tar.gz ~/myprojectname

# 2. Move to external drive or cloud
mv ~/myprojectname-archive.tar.gz /Volumes/Archive/

# 3. Delete the environment to free disk space
tinybridge delete myprojectname --force

# 4. Later, restore when needed
tar -xzf /Volumes/Archive/myprojectname-archive.tar.gz -C ~/
tinybridge up myprojectname
```

#### Pause vs. Delete: Decision Matrix

| Need | Action | Disk Space | Resume Time | Data Loss | Cost |
|------|--------|-----------|------------|-----------|------|
| **Quick break (1-7 days)** | `tinybridge down` | ~50GB | Instant restart | None | Low |
| **Extended pause (weeks)** | `down` + backup | ~50GB + backup | Instant restart | None | Medium |
| **Long-term archive (months)** | Delete + archive | Archive size only | Need to recreate | No (backed up) | Low |
| **Never coming back** | Delete | Freed | N/A | OK | Low |

#### Resuming After a Long Pause

When you come back after a long pause:

```bash
# 1. Restart the environment
tinybridge up myprojectname

# 2. Verify everything is intact
tinybridge shell myprojectname
ubuntu@myprojectname:~$ ls          # Files still there
ubuntu@myprojectname:~$ git status  # Repo still there

# 3. Update system (recommended after long pause)
ubuntu@myprojectname:~$ sudo apt update && sudo apt upgrade -y

# 4. Reinstall development dependencies if needed
ubuntu@myprojectname:~$ pip install -r requirements.txt
```

#### Disk Space Monitoring While Paused

Check how much space paused environments use:

```bash
# Show all environments and sizes
tinybridge info

# Output:
# Environment        Status    Size      
# backend           Stopped   45GB      
# ml-training       Stopped   120GB     
# database          Stopped   35GB      
# Total Used:               200GB      

# Free space available: 500GB
```

Free up space by deleting paused environments you don't need:

```bash
# Delete old paused environment
tinybridge delete old-project --force

# Check reclaimed space
tinybridge info
```

#### Multi-Environment Pause Strategy

Efficiently manage multiple paused environments:

```bash
# You have 5 projects, only work on 3 at a time
tinybridge up project-a       # Active
tinybridge up project-b       # Active
tinybridge up project-c       # Active

# Pause projects you're not using now
tinybridge down project-d     # Paused (still using disk)
tinybridge down project-e     # Paused (still using disk)

# When disk space is low, delete paused projects
tinybridge delete project-d --force    # Free ~50GB

# Can recreate if needed
tinybridge up project-d                # Recreates from env.yaml
```

#### Real-World Example: Research Project Lifecycle

```bash
# Week 1: Active development
tinybridge up research-2024
tinybridge shell research-2024
ubuntu@research:~$ python experiment.py

# Week 2: Pause while focusing on other work
tinybridge checkpoint research-2024 --name "week-1-complete"
tinybridge down research-2024              # Paused, disk still used

# Week 5: Come back to research
tinybridge up research-2024                # Instantly restore
tinybridge shell research-2024
ubuntu@research:~$ ls                      # All files intact

# Week 8: Archive before deleting
tar -czf research-2024-final.tar.gz ~/research-2024
cp research-2024-final.tar.gz ~/Dropbox/archive/
tinybridge delete research-2024 --force    # Free disk space

# Year later: Need to revisit
tar -xzf ~/Dropbox/archive/research-2024-final.tar.gz
tinybridge up research-2024                # Recreate with all data
```

---

## Clarifying: Save/Close vs. Permanent Termination

This is the most important distinction to understand:

### Quick Reference Table

| Operation | Command | Environment Still Exists? | Can Restart? | Disk Space | When to Use |
|-----------|---------|--------------------------|--------------|-----------|-------------|
| **Save & Close (Pause)** | `tinybridge down` | ✅ Yes | ✅ Yes, instantly | ~50GB used | Taking a break, coming back later |
| **Checkpoint** | `tinybridge checkpoint --name "x"` | ✅ Yes | ✅ Yes, from checkpoint | ~50GB + snapshots | Before risky experiments |
| **Permanent Delete** | `tinybridge delete --force` | ❌ No | ❌ No (must recreate) | Freed | Done with project, need disk space |
| **Cleanup All** | `tinybridge cleanup --all` | ❌ No (stopped only) | ❌ No | Freed | Removing multiple unused environments |

### Visual Workflow

```
┌─ RUNNING ENVIRONMENT ─┐
│  tinybridge up myenv  │
│  Environment: Active  │
│  Disk: ~50GB used     │
└──────────────────────┘
         │
         ├─ tinybridge down (PAUSE/SAVE)
         │  └─→ Environment: Paused (Disk: ~50GB still used)
         │      └─→ tinybridge up (RESUME)
         │
         └─ tinybridge delete --force (TERMINATE)
            └─→ Environment: Deleted (Disk: Freed)
                └─→ Cannot resume (must recreate from env.yaml)
```

### When to Use Each

**Use `tinybridge down` (Save/Close) when:**
- ✅ You'll come back to the project later today/this week/this month
- ✅ You want to pause but keep all state intact
- ✅ You're switching between multiple projects (pause one, work on another)
- ✅ You want instant resume with everything exactly as you left it
- ✅ Disk space isn't a concern (environment still uses ~50GB)

Example:
```bash
# Working on frontend
tinybridge down frontend    # Pause it

# Switch to backend
tinybridge up backend       # Start backend

# Later...
tinybridge up frontend      # Resume frontend exactly as it was
```

**Use `tinybridge delete --force` (Terminate) when:**
- ✅ Project is truly done (shipped, archived, or no longer needed)
- ✅ You need to free disk space urgently
- ✅ You're certain you won't need this environment again
- ✅ You have backups or version control of important files
- ✅ You want to remove clutter and keep only active environments

Example:
```bash
# Project is shipped and archived
tinybridge delete shipped-project --force   # Free ~50GB

# Later, if needed:
git clone <repo>
tinybridge up shipped-project               # Recreate fresh from repo
```

### The Critical Difference Explained

**Save/Close (`down`):**
- Environment is **paused**, not deleted
- **All state is preserved** exactly as-is
- Can resume **instantly** with `tinybridge up`
- **Disk space is NOT freed** (still uses ~50GB)
- **Your files remain** in `~/myprojectname/`
- **No data loss** possible

**Terminate (`delete --force`):**
- Environment is **permanently deleted**
- **State is destroyed** (can't resume)
- Must **recreate from env.yaml** to restore
- **Disk space IS freed** (~50GB recovered)
- **Mac files are preserved** in `~/myprojectname/`
- **Data loss possible** if not backed up

### Decision Tree

```
Do you think you'll use this environment again?
│
├─ YES (even in 6 months)
│  └─→ tinybridge down    (SAVE/CLOSE)
│      Keep everything, resume later
│
└─ NO (project done, never coming back)
   │
   └─ Do you have important files to keep?
      │
      ├─ YES
      │  ├─ Back them up first
      │  │  cp -r ~/myenv ~/myenv-backup
      │  │
      │  └─ Then delete
      │     tinybridge delete myenv --force
      │
      └─ NO (just throwaway test)
         └─→ tinybridge delete myenv --force
             (Disk freed, safe to delete)
```

### Common Confusion Resolved

**Q: Is `tinybridge down` the same as deleting?**
A: NO. `down` is a pause/stop. Delete is permanent. Use `down` if you might come back.

**Q: Will my files disappear if I use `tinybridge down`?**
A: NO. Your files in `~/myprojectname/` are always safe. They're on your Mac, not in the environment.

**Q: How do I recover after `tinybridge delete`?**
A: You must recreate the environment:
```bash
# If you have env.yaml backed up:
tinybridge up myprojectname      # Recreates from env.yaml

# If you have files backed up:
cp -r ~/myprojectname-backup/* ~/myprojectname/
```

**Q: Can I delete an environment while it's running?**
A: No, stop it first: `tinybridge down myenv`, then `tinybridge delete myenv --force`

---

### Safe Deletion Workflow

**If you're uncertain, use this safe workflow:**

```bash
# 1. Stop the environment
tinybridge down myprojectname

# 2. Back up your important files
cp -r ~/myprojectname ~/myprojectname.backup

# 3. Back up environment config
cp env.yaml env.yaml.backup

# 4. Now it's safe to delete
tinybridge delete myprojectname --force

# 5. Verify deletion
tinybridge list  # Should not show myprojectname

# 6. Optional: clean up backup after confirming
rm -rf ~/myprojectname.backup
rm env.yaml.backup
```

### Data Recovery After Deletion

If you accidentally deleted an environment:

**Option 1: Recreate from Backup**
```bash
# If you backed up env.yaml
cp env.yaml.backup env.yaml

# Start a new environment with same config
tinybridge up myprojectname

# Restore backed-up files
cp -r ~/myprojectname.backup/* ~/myprojectname/
```

**Option 2: Restore from Git**
```bash
# If your files were in git
git clone <repo>
cd myprojectname

# Recreate the environment
tinybridge up myprojectname
```

**Option 3: Restore from Time Machine** (macOS)
```bash
# Mac files are always in Time Machine (if enabled)
# Restore via Time Machine
# Recover files from ~/myprojectname/
```

### Freeing Disk Space

After deleting environments, free up disk space:

```bash
# Check current disk usage
tinybridge info  # Shows environment sizes

# Remove unused images (after deleting all environments using them)
tinybridge cleanup --images

# Clear cache
tinybridge cleanup --cache
```

### Scenarios: When to Delete vs. Pause

**Scenario 1: Project Complete**
```bash
# Project shipped, archiving for history
tinybridge down myprojectname     # Keep for 6 months
# Later:
tinybridge delete myprojectname --force   # Remove after archival period
```

**Scenario 2: Quick Testing**
```bash
# Test a feature
tinybridge up test-env
# ... test complete ...
tinybridge delete test-env --force     # Clean up immediately
```

**Scenario 3: Resource Constrained**
```bash
# Low on disk space
tinybridge down heavy-ml-training    # Paused but not deleted (100GB)
tinybridge delete heavy-ml-training --force  # Delete to free space
# Can recreate later if needed
```

**Scenario 4: Taking a Break**
```bash
# Going on vacation, will return to project
tinybridge down backend   # Pause, don't delete
# ... 2 weeks later ...
tinybridge up backend     # Resume where you left off
```

---

### Tips for Multiple Environments

**Tip 1: Use Descriptive Names**
- ✅ Good: `backend`, `ml-training`, `postgres-dev`
- ❌ Avoid: `env1`, `env2`, `test`

**Tip 2: Monitor Resources**
```bash
# Check Mac's resource usage while running environments
top
# or
Activity Monitor  # macOS built-in
```

**Tip 3: Use Project-Specific Directories**
```bash
# Keep env.yaml in each project
~/backend/env.yaml
~/ml-training/env.yaml
~/database/env.yaml

# Navigate before starting
cd backend && tinybridge up backend
```

**Tip 4: Automate with Scripts**
```bash
#!/bin/bash
# start-all.sh

cd ~/backend && tinybridge up backend &
cd ~/ml-training && tinybridge up ml-training &
cd ~/database && tinybridge up database &

echo "All environments starting..."
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

### Phase 4: Advanced Networking & Device Support
- Hardware passthrough (USB, serial devices)
- Advanced network policies
- Device isolation

### Phase 5: Ecosystem
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
