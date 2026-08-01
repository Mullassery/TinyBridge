# TinyBridge

Linux development on macOS. Open source, CLI-first, zero configuration.

[![Crates.io](https://img.shields.io/crates/v/tinybridge)](https://crates.io/crates/tinybridge)
[![Tests](https://img.shields.io/github/actions/workflow/status/Mullassery/TinyBridge/tests.yml?label=tests)](https://github.com/Mullassery/TinyBridge/actions)

Lightweight, fast Linux VM integration for Mac developers. Develop for Linux without heavy virtualization.

## Quick Start

```bash
# Install (Homebrew tap, all architectures)
brew tap Mullassery/tinybridge
brew install tinybridged

# Start Linux environment
tinybridge start

# Your shell is now in Linux
$ uname -a
Linux tinybridge 6.x.x
```

## Key Features

- Instant Linux environment on macOS
- Zero configuration required
- Fast boot (1.5s to interactive shell)
- Full Linux compatibility
- SSH access built-in
- Minimal resource usage

## Installation

### Homebrew (Recommended)

**Supports macOS on Intel and Apple Silicon (M1/M2/M3)**

```bash
brew tap Mullassery/tinybridge
brew install tinybridged
```

This installs two components:
- **`tinybridge`** - CLI tool for managing Linux environments
- **`tinybridged`** - Background daemon with auto-start via LaunchAgent

### Minimal CLI Only

If you only need the command-line tool without the daemon:

```bash
brew tap Mullassery/tinybridge
brew install tinybridge
```

### What Gets Installed

| Component | Purpose | Auto-Start |
|-----------|---------|-----------|
| `tinybridge` | Command-line interface | Manual |
| `tinybridged` | Background daemon | Yes (via LaunchAgent) |

**Note:** `tinybridged` automatically installs `tinybridge` as a dependency.

## Use Cases

- Develop Linux-only applications on Mac
- Test Docker containers locally
- Cross-platform development
- CI/CD pipeline testing
- Kernel module development

## Documentation

- [Getting Started](docs/getting-started.md)
- [Configuration](docs/config.md)
- [Advanced Usage](docs/advanced.md)

## License

MIT License - See LICENSE
