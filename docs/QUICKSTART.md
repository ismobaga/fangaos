# Quick Start Guide

Get FangaOS up and running in 5 minutes!

## Prerequisites

You need:
- A Unix-like system (Linux or macOS recommended)
- Internet connection (for downloads)
- ~2GB of disk space

## Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default nightly
```

## Step 2: Install Build Tools

### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y build-essential qemu-system-x86 xorriso mtools gdisk curl git
```

### Arch Linux
```bash
sudo pacman -S base-devel qemu-full xorriso mtools gptfdisk curl git
```

### macOS
```bash
brew install qemu xorriso mtools
brew install --cask gptfdisk
```

## Step 3: Clone and Build

```bash
# Clone the repository
git clone https://github.com/ismobaga/fangaos.git
cd fangaos

# Build and run!
make run
```

## Expected Output

You should see:
1. Build process (~1-2 minutes on first run)
2. QEMU window opens
3. Screen fills with color
4. Terminal shows:
```
[Fanga] entered _start
[Fanga] bootloader: Limine
[Fanga] bootloader version: ...
[Fanga] HHDM offset: 0x...
[Fanga] mem total:  2097152 KiB
[Fanga] mem usable: 2094080 KiB
[Fanga] framebuffer filled ✅
```

## Troubleshooting

### Build fails with "error: no default toolchain configured"
```bash
rustup default nightly
```

### Build fails with "can't find crate for std"
This is expected! FangaOS doesn't use std. Check for other errors.

### QEMU doesn't start
```bash
# Check QEMU is installed
which qemu-system-x86_64

# Try manual QEMU start
qemu-system-x86_64 --version
```

### "Permission denied" errors
```bash
# Check file permissions
ls -l

# Make sure you're not running as root
whoami  # Should NOT be root
```

### Build is very slow
First build is slow because it:
- Downloads dependencies
- Downloads OVMF firmware
- Clones Limine bootloader
- Compiles everything

Subsequent builds are much faster!

### Network issues (can't download dependencies)
```bash
# Check internet connection
ping -c 3 github.com

# Try manual download of Limine
cd /tmp
git clone https://github.com/limine-bootloader/limine.git --branch=v9.x-binary --depth=1
```

## Next Steps

### Explore the Code
```bash
# View kernel source
cat kernel/crates/fanga-kernel/src/main.rs

# View architecture code
ls kernel/crates/fanga-arch-x86_64/src/
```

### Make a Change
1. Edit `kernel/crates/fanga-kernel/src/main.rs`
2. Change the framebuffer color:
```rust
fb_fill_color(0xFF1E1E2E); // Change to 0xFFFF0000 for red
```
3. Rebuild and run:
```bash
make run
```

### Read the Documentation
- [README.md](../README.md) - Project overview
- [ROADMAP.md](../ROADMAP.md) - Development plan
- [CONTRIBUTING.md](../CONTRIBUTING.md) - How to contribute
- [docs/DEVELOPMENT.md](DEVELOPMENT.md) - Developer guide
- [docs/ARCHITECTURE.md](ARCHITECTURE.md) - Architecture details

## Common Commands

```bash
# Build only (no ISO)
cd kernel && cargo build --release \
    -Z build-std=core,compiler_builtins,alloc \
    --target x86_64-fanga-kernel.json

# Build ISO
make

# Run in QEMU
make run

# Run with more memory
make run QEMUFLAGS="-m 4G"

# Clean build artifacts
make clean

# Clean everything
make distclean

# Format code
cd kernel && cargo fmt

# Check code (linting)
cd kernel && cargo clippy
```

## Getting Help

- **Documentation**: Check the `docs/` directory
- **Issues**: Search [GitHub Issues](https://github.com/ismobaga/fangaos/issues)
- **Questions**: Open a [GitHub Discussion](https://github.com/ismobaga/fangaos/discussions)

## What's Next?

1. **Read the roadmap**: See what features are planned
2. **Pick a task**: Look for "good first issue" labels
3. **Make a contribution**: Follow CONTRIBUTING.md
4. **Share your experience**: Tell us what you think!

Happy hacking! 🚀

---

**Tip**: Bookmark this page for quick reference!
