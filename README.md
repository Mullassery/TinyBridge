# TinyBridge

Linux development on macOS. Open source, CLI-first, zero configuration.

[![Crates.io](https://img.shields.io/crates/v/tinybridge)](https://crates.io/crates/tinybridge)
[![Tests](https://img.shields.io/github/actions/workflow/status/Mullassery/TinyBridge/tests.yml?label=tests)](https://github.com/Mullassery/TinyBridge/actions)

Lightweight, fast Linux VM integration for Mac developers. Develop for Linux without heavy virtualization.

## Quick Start

```bash
# Install
brew install mullassery/tinybridge/tinybridge

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

```bash
brew install mullassery/tinybridge/tinybridge
```

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
