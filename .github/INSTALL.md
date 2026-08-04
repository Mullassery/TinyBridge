# Installation Guide

## Quick Install

```bash
pip install tinybridge
```

## Requirements

- Python 3.10+
- macOS 10.13+, Ubuntu 20.04+, Windows 10+

## Installation

### Standard Install (Recommended)
Works for most users with prebuilt wheels:
```bash
pip install tinybridge
```

### From Source (If Needed)
For ARM or custom builds:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Then install
pip install tinybridge
```

## Troubleshooting

### "No wheels available for your platform"
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
pip install --force-reinstall tinybridge
```

### Python version issues
Ensure Python 3.10+:
```bash
python --version
```

### Missing dependencies (Linux)
```bash
sudo apt-get install python3-dev build-essential
```

## Next Steps

After installation:
1. See [README.md](../README.md) for quick start
2. Check [examples/](../examples/) for usage examples
3. Read [docs/](../docs/) for full documentation

For more help, see [full installation guide](INSTALL.md).
