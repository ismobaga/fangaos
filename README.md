# FangaOS

A modern x86_64 operating system kernel written in Rust.

[![Test Suite](https://github.com/ismobaga/fangaos/actions/workflows/test.yml/badge.svg)](https://github.com/ismobaga/fangaos/actions/workflows/test.yml)
[![QEMU Tests](https://github.com/ismobaga/fangaos/actions/workflows/qemu-test.yml/badge.svg)](https://github.com/ismobaga/fangaos/actions/workflows/qemu-test.yml)
[![Coverage](https://github.com/ismobaga/fangaos/actions/workflows/coverage.yml/badge.svg)](https://github.com/ismobaga/fangaos/actions/workflows/coverage.yml)

## Features

- **Memory Management**
  - Physical memory allocator (bitmap-based)
  - Virtual memory management with paging
  - Dynamic heap allocation
  - Memory statistics and debugging

- **Interrupt Handling**
  - IDT (Interrupt Descriptor Table) setup
  - Exception handlers
  - Hardware interrupt support (PIC and APIC)
  - Keyboard input handling

- **System Calls**
  - SYSCALL/SYSRET instruction support
  - Basic syscalls (read, write, exit)
  - Syscall argument validation
  - Error handling and return codes

- **I/O Subsystem**
  - Framebuffer console
  - Serial port communication
  - Keyboard input
  - Custom font rendering

- **Architecture Support**
  - x86_64 primary architecture
  - UEFI boot support via Limine bootloader

## Building

### Prerequisites

```bash
# Install Rust nightly
rustup toolchain install nightly
rustup component add rust-src

# Install build dependencies
sudo apt-get install -y \
  xorriso \
  mtools \
  qemu-system-x86 \
  ovmf \
  lld
```

### Build the Kernel

```bash
# Build ISO image
make all

# Build kernel binary only
make kernel

# Build HDD image
make all-hdd
```

## Running

### QEMU

```bash
# Run with UEFI firmware
make run

# Run with HDD image
make run-hdd

# Run with BIOS (if supported)
make run-bios
```

### Custom QEMU Flags

```bash
# Run with custom memory size
make run QEMUFLAGS="-m 4G"

# Run with serial output
make run QEMUFLAGS="-m 2G -serial stdio"
```

## Testing

FangaOS has a comprehensive testing infrastructure. See [TESTING.md](TESTING.md) for detailed information.

### Quick Start

```bash
# Run all unit and integration tests
make test

# Run QEMU boot test
make test-qemu

# Generate coverage report
make coverage
```

### Test Organization

- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test component interactions
- **QEMU Tests**: Test kernel boot and initialization

## Project Structure

```
fangaos/
├── kernel/                    # Kernel source code
│   ├── crates/
│   │   ├── fanga-kernel/     # Main kernel crate
│   │   │   ├── src/
│   │   │   │   ├── memory/   # Memory management
│   │   │   │   ├── io/       # I/O subsystems
│   │   │   │   ├── main.rs   # Kernel entry point
│   │   │   │   └── lib.rs    # Library interface for tests
│   │   │   └── tests/        # Integration tests
│   │   └── fanga-arch-x86_64/# x86_64 architecture support
│   │       └── src/
│   │           ├── interrupts/# Interrupt handling
│   │           ├── gdt.rs     # Global Descriptor Table
│   │           ├── serial.rs  # Serial communication
│   │           └── keyboard.rs# Keyboard driver
│   └── Cargo.toml            # Workspace configuration
├── scripts/                   # Build and test scripts
│   └── qemu-test.sh          # QEMU automated testing
├── .github/
│   └── workflows/            # CI/CD pipelines
├── docs/                     # Documentation
├── Makefile                  # Build system
└── TESTING.md               # Testing guide
```

## Documentation

- [Testing Guide](TESTING.md) - How to run and write tests
- [Memory Management](docs/MEMORY_MANAGEMENT.md) - Memory subsystem design
- [Interrupt Handling](docs/INTERRUPT_HANDLING.md) - Interrupt architecture
- [Input/Output](docs/INPUT_OUTPUT.md) - I/O subsystem design
- [System Calls](docs/SYSTEM_CALLS.md) - System call interface
- [Implementation Summary](docs/IMPLEMENTATION_SUMMARY.md) - Overall design

## Development

### Code Style

```bash
# Format code
cd kernel && cargo fmt --all

# Run linter
cd kernel/crates/fanga-kernel && cargo clippy
```

### Debugging

```bash
# Build with debug symbols
make kernel

# Run with GDB
qemu-system-x86_64 \
  -s -S \
  -M q35 \
  -m 2G \
  -drive if=pflash,unit=0,format=raw,file=ovmf/ovmf-code-x86_64.fd,readonly=on \
  -drive if=pflash,unit=1,format=raw,file=ovmf/ovmf-vars-x86_64.fd \
  -cdrom fangaos-x86_64.iso &

# In another terminal
gdb kernel/target/x86_64-fanga-kernel/release/fanga-kernel
(gdb) target remote :1234
(gdb) continue
```

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **Write Tests**: All new features should include unit and/or integration tests
2. **Format Code**: Run `cargo fmt` before committing
3. **Check Lints**: Run `cargo clippy` and address warnings
4. **Test Thoroughly**: Run `make test` and `make test-qemu`
5. **Document Changes**: Update relevant documentation

## Continuous Integration

FangaOS uses GitHub Actions for CI/CD:

- **Test Suite**: Runs unit tests, integration tests, and builds the kernel
- **QEMU Tests**: Boots the kernel in QEMU and verifies initialization
- **Coverage**: Generates test coverage reports

All checks must pass before merging pull requests.

## License

[Add license information here]

## Architecture

FangaOS is designed with modularity in mind:

- **No Standard Library**: Uses `#![no_std]` for bare-metal execution
- **Memory Safety**: Leverages Rust's ownership system for safety
- **Modular Design**: Separate crates for architecture-specific code
- **Testability**: Library interface allows comprehensive testing

## Status

FangaOS is currently in active development. Core features implemented:

- ✅ Memory management (PMM, VMM, heap)
- ✅ Interrupt handling (exceptions and IRQs)
- ✅ Basic I/O (framebuffer, serial, keyboard)
- ✅ System calls (syscall/sysret interface)
- ✅ Comprehensive test suite
- ✅ CI/CD pipeline
- 🚧 Process management (in progress)
- 🚧 File system (planned)
- 🚧 Networking (planned)

## Resources

- [OSDev Wiki](https://wiki.osdev.org/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Writing an OS in Rust](https://os.phil-opp.com/)
- [Intel® 64 and IA-32 Architectures Software Developer Manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
