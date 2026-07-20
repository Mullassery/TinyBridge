# Git Integration & Deployment at Scale with TinyBridge

**How TinyBridge enables git-driven infrastructure and scaled deployments**

---

## Overview

TinyBridge treats infrastructure as code by making `env.yaml` the single source of truth for your development environment. This enables:

- **Git-based workflows** — Version control for infrastructure
- **Team consistency** — Everyone works in identical environments
- **Deployment parity** — Local environments match production
- **Scalable CI/CD** — Fast, reproducible environment provisioning
- **Disaster recovery** — Complete environment history in git

---

## Part 1: Git Integration

### Environment as Code

Your entire development environment lives in one file, checked into git:

```yaml
# env.yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: my-project
  version: "1.2.3"
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
    - postgresql@16
    - redis@7.2
```

**Benefits:**
- ✅ Entire environment definition in one file
- ✅ Diff-able (see what changed between versions)
- ✅ Version-controlled (history of all environment changes)
- ✅ Reviewable (PRs to change infrastructure)
- ✅ Auditable (who changed what, when)

### Workflow: Proposing Environment Changes

**Developer wants to upgrade Python:**

```bash
# Create a branch
git checkout -b upgrade-python-311

# Edit env.yaml
# Change: python@3.10 → python@3.11

# Commit
git add env.yaml
git commit -m "upgrade: python 3.11 for f-strings"

# Push and create PR
git push origin upgrade-python-311
```

**Code review:**
```bash
# Reviewer sees exactly what changed
# Before: python@3.10
# After:  python@3.11

# Can test locally with the new version
git checkout upgrade-python-311
tinybridge up  # Gets Python 3.11

# Merges PR
# All other team members now get Python 3.11
```

**Why this is better than current approaches:**
- ❌ Old way: "Someone mention to everyone we're upgrading Python" (someone forgets)
- ❌ Old way: Docker rebuild is expensive, silently fails in one environment
- ✅ New way: One-line change, git history, reproducible

### Versioning Environments

Track environment versions alongside code releases:

```bash
# Tag environment version with release
git tag -a v1.2.3 -m "Release with Python 3.11 support"

# Later, check out exact environment from 6 months ago
git checkout v1.1.0
tinybridge up  # Gets Python 3.10, Node 18, etc from that release

# Run production debugging with exact local setup
```

**Use cases:**
- Reproduce bugs from old versions
- Test backward compatibility
- Verify migration path
- Audit environment changes around incidents

### Environment Branching

Different branches can have different environments:

```bash
# Main branch (production-ready)
# env.yaml: Ubuntu 24.04, Python 3.11, strict versions

# dev branch (experimental)
# env.yaml: Ubuntu 24.04, Python 3.12-rc, bleeding-edge tools

# feature/ml-pipeline (specialized)
# env.yaml: Ubuntu 24.04, Python 3.11, TensorFlow 2.14, CUDA 12

# Developers switch branches and get appropriate environment
git checkout feature/ml-pipeline
tinybridge up  # Gets ML-optimized environment
```

---

## Part 2: Deployment at Scale

### Single Source of Truth

Your `env.yaml` defines the environment for:

1. **Local development** — Developer's MacBook
2. **CI/CD pipelines** — GitHub Actions, GitLab CI, Jenkins
3. **Staging servers** — Pre-production environment
4. **Production** — Same OS, tools, versions as local

**Example CI/CD:**

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup environment
        run: |
          # Read env.yaml and provision identical environment
          tinybridge up test-env --config env.yaml
      
      - name: Run tests
        run: tinybridge exec test-env "pytest tests/"
      
      - name: Build
        run: tinybridge exec test-env "cargo build --release"
```

**Impact:**
- Tests run in exact same environment as developer's machine
- No "works locally but fails in CI" surprises
- Environment changes automatically tested
- New developers don't need CI-specific setup instructions

### Parallel Environment Provisioning

Deploy multiple environments simultaneously:

```bash
# Production deployment: boot API + DB + Cache in parallel
tinybridge up api-prod &
tinybridge up db-prod &
tinybridge up cache-prod &
wait

# All three running independently, immediately ready
```

**Scalability benefits:**
- Each environment can be tuned for its role (API: 8 CPU, DB: 16 CPU)
- Resource isolation (one environment crashing doesn't affect others)
- Fast provisioning (parallel startup)
- Independent scaling (upgrade one without touching others)

### Environment Templates

Maintain standard environment templates for different use cases:

```bash
# templates/backend.yaml
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: backend-template
substrate:
  os: ubuntu
  version: "24.04"
resources:
  cpu: 8
  memory: 16GB
  disk: 100GB
native:
  tools:
    - python@3.11
    - postgresql@16
    - redis@7.2
    - docker
```

**Usage:**
```bash
# Create new project from template
cp templates/backend.yaml myservice/env.yaml

# Customize for this service
sed -i 's/backend-template/myservice-prod/' myservice/env.yaml

# Deploy
cd myservice && tinybridge up
```

---

## Part 3: Team & Organization Scale

### Team Onboarding

**Old way:**
```
New developer receives:
- 10-page onboarding doc
- "Run these commands..."
- "Install these tools..."
- "Set this environment variable..."
- Help from senior dev when something breaks
```

**TinyBridge way:**
```bash
# New developer clones repo
git clone https://github.com/mycompany/backend.git
cd backend

# One command
tinybridge up

# Identical environment to everyone else
# No missing dependencies
# No version mismatches
# No environment setup help needed
```

**Onboarding improvement:** Eliminates manual setup steps per developer. Scale across teams.

### Cross-Team Collaboration

Teams can share environments via git branches:

```bash
# Frontend team creates API mock environment
git checkout -b feature/api-mock

# In env.yaml: Add mock-server tool
# frontend-team/env.yaml
mock-server@1.2.0

# Backend team pulls the branch
git fetch origin feature/api-mock:api-mock
git checkout api-mock

# Backend developers now have mock server
# Can test against exact frontend expectations
tinybridge up
```

### Environment Change Review

Every infrastructure change goes through code review:

```bash
# PR: "Add PostgreSQL to development environment"
# Reviewer sees:
- What changed (postgres added)
- Why (new feature needs database)
- Version (postgresql@16)
- Resource impact (added 2GB memory)

# Reviewer can:
- Test locally with the change
- Verify tests pass with new setup
- Ask questions
- Approve or request changes

# Once merged, all developers get the change automatically
```

---

## Part 4: CI/CD Integration

### Environment Parity in Pipelines

Every CI/CD job uses the same `env.yaml`:

```bash
# No divergence between local and CI environments
# No "works on my machine" failures
# No environment-related flakes
```

### Test Matrix Scaling

Run tests across multiple environment configurations:

```yaml
# .github/workflows/test-matrix.yml
strategy:
  matrix:
    python-version: ["3.10", "3.11", "3.12"]
    os: ["ubuntu-22.04", "ubuntu-24.04"]

steps:
  - run: |
      tinybridge up test-env \
        --substrate-os ubuntu \
        --substrate-version ${{ matrix.os-version }} \
        --override python@${{ matrix.python-version }}
      
      tinybridge exec test-env "pytest tests/"
```

**Result:** Test every supported Python version × OS combination automatically.

### Deployment Pipeline Example

```bash
# 1. Developer pushes to feature branch
git push origin feature/new-api

# 2. GitHub Actions runs
# - Provisions environment from env.yaml
# - Runs tests
# - Builds binary
# - Pushes to staging registry

# 3. Staging deployment
# - Reads env.yaml
# - Provisions staging environment with same specs
# - Deploys binary
# - Runs smoke tests

# 4. Production deployment
# - Same env.yaml ensures production matches staging
# - Deploys with confidence
# - Rollback available (env version in git)
```

---

## Part 5: Disaster Recovery & Audit Trail

### Incident Investigation

When something breaks in production:

```bash
# Check environment used at time of incident (git history)
git log --until="2026-07-20" -1 env.yaml

# Boot exact environment from that time
git checkout <commit-hash>
tinybridge up

# Reproduce the issue locally
# Verify fix works
# Deploy fix
```

### Rollback Capability

Every environment change is reversible:

```bash
# Something went wrong after changing tools
git revert <commit-hash>

# All environments revert to previous setup
tinybridge up

# System stable again
```

### Audit Trail

Complete history of who changed what:

```bash
git log --oneline env.yaml
# 3a2b1c0 - chore: upgrade postgresql 15 → 16 (alice)
# 2f1e9d8 - feat: add redis for caching (bob)
# 1c0a9f7 - fix: python 3.10 → 3.11 for security patch (charlie)

git show 3a2b1c0
# Shows exact change, commit message, timestamp, author
```

---

## Part 6: Scaling Patterns

### Microservices Architecture

Each service has its own env.yaml:

```
monorepo/
├── services/
│   ├── api/
│   │   └── env.yaml (Python 3.11, PostgreSQL 16, Redis)
│   ├── worker/
│   │   └── env.yaml (Python 3.11, Celery, Redis)
│   └── frontend/
│       └── env.yaml (Node 20, npm, webpack)
├── shared/
│   └── env.yaml (Common tools)
```

**Deployment:**
```bash
# Deploy all services with their specific environments
for service in api worker frontend; do
  tinybridge up $service --config services/$service/env.yaml &
done
wait

# Each runs with optimized settings
```

### Regional Deployments

Different regions can have region-specific configurations:

```bash
# us-east region (prod)
git checkout prod-us-east
tinybridge up api-us-east

# eu-west region (prod)
git checkout prod-eu-west
tinybridge up api-eu-west

# Each region can have different:
# - Resource allocations (EU requires more compliance tooling)
# - Tool versions (comply with local regulations)
# - Monitoring (regional data residency)
```

### Blue-Green Deployments

Use git branches for blue-green environment management:

```bash
# Blue (current production)
git checkout main
tinybridge up production-blue

# Green (new version, being tested)
git checkout release-v2.0
tinybridge up production-green

# Run tests against green
# Switch traffic (git merge green → blue)
# Keep blue as rollback point
```

---

## Part 7: Practical Examples

### Example 1: Database Migration

```bash
# Feature branch: add user_roles table
git checkout -b feature/user-roles

# Update env.yaml to trigger migration
# env.yaml: postgresql version unchanged, but migration script added

git add env.yaml
git commit -m "add: user_roles migration"

# Developer tests locally
tinybridge up
# PostgreSQL boots with migration pre-run

# PR merged
# CI runs same migration automatically
# Staging gets same migration
# Production runs same migration (no surprises)
```

### Example 2: Security Patch

```bash
# Critical: OpenSSL vulnerability
git checkout -b hotfix/openssl-2.0.8

# Update env.yaml
# Change: base ubuntu version to trigger patch

git add env.yaml
git commit -m "security: update OpenSSL CVE-2024-xxxx"

# All environments (local, CI, staging, prod) get patched
# Git history shows exactly when and why patch applied
# Easy to verify all systems patched
```

### Example 3: Onboarding a New Developer

```bash
# Day 1: New developer joins
git clone https://github.com/mycompany/project.git
cd project

tinybridge up

# Entire environment ready
# Same Python version as everyone
# Same PostgreSQL version as production
# Same Redis version as staging
# All dependencies installed
# Ready to contribute (no setup needed)
```

---

## Part 8: Benefits Summary

### For Developers
✅ Reproducible environments  
✅ No "works on my machine" surprises  
✅ Fast onboarding  
✅ Easy collaboration  

### For DevOps/Platform Teams
✅ Single source of truth  
✅ Audit trail of all changes  
✅ Disaster recovery  
✅ Scalable deployments  

### For Organizations
✅ Reduced onboarding time  
✅ Faster CI/CD  
✅ Better incident response  
✅ Compliance & audit trails  

### For Security
✅ Version lock-in (no surprise upgrades)  
✅ Complete audit history  
✅ Vulnerability patch tracking  
✅ Compliance verification  

---

## Part 9: When to Use

### TinyBridge Git Integration is ideal for:

✅ **Teams** — Shared environments via git  
✅ **Microservices** — Each service has its own env.yaml  
✅ **CI/CD** — Reproducible builds and tests  
✅ **Compliance** — Audit trail and version control  
✅ **Incident Response** — Easy rollback and reproduction  
✅ **Onboarding** — New developers get full setup in one command  

### Not ideal for:

❌ **One-person projects** — env.yaml overhead not justified  
❌ **Complex Kubernetes clusters** — use Helm for orchestration  
❌ **Monolithic legacy apps** — env.yaml per monolith might be excessive  

---

## Part 10: Getting Started

### Enable Git Integration

1. **Add env.yaml to repository:**
```bash
cat > env.yaml << 'EOF'
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
EOF

git add env.yaml
git commit -m "infrastructure: add TinyBridge environment configuration"
git push
```

2. **Create CI/CD integration:**
```bash
# Add to your CI/CD pipeline
# (see examples above for your platform)
```

3. **Share with team:**
```bash
# Team pulls repo and boots environment
git clone https://github.com/mycompany/project.git
cd project
tinybridge up
```

---

## Conclusion

TinyBridge's `env.yaml` in git turns infrastructure from a manually-managed mess into a version-controlled, reviewable, auditable asset.

This enables:
- **Fast deployment** at scale
- **Reliable reproduction** of any environment state
- **Team consistency** without manual coordination
- **Compliance & audit** trails automatically
- **Disaster recovery** via git history

The result: Your entire development infrastructure, portable, reviewable, and version-controlled.

---

## Next Steps

- **[Getting Started](../GETTING_STARTED.md)** — Boot your first environment
- **[User Guide](../USER_README.md)** — Complete command reference
- **[Architecture](./ARCHITECTURE.md)** — How it works under the hood
