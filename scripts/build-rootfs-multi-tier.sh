#!/bin/bash
# Build Ubuntu 24.04 minimal rootfs with multi-tier lazy loading optimization
#
# Tier 1 (1.5s): SSH ready — kernel, VirtioFS, sshd
# Tier 2 (5s): Usable system — DNS, logging, manager in background
# Tier 3 (120s): Complete system — caches, monitoring, tools
# Tier 4 (on-demand): Optional services masked

set -euo pipefail

ROOTFS_DIR="${ROOTFS_DIR:-.tinybridge-rootfs}"
UBUNTU_VERSION="24.04"

echo "Building Ubuntu $UBUNTU_VERSION rootfs with multi-tier lazy loading..."
echo "Target: 1.5s SSH ready, 5s usable, 120s complete"

# Create build directory
mkdir -p "$ROOTFS_DIR"
cd "$ROOTFS_DIR"

# Step 1: Download Ubuntu cloud image and verify its checksum before using it.
#
# SECURITY: a plain `wget` with no integrity check means a compromised mirror, a MITM'd
# download, or a partial/corrupted transfer would be silently accepted as a trusted guest
# boot image. This now downloads the upstream SHA256SUMS file alongside the image and
# refuses to proceed if the hash doesn't match. Note: Ubuntu's cloud-images.ubuntu.com no
# longer serves a "minimal/releases/<ver>/release" tree (it now 404s); this uses the
# actively maintained "releases/<ver>/release" server cloud image tree instead.
echo "[1/6] Downloading Ubuntu $UBUNTU_VERSION cloud image..."
CLOUD_IMG="ubuntu-$UBUNTU_VERSION-server-cloudimg-arm64.img"
CLOUD_IMG_BASE_URL="https://cloud-images.ubuntu.com/releases/$UBUNTU_VERSION/release"

if [ ! -f "$CLOUD_IMG" ]; then
    curl -fsSL -o "$CLOUD_IMG" "$CLOUD_IMG_BASE_URL/$CLOUD_IMG"
fi

echo "[1/6] Verifying checksum against upstream SHA256SUMS..."
curl -fsSL -o SHA256SUMS "$CLOUD_IMG_BASE_URL/SHA256SUMS"
EXPECTED_SHA256=$(grep " \*${CLOUD_IMG}\$" SHA256SUMS | awk '{print $1}')
if [ -z "$EXPECTED_SHA256" ]; then
    echo "ERROR: could not find a checksum for $CLOUD_IMG in upstream SHA256SUMS" >&2
    exit 1
fi

ACTUAL_SHA256=$(shasum -a 256 "$CLOUD_IMG" | awk '{print $1}')
if [ "$ACTUAL_SHA256" != "$EXPECTED_SHA256" ]; then
    echo "ERROR: checksum mismatch for $CLOUD_IMG" >&2
    echo "  expected: $EXPECTED_SHA256" >&2
    echo "  actual:   $ACTUAL_SHA256" >&2
    rm -f "$CLOUD_IMG"
    exit 1
fi
echo "  OK: $CLOUD_IMG checksum verified ($ACTUAL_SHA256)"

# Step 2: Prepare rootfs (extract cloud-init image)
echo "[2/6] Preparing rootfs..."
# In production, would unpack cloud image; for CI, we create config structure
mkdir -p etc/systemd/{system,system-generators}
mkdir -p etc/systemd/system-generators
mkdir -p etc/kernel/cmdline.d
mkdir -p usr/local/bin

# Step 3: Kernel cmdline optimization
echo "[3/6] Setting kernel cmdline for minimal boot..."
cat > etc/kernel/cmdline.d/99-tinybridge.conf <<'EOF'
root=/dev/vda1 rw console=hvc0 quiet systemd.unified_cgroup_hierarchy=1 systemd.show_status=no systemd.log_level=notice
EOF

# Step 4: Tier 1 - Mask unnecessary services
echo "[4/6] Configuring Tier 1 (SSH only, 1.5s)..."
mkdir -p etc/systemd/system

# Create mask override directory for each service
for service in \
  getty@.service \
  console-getty.service \
  systemd-udevd.service \
  systemd-udevd-control.socket \
  systemd-udevd-kernel.socket \
  dbus.service \
  dbus.socket \
  systemd-random-seed.service \
  systemd-fsck-root.service \
  e2fsck-root.service; do

  mkdir -p "etc/systemd/system/${service}.d"
  cat > "etc/systemd/system/${service}.d/tier1-mask.conf" <<EOF
# Tier 1 Mask: Not needed for SSH boot
[Unit]
ConditionPathExists=!/.tinybridge-tier1-skip

[Service]
Type=oneshot
ExecStart=/bin/true
RemainAfterExit=yes
EOF
done

# Step 5: Tier 2 - Background services with Type=idle
echo "[5/6] Configuring Tier 2 (usable, 1.5-5s)..."
for service in \
  systemd-resolved.service \
  systemd-timesyncd.service \
  systemd-logind.service; do

  mkdir -p "etc/systemd/system/${service}.d"
  cat > "etc/systemd/system/${service}.d/tier2-defer.conf" <<EOF
# Tier 2: Load in background after SSH ready
[Unit]
After=sshd.service
ConditionPathExists=!/.tinybridge-no-tier2

[Service]
Type=idle
StartLimitIntervalSec=60
StartLimitBurst=5
EOF
done

# Socket activation for journald
mkdir -p etc/systemd/system/systemd-journald.socket.d
cat > etc/systemd/system/systemd-journald.socket.d/tier2-eager.conf <<'EOF'
# Tier 2: Socket activation - start on first log
[Socket]
ListenStream=/run/systemd/journal/stdout
ListenStream=/run/systemd/journal/dev-log
ListenDatagram=/dev/log
EOF

# Step 6: Tier 3 - Eventual system services
echo "[6/6] Configuring Tier 3 (complete system, 5-120s)..."

# Create placeholder services for Tier 3 (actual services depend on rootfs)
mkdir -p etc/systemd/system
cat > etc/systemd/system/tier3-loader.service <<'EOF'
[Unit]
Description=Tier 3 Service Loader (eventual startup)
After=sshd.service
After=systemd-resolved.service
ConditionPathExists=!/.tinybridge-no-tier3

[Service]
Type=oneshot
ExecStart=/usr/local/bin/tier3-loader.sh
RemainAfterExit=yes
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Create Tier 3 loader script (placeholder)
mkdir -p usr/local/bin
cat > usr/local/bin/tier3-loader.sh <<'EOF'
#!/bin/bash
# Tier 3 Service Loader
# Starts non-critical services after Tier 1 & 2 complete

echo "Tier 3: Loading deferred services..."

# In actual rootfs, this would start:
# - APT package cache update
# - Monitoring agents
# - Development tools
# - System optimizations

# Placeholder: just log completion
echo "Tier 3: Complete system ready"
EOF
chmod +x usr/local/bin/tier3-loader.sh

# Create systemd generator for automatic tier assignment
echo "Creating systemd generator for multi-tier boot..."
mkdir -p etc/systemd/system-generators
cat > etc/systemd/system-generators/99-tinybridge-tiers <<'EOF'
#!/bin/bash
# systemd generator: Apply multi-tier lazy loading strategy
#
# This generator modifies service startup order to implement:
# Tier 1 (1.5s): SSH ready
# Tier 2 (1.5-5s): Background load
# Tier 3 (5-120s): Eventual startup
# Tier 4 (on-demand): Masked

NORMAL_DIR="${1:-.}"
EARLY_DIR="${2:-.}"

# Tier 2: Type=idle services (load while user works)
for service in \
  systemd-resolved.service \
  systemd-timesyncd.service \
  systemd-logind.service \
  rsyslog.service; do

  mkdir -p "$NORMAL_DIR/${service}.d"
  cat > "$NORMAL_DIR/${service}.d/tier2.conf" <<'SERVICECFG'
[Unit]
After=sshd.service
ConditionPathExists=!/.tinybridge-skip-tier2

[Service]
Type=idle
SERVICECFG
done

# Tier 3: Services that load after Tier 2 complete
for service in \
  apt-daily.service \
  apt-daily-upgrade.service; do

  if [ -f "$NORMAL_DIR/${service}" ]; then
    mkdir -p "$NORMAL_DIR/${service}.d"
    cat > "$NORMAL_DIR/${service}.d/tier3.conf" <<'SERVICECFG'
[Unit]
After=sshd.service
After=systemd-resolved.service
ConditionPathExists=!/.tinybridge-skip-tier3

[Service]
Type=idle
SERVICECFG
  fi
done

# Tier 4: On-demand only (masked by default)
for service in \
  bluetooth.service \
  cups.socket \
  avahi-daemon.service; do

  if [ -f "$NORMAL_DIR/${service}" ]; then
    mkdir -p "$NORMAL_DIR/${service}.d"
    cat > "$NORMAL_DIR/${service}.d/tier4-mask.conf" <<'SERVICECFG'
[Unit]
# Tier 4: Only start on explicit user request
ConditionPathExists=/.tinybridge-tier4-enable
SERVICECFG
  fi
done

exit 0
EOF
chmod +x etc/systemd/system-generators/99-tinybridge-tiers

echo "✓ Multi-tier rootfs configuration complete"
echo ""
echo "Configuration summary:"
echo "  Tier 1 (SSH ready): 1.5s"
echo "  Tier 2 (background): 1.5-5s"
echo "  Tier 3 (eventual): 5-120s"
echo "  Tier 4 (on-demand): masked"
echo ""
echo "Files created:"
echo "  - etc/kernel/cmdline.d/99-tinybridge.conf (minimal kernel cmdline)"
echo "  - etc/systemd/system/*/tier*.conf (per-service tier config)"
echo "  - etc/systemd/system-generators/99-tinybridge-tiers (auto-tier assignment)"
echo "  - usr/local/bin/tier3-loader.sh (Tier 3 loader)"
echo ""
echo "To apply to rootfs, copy these files to your Ubuntu 24.04 minimal image"
echo "then rebuild VM."
