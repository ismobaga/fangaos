# FangaOS

A modern, Rust-based operating system kernel with multi-architecture support.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-not_yet_specified-blue)]()
[![Rust](https://img.shields.io/badge/rust-nightly-orange)]()

## 🚀 Overview

FangaOS is an experimental operating system kernel written in Rust, designed to be:

- **Safe**: Leveraging Rust's memory safety guarantees
- **Modern**: No legacy baggage, clean architecture from the ground up
- **Multi-architecture**: Designed to support x86_64, ARM64, RISC-V, and LoongArch64
- **Educational**: Well-documented and approachable for learning OS development

## ✨ Current Features

- ✅ Boots on x86_64 using Limine bootloader
- ✅ Serial port output for debugging (COM1)
- ✅ Basic interrupt handling (IDT, PIC)
- ✅ Framebuffer graphics output
- ✅ Memory map access from bootloader
- ✅ Higher Half Direct Map (HHDM) support
- ✅ Panic handler with serial output

## 📋 Requirements

### Build Dependencies

- **Rust Toolchain**: Nightly (see `rust-toolchain.toml`)
- **Make**: GNU Make for build automation
- **QEMU**: For testing and running the OS
  - `qemu-system-x86_64` (primary target)
  - `qemu-system-aarch64`, `qemu-system-riscv64`, `qemu-system-loongarch64` (optional)
- **xorriso**: For creating ISO images
- **curl**: For downloading OVMF firmware
- **mtools**: For HDD image creation
- **sgdisk**: For disk partitioning

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y build-essential curl qemu-system-x86 xorriso mtools gdisk
```

### Arch Linux

```bash
sudo pacman -S base-devel curl qemu-full xorriso mtools gptfdisk
```

### macOS

```bash
brew install qemu xorriso mtools
brew install --cask gptfdisk
```

## 🔨 Building

### Quick Start

Build and run the OS in QEMU:

```bash
make run
```

This will:
1. Build the kernel
2. Create a bootable ISO image
3. Launch QEMU with the OS

### Build Options

Build ISO image only:
```bash
make
```

Build HDD image:
```bash
make all-hdd
```

Run from HDD image:
```bash
make run-hdd
```

Run with BIOS (legacy boot):
```bash
make run-bios
```

### Architecture Selection

Build for different architectures:

```bash
# x86_64 (default)
make KARCH=x86_64 run

# ARM64
make KARCH=aarch64 run

# RISC-V 64-bit
make KARCH=riscv64 run

# LoongArch64
make KARCH=loongarch64 run
```

### QEMU Options

Customize QEMU parameters:

```bash
# Increase memory to 4GB
make run QEMUFLAGS="-m 4G"

# Enable KVM acceleration
make run QEMUFLAGS="-m 2G -enable-kvm"

# Connect GDB for debugging
make run QEMUFLAGS="-m 2G -s -S"
```

## 🧪 Testing

Currently, the kernel outputs debug information via serial port (COM1), which is visible in the terminal when running with `make run`.

Expected output:
```
[Fanga] entered _start
[Fanga] bootloader: Limine
[Fanga] bootloader version: ...
[Fanga] HHDM offset: 0x...
[Fanga] mem total:  ... KiB
[Fanga] mem usable: ... KiB
[Fanga] framebuffer filled ✅
```

## 📁 Project Structure

```
fangaos/
├── kernel/                    # Kernel source code
│   ├── crates/
│   │   ├── fanga-kernel/      # Main kernel entry point
│   │   └── fanga-arch-x86_64/ # x86_64 architecture-specific code
│   ├── linker.ld              # Linker script
│   ├── x86_64-fanga-kernel.json # Target specification
│   └── Cargo.toml             # Workspace configuration
├── limine.conf                # Bootloader configuration
├── Makefile                   # Build system
├── ROADMAP.md                 # Development roadmap
└── README.md                  # This file
```

## 🗺️ Roadmap

FangaOS is in early development. See [ROADMAP.md](ROADMAP.md) for detailed plans including:

- **Phase 1**: Memory management, keyboard input, basic I/O
- **Phase 2**: Multitasking, system calls, file systems
- **Phase 3**: Networking, GUI, advanced features

Key priorities:
1. Physical and virtual memory management
2. Keyboard driver and console input
3. Process/task management and scheduling
4. File system support
5. User space and system calls

## 🤝 Contributing

Contributions are welcome! Whether you're:

- Fixing bugs
- Adding features
- Improving documentation
- Testing on different hardware

Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

For major changes, please open an issue first to discuss what you'd like to change.

### Development Workflow

1. **Clone the repository**
   ```bash
   git clone https://github.com/ismobaga/fangaos.git
   cd fangaos
   ```

2. **Make changes**
   - Edit kernel code in `kernel/crates/`
   - Follow Rust conventions and style

3. **Test locally**
   ```bash
   make run
   ```

4. **Check code quality**
   ```bash
   cd kernel && cargo clippy
   cd kernel && cargo fmt
   ```

5. **Submit PR**
   - Write clear commit messages
   - Reference related issues
   - Include testing notes

## 📚 Resources

### Learning OS Development
- [OSDev Wiki](https://wiki.osdev.org/) - Comprehensive OS development resource
- [Philipp Oppermann's Blog](https://os.phil-opp.com/) - Writing an OS in Rust
- [The Rust Book](https://doc.rust-lang.org/book/) - Learning Rust
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - Unsafe Rust and FFI

### Related Projects
- [Redox OS](https://www.redox-os.org/) - Unix-like OS written in Rust
- [Limine](https://github.com/limine-bootloader/limine) - Modern bootloader
- [Theseus OS](https://www.theseus-os.com/) - Rust OS research project

### Rust OSDev Community
- [rust-osdev](https://github.com/rust-osdev) - Rust OS development organization
- [r/osdev](https://www.reddit.com/r/osdev/) - OS development subreddit

## 📜 License

Not yet specified. License will be chosen and added soon.

## 🙏 Acknowledgments

- [Limine](https://github.com/limine-bootloader/limine) - For the excellent bootloader
- [rust-osdev](https://github.com/rust-osdev) - For Rust OS development tools
- The Rust community - For creating an amazing language for systems programming

## 📞 Contact

- **Project**: [github.com/ismobaga/fangaos](https://github.com/ismobaga/fangaos)
- **Issues**: [github.com/ismobaga/fangaos/issues](https://github.com/ismobaga/fangaos/issues)

---

**Status**: Early Development (v0.1.0)  
**Last Updated**: December 2024
