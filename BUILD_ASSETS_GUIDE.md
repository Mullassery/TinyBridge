# Building Production Assets for TinyBridge

**Status:** Skeleton assets created. Production assets require external access.

---

## Current State

Assets directory exists with demonstration skeleton:

```bash
~/.tinybridge/assets/
├── vmlinux (9 B stub)
├── vmlinux.sha256
├── ubuntu-24.04-rootfs.tar.gz (66-file skeleton, 4 KB)
└── ubuntu-24.04-rootfs.tar.gz.sha256
```

The skeleton demonstrates the proper structure but is non-functional. A real system needs:
- **vmlinux:** 6-8 MB compiled Linux kernel (arm64)
- **ubuntu-24.04-rootfs.tar.gz:** 150-300 MB complete Ubuntu filesystem

---

## How to Get Production Kernel

### Option 1: Download Pre-Built Kernel (Easiest)

Cloud-Hypervisor releases include optimized arm64 kernels:

```bash
cd ~/.tinybridge/assets

# Download kernel
curl -L -o vmlinux \
  https://github.com/cloud-hypervisor/cloud-hypervisor/releases/download/v31.0/arch-arm64-vmlinux

# Verify
file vmlinux
sha256sum vmlinux
```

**Alternatives if Cloud-Hypervisor isn't available:**

```bash
# Firecracker kernel
curl -L -o vmlinux \
  https://github.com/firecracker-microvm/firecracker/releases/download/v1.3.0/vmlinux.bin

# Direct Ubuntu arm64 kernel
curl -L -o vmlinux \
  https://kernel.ubuntu.com/~kernel-ppa/mainline/v6.8/arm64/vmlinuz-6.8-generic

# Check what downloaded
file vmlinux
ls -lh vmlinux
```

Expected output:
```
vmlinux: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), ...
8.2M  vmlinux
```

### Option 2: Build Kernel Locally (If Pre-built Unavailable)

```bash
# Clone Linux kernel source
git clone --depth 1 --branch v6.8 \
  https://github.com/torvalds/linux.git /tmp/linux-build

cd /tmp/linux-build

# Minimal arm64 config for VZ Framework
make ARCH=arm64 defconfig
make ARCH=arm64 -j$(nproc)

# Kernel will be at:
# arch/arm64/boot/Image (uncompressed)
# arch/arm64/boot/Image.gz (compressed)

cp arch/arm64/boot/Image ~/.tinybridge/assets/vmlinux
```

Expected build time: 30-60 minutes on M5 MacBook

---

## How to Build Production Rootfs

### Option 1: Use Debootstrap (Linux or WSL2)

If you have access to a Linux machine or WSL2 on Windows:

```bash
# Install debootstrap if needed
sudo apt-get install debootstrap

# Build rootfs
mkdir -p /tmp/ubuntu-rootfs
sudo debootstrap \
    --arch arm64 \
    --variant minbase \
    --include=openssh-server,systemd,curl,wget,build-essential,python3 \
    noble \
    /tmp/ubuntu-rootfs \
    http://ports.ubuntu.com/ubuntu-ports/

# Configure
sudo bash -c 'echo "root:root" | chpasswd -R /tmp/ubuntu-rootfs'
sudo bash -c 'echo "PermitRootLogin yes" >> /tmp/ubuntu-rootfs/etc/ssh/sshd_config'

# Create fstab
sudo bash -c 'cat > /tmp/ubuntu-rootfs/etc/fstab << EOF
/dev/vda1 / ext4 defaults 0 1
proc /proc proc defaults 0 0
sysfs /sys sysfs defaults 0 0
tmpfs /dev/shm tmpfs defaults 0 0
devpts /dev/pts devpts defaults 0 0
EOF'

# Compress
cd /tmp
sudo tar -czf ubuntu-24.04-rootfs.tar.gz -C ubuntu-rootfs .
sudo chown $USER ubuntu-24.04-rootfs.tar.gz
cp ubuntu-24.04-rootfs.tar.gz ~/.tinybridge/assets/

# Verify
ls -lh ~/.tinybridge/assets/ubuntu-24.04-rootfs.tar.gz
```

Expected result:
```
150-300 MB  ubuntu-24.04-rootfs.tar.gz
```

### Option 2: Use Docker to Build Rootfs (On macOS)

```bash
# Create Dockerfile
cat > /tmp/Dockerfile << 'EOF'
FROM ubuntu:24.04

# Minimal setup
RUN apt-get update && apt-get install -y openssh-server systemd curl wget

# Enable SSH
RUN mkdir /run/sshd
RUN echo "PermitRootLogin yes" >> /etc/ssh/sshd_config
RUN echo "root:root" | chpasswd

# Create init script
RUN mkdir -p /init
CMD ["/bin/bash"]
EOF

# Build container
docker build -t ubuntu-24.04-builder -f /tmp/Dockerfile .

# Export filesystem
docker run --name ubuntu-exporter ubuntu-24.04-builder true
docker export ubuntu-exporter > /tmp/ubuntu-24.04-rootfs.tar
gzip /tmp/ubuntu-24.04-rootfs.tar

# Copy to assets
cp /tmp/ubuntu-24.04-rootfs.tar.gz ~/.tinybridge/assets/

# Cleanup
docker rm ubuntu-exporter
```

### Option 3: Download Pre-Built Rootfs

Some projects provide pre-built Ubuntu arm64 rootfs images:

```bash
# Ubuntu cloud images for arm64
curl -L -o /tmp/ubuntu.tar.xz \
  https://cloud-images.ubuntu.com/releases/noble/release/ubuntu-24.04-server-cloudimg-arm64.tar.xz

# Extract and recompress
cd /tmp
tar -xf ubuntu.tar.xz
tar -czf ubuntu-24.04-rootfs.tar.gz -C extracted .

cp ubuntu-24.04-rootfs.tar.gz ~/.tinybridge/assets/
```

---

## Verification & Checksums

After downloading/building, verify integrity:

```bash
cd ~/.tinybridge/assets

# Generate checksums
sha256sum vmlinux > vmlinux.sha256
sha256sum ubuntu-24.04-rootfs.tar.gz > ubuntu-24.04-rootfs.tar.gz.sha256

# Verify files
cat vmlinux.sha256
cat ubuntu-24.04-rootfs.tar.gz.sha256

# Test extraction
mkdir -p /tmp/test-rootfs
tar -tzf ubuntu-24.04-rootfs.tar.gz | head -20  # Should show /bin, /etc, /usr, etc.
```

Expected directory structure:
```
$ tar -tzf ubuntu-24.04-rootfs.tar.gz | head -20
./
./bin/
./boot/
./dev/
./etc/
./etc/apt/
./etc/init.d/
./etc/ssh/
./home/
./lib/
./media/
./mnt/
./opt/
./proc/
./root/
./run/
./sbin/
./srv/
./sys/
./tmp/
./usr/
./var/
```

---

## Testing Assets

Once you have real assets, test them:

```bash
# Test kernel
file ~/.tinybridge/assets/vmlinux
# Should show: ELF 64-bit LSB executable, ARM aarch64

# Test rootfs
tar -tzf ~/.tinybridge/assets/ubuntu-24.04-rootfs.tar.gz | wc -l
# Should show: 2000+ files (real rootfs), not 66 (skeleton)

# Verify checksums
cd ~/.tinybridge/assets
sha256sum -c vmlinux.sha256
sha256sum -c ubuntu-24.04-rootfs.tar.gz.sha256
```

---

## What the Daemon Will Do

Once real assets are in place, the daemon will:

```bash
# When you run: tinybridge up myenv

1. Read ~/.tinybridge/assets/vmlinux
2. Verify checksum against vmlinux.sha256
3. Load kernel into Apple VZ Framework
4. Extract ubuntu-24.04-rootfs.tar.gz to VM filesystem
5. Mount VirtioFS for file sharing
6. Boot Linux and start SSH daemon
7. Report "SSH ready" when port 22 responds
```

Expected timeline:
- Kernel load: 0.3s
- Rootfs extract: 2-3s
- VirtioFS mount: 0.5s
- Linux boot: 1-2s
- **Total to SSH: 4-6s (target: <2s requires further optimization)**

---

## Production vs. Skeleton

| Component | Skeleton | Production |
|-----------|----------|------------|
| vmlinux | 9 B stub | 6-8 MB real kernel |
| rootfs | 4 KB (66 files) | 150-300 MB (2000+ files) |
| SSH | Fake config | Real openssh-server |
| Packages | None | apt with ~100 base packages |
| Boot time | N/A | 4-6s target |
| Use case | Demo structure | Actual Linux development |

---

## Next Steps

### On Your M5 MacBook:

1. **Get kernel** (30 min)
   ```bash
   curl -L -o ~/.tinybridge/assets/vmlinux \
     https://github.com/cloud-hypervisor/cloud-hypervisor/releases/download/v31.0/arch-arm64-vmlinux
   ```

2. **Build rootfs** (1 hour)
   ```bash
   # Via debootstrap if on Linux/WSL2
   sudo debootstrap --arch arm64 noble ./rootfs http://ports.ubuntu.com/ubuntu-ports/
   tar -czf ~/.tinybridge/assets/ubuntu-24.04-rootfs.tar.gz -C rootfs .
   
   # OR via Docker
   docker export ubuntu:24.04 | gzip > ~/.tinybridge/assets/ubuntu-24.04-rootfs.tar.gz
   ```

3. **Create checksums** (1 min)
   ```bash
   cd ~/.tinybridge/assets
   sha256sum * > checksums.txt
   ```

4. **Test daemon** (5 min)
   ```bash
   tinybridge up test-env
   tinybridge shell test-env
   ```

5. **Run benchmarks** (See TESTING_REPORT.md)

---

## Troubleshooting

### "vmlinux is not a valid ELF file"
- Wrong architecture (need arm64, not x86_64)
- File corrupted during download
- Solution: Re-download or build from source

### "rootfs.tar.gz is empty or corrupted"
- Download failed
- Wrong compression format
- Solution: Verify with `tar -tzf` before using

### "Kernel loads but VM won't boot"
- Rootfs filesystem mismatch
- Missing critical system files (/etc/fstab, /sbin/init)
- Solution: Rebuild rootfs with proper debootstrap setup

### "SSH port doesn't respond"
- VM booted but SSH daemon didn't start
- SSH config missing or incorrect
- Solution: Check `/etc/ssh/sshd_config` in rootfs

---

## References

- **Cloud-Hypervisor:** https://github.com/cloud-hypervisor/cloud-hypervisor/releases
- **Ubuntu Ports (arm64):** http://ports.ubuntu.com/ubuntu-ports/
- **Debootstrap:** https://wiki.debian.org/Debootstrap
- **Linux kernel build:** https://www.kernel.org/doc/html/latest/kbuild/

---

**Summary:** Skeleton is built. Production assets require 1-2 hours of downloading/building on real hardware with internet access. Once in place, TinyBridge testing can proceed.
