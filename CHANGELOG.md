# Changelog

All notable changes to FangaOS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive development roadmap (ROADMAP.md)
- Project README with build instructions
- Contributing guidelines (CONTRIBUTING.md)
- This changelog file

### Changed
- Nothing yet

### Deprecated
- Nothing yet

### Removed
- Nothing yet

### Fixed
- Nothing yet

### Security
- Nothing yet

## [0.1.0] - 2024-12-13

### Added
- Initial x86_64 kernel implementation
- Limine bootloader integration
- Serial port (COM1) output for debugging
- Basic interrupt handling:
  - Interrupt Descriptor Table (IDT) setup
  - Programmable Interrupt Controller (PIC) initialization
  - Debug breakpoint handler (int3)
- Framebuffer access and initialization
- Memory map access from bootloader
- Higher Half Direct Map (HHDM) support
- Multi-architecture build system supporting:
  - x86_64 (primary, functional)
  - aarch64 (ARM64) - build infrastructure only
  - riscv64 (RISC-V 64-bit) - build infrastructure only
  - loongarch64 - build infrastructure only
- QEMU test runners for all architectures
- Kernel workspace with two crates:
  - `fanga-kernel`: Main kernel entry point
  - `fanga-arch-x86_64`: x86_64-specific code
- Panic handler with serial output
- Basic port I/O abstractions
- Linker script for kernel layout
- OVMF firmware download automation
- ISO and HDD image creation
- `.gitignore` for build artifacts

[Unreleased]: https://github.com/ismobaga/fangaos/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ismobaga/fangaos/releases/tag/v0.1.0
