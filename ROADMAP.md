# FangaOS Development Roadmap

This document outlines the development roadmap for FangaOS, a Rust-based operating system kernel. It includes planned features, improvements, and long-term goals organized by priority and development phases.

## Table of Contents
- [Project Status](#project-status)
- [Phase 1: Core Foundation (Short-Term)](#phase-1-core-foundation-short-term)
- [Phase 2: Essential OS Features (Medium-Term)](#phase-2-essential-os-features-medium-term)
- [Phase 3: Advanced Features (Long-Term)](#phase-3-advanced-features-long-term)
- [Infrastructure & Tooling](#infrastructure--tooling)
- [Documentation & Community](#documentation--community)

---

## Project Status

### Currently Implemented ✅
- [x] Basic x86_64 kernel booting with Limine bootloader
- [x] Serial port output (COM1) for debugging
- [x] Interrupt Descriptor Table (IDT) setup
- [x] Programmable Interrupt Controller (PIC) initialization
- [x] Framebuffer access and basic graphics output
- [x] Memory map access from bootloader
- [x] HHDM (Higher Half Direct Map) support
- [x] Multi-architecture build system (x86_64, aarch64, riscv64, loongarch64)
- [x] Panic handler with serial output

### What's Missing / Needs Improvement ⚠️
- No memory management (physical/virtual)
- No keyboard input handling
- No filesystem support
- No process/task management
- No system calls
- No user space
- Limited error handling
- No tests
- No documentation (README, API docs)
- No logging framework

---

## Phase 1: Core Foundation (Short-Term)
*Priority: High | Timeline: 1-3 months*

### 1.1 Documentation & Project Setup
- [ ] **README.md** - Project overview, build instructions, requirements
- [ ] **CONTRIBUTING.md** - Contribution guidelines, code style
- [ ] **LICENSE** - Choose and add appropriate license
- [ ] **CODE_OF_CONDUCT.md** - Community guidelines
- [ ] **Architecture documentation** - High-level design docs
- [ ] **Inline code documentation** - Rustdoc comments for all public APIs
- [ ] **Build documentation** - Detailed setup guide for developers

### 1.2 Memory Management
- [ ] **Physical memory allocator** (bitmap or buddy allocator)
- [ ] **Virtual memory manager** (page table management)
- [ ] **Heap allocator** (implement GlobalAlloc for dynamic allocation)
- [ ] **Memory regions management** (kernel, user, MMIO)
- [ ] **Page frame allocator** (track free/used pages)
- [ ] **Memory statistics and debugging tools**

### 1.3 Input/Output
- [ ] **Keyboard driver** (PS/2 and/or USB)
- [ ] **Basic text output** (VGA text mode or framebuffer console)
- [ ] **Console abstraction** (unify serial and screen output)
- [ ] **Logging framework** (with log levels: DEBUG, INFO, WARN, ERROR)
- [ ] **Better framebuffer management** (font rendering, scrolling)

### 1.4 Interrupt Handling Improvements
- [ ] **Complete IRQ handlers** (timer, keyboard, etc.)
- [ ] **Exception handlers** (page fault, general protection, etc.)
- [ ] **APIC support** (Advanced Programmable Interrupt Controller)
- [ ] **Interrupt registration system** (dynamic handler registration)

### 1.5 Testing Infrastructure
- [ ] **Unit tests** for core modules
- [ ] **Integration tests** for kernel components
- [ ] **CI/CD pipeline** (GitHub Actions for automated testing)
- [ ] **Test coverage reporting**
- [ ] **QEMU automated testing** scripts

---

## Phase 2: Essential OS Features (Medium-Term)
*Priority: Medium | Timeline: 3-9 months*

### 2.1 Multitasking & Scheduling
- [ ] **Process/Task structure** (TCB - Task Control Block)
- [ ] **Context switching** (save/restore CPU state)
- [ ] **Scheduler** (round-robin, then priority-based)
- [ ] **Thread support** (kernel threads)
- [ ] **Process creation and termination**
- [ ] **Inter-process communication (IPC)** basics

### 2.2 System Calls
- [ ] **System call interface** (syscall/sysret instructions)
- [ ] **Basic syscalls** (exit, write, read, fork, exec)
- [ ] **Syscall argument validation**
- [ ] **Error handling and return codes**

### 2.3 User Space
- [ ] **User space memory layout**
- [ ] **ELF loader** (load and execute user programs)
- [ ] **User mode switching** (ring 3)
- [ ] **User space libraries** (basic libc-like functionality)
- [ ] **Simple shell** (command interpreter)

### 2.4 File System
- [ ] **Virtual File System (VFS)** abstraction
- [ ] **RAM disk / initrd** support
- [ ] **Basic file operations** (open, close, read, write)
- [ ] **Directory operations** (list, create, remove)
- [ ] **Simple filesystem implementation** (e.g., FAT32 or custom)
- [ ] **Device file abstraction** (/dev)

### 2.5 Device Drivers
- [ ] **PCI enumeration and configuration**
- [ ] **ATA/AHCI disk driver** (storage)
- [ ] **Network card driver** (e1000 or virtio-net for QEMU)
- [ ] **RTC (Real-Time Clock) driver**
- [ ] **Driver framework** (unified interface for drivers)

### 2.6 Time Management
- [ ] **Timer implementation** (PIT, HPET, or APIC timer)
- [ ] **Clock abstraction** (system uptime, timestamps)
- [ ] **Sleep/wait functionality**
- [ ] **Scheduling quantum management**

---

## Phase 3: Advanced Features (Long-Term)
*Priority: Low | Timeline: 9+ months*

### 3.1 Advanced Memory Management
- [ ] **Copy-on-write (COW)** for process forking
- [ ] **Memory-mapped files** (mmap)
- [ ] **Shared memory** between processes
- [ ] **Demand paging** (lazy allocation)
- [ ] **Swapping** (disk-based virtual memory)
- [ ] **NUMA awareness** (for multi-socket systems)

### 3.2 Advanced Multitasking
- [ ] **User space threads** (pthread-like API)
- [ ] **Thread-local storage (TLS)**
- [ ] **Real-time scheduling** (deadline, FIFO)
- [ ] **CPU affinity** control
- [ ] **Load balancing** across CPU cores

### 3.3 Multiprocessor Support
- [ ] **SMP (Symmetric Multiprocessing)** initialization
- [ ] **Per-CPU data structures**
- [ ] **Spinlocks and mutexes** (SMP-safe)
- [ ] **CPU hotplug** support
- [ ] **Scheduler per-CPU runqueues**

### 3.4 Networking
- [ ] **Network stack** (TCP/IP)
- [ ] **Socket API** (BSD sockets)
- [ ] **Ethernet frame handling**
- [ ] **ARP, ICMP, UDP, TCP protocols**
- [ ] **DNS resolver**
- [ ] **Loopback interface**

### 3.5 Graphics & GUI
- [ ] **Framebuffer improvements** (multiple resolutions, modes)
- [ ] **2D graphics library** (drawing primitives)
- [ ] **Window manager** basics
- [ ] **GUI toolkit** (simple widgets)
- [ ] **GPU driver** (basic acceleration)

### 3.6 Security & Hardening
- [ ] **Address Space Layout Randomization (ASLR)**
- [ ] **Stack canaries** (buffer overflow protection)
- [ ] **Capabilities system** (fine-grained permissions)
- [ ] **Secure boot** support
- [ ] **Kernel module signing**
- [ ] **Audit logging**

### 3.7 POSIX Compatibility
- [ ] **POSIX-compliant system calls**
- [ ] **Signals** (SIGKILL, SIGTERM, etc.)
- [ ] **Pipes** (anonymous and named)
- [ ] **Porting standard utilities** (ls, cat, grep, etc.)
- [ ] **Shell scripting** support (bash-like)

### 3.8 Advanced File Systems
- [ ] **ext2/ext3/ext4** read/write support
- [ ] **File permissions and ownership** (UNIX-style)
- [ ] **Symbolic and hard links**
- [ ] **File locking** mechanisms
- [ ] **Journaling** for crash recovery
- [ ] **FUSE** (Filesystem in Userspace) support

### 3.9 Power Management
- [ ] **ACPI** (Advanced Configuration and Power Interface)
- [ ] **CPU frequency scaling**
- [ ] **Suspend/resume** (sleep states)
- [ ] **Shutdown/reboot** functionality
- [ ] **Power consumption monitoring**

---

## Infrastructure & Tooling

### Development Tools
- [ ] **Debugger support** (GDB integration)
- [ ] **Kernel profiling tools** (performance analysis)
- [ ] **Memory leak detection**
- [ ] **Code coverage tools**
- [ ] **Static analysis** (Clippy, Miri)
- [ ] **Benchmarking suite**

### Build System Improvements
- [ ] **Cross-compilation** for all architectures
- [ ] **Reproducible builds**
- [ ] **Build optimization** (faster compilation)
- [ ] **Dependency management** (vendoring, caching)
- [ ] **Custom toolchain management**

### Testing & QA
- [ ] **Automated regression tests**
- [ ] **Fuzzing** (kernel API fuzzing)
- [ ] **Stress tests** (memory, CPU, I/O)
- [ ] **Performance benchmarks**
- [ ] **Compatibility tests** (hardware matrix)

### CI/CD
- [ ] **GitHub Actions** workflows (build, test, lint)
- [ ] **Automated releases** (versioned ISOs)
- [ ] **Nightly builds**
- [ ] **Test result reporting**
- [ ] **Coverage badges** in README

---

## Documentation & Community

### Documentation Priorities
- [ ] **User guide** (how to use FangaOS)
- [ ] **Developer guide** (how to contribute)
- [ ] **API reference** (kernel APIs, auto-generated)
- [ ] **Architecture overview** (design decisions)
- [ ] **Porting guide** (new architectures)
- [ ] **Tutorial series** (blog posts, videos)

### Community Building
- [ ] **Project website** (landing page, documentation hub)
- [ ] **Discussion forum** (GitHub Discussions or Discord)
- [ ] **Regular releases** (semantic versioning)
- [ ] **Changelog** maintenance (CHANGELOG.md)
- [ ] **Roadmap tracking** (public project board)
- [ ] **Monthly/quarterly updates** (progress reports)

### Code Quality
- [ ] **Coding style guide** (Rust conventions)
- [ ] **PR templates** and review process
- [ ] **Issue templates** (bug reports, feature requests)
- [ ] **Architecture Decision Records (ADRs)**
- [ ] **Code review checklist**

---

## Multi-Architecture Support

### Priority Architectures
- [x] **x86_64** - Primary development target
- [ ] **aarch64** (ARM64) - Complete implementation
- [ ] **riscv64** - Complete implementation
- [ ] **loongarch64** - Complete implementation

### Architecture-Specific Features
- [ ] **Per-arch interrupt handling**
- [ ] **Per-arch memory management** (page tables)
- [ ] **Per-arch context switching**
- [ ] **Per-arch syscall interface**
- [ ] **Architecture abstraction layer** (unified API)

---

## Ideas & Suggestions

### Innovative Features
- [ ] **Microkernel design** (move drivers to user space)
- [ ] **Capability-based security model**
- [ ] **WebAssembly runtime** (run WASM in kernel/userspace)
- [ ] **Container support** (lightweight virtualization)
- [ ] **Live kernel patching** (update without reboot)
- [ ] **Time-travel debugging** (record/replay)

### Unique Selling Points
- [ ] **Rust safety** (memory safety, thread safety)
- [ ] **Modern architecture** (no legacy baggage)
- [ ] **Developer-friendly** (good docs, easy to contribute)
- [ ] **Educational** (well-documented for learning)
- [ ] **Minimalist** (clean, understandable codebase)

### Optional Experiments
- [ ] **Formal verification** of critical components
- [ ] **Unikernel mode** (single-application OS)
- [ ] **Distributed OS features** (cluster support)
- [ ] **Machine learning acceleration** (ML syscalls)
- [ ] **Blockchain integration** (trustless operations)

---

## Getting Started

### For New Contributors
1. Start with **Phase 1.1** (Documentation) - help improve docs
2. Pick **good first issues** from Phase 1 (marked accordingly)
3. Work on **isolated components** (drivers, small features)
4. Read **CONTRIBUTING.md** (once created) for guidelines
5. Join **community discussions** for questions

### Priority Order
1. **Documentation** (README, guides) - foundational
2. **Memory management** - critical for everything else
3. **Testing** - ensures quality and prevents regressions
4. **Multitasking** - enables real OS functionality
5. **File system** - makes OS practical
6. Everything else builds on these foundations

---

## Version Milestones

### v0.1.0 - "Hello World" (Current)
- Basic boot, serial output, framebuffer

### v0.2.0 - "Foundation"
- Memory management, keyboard input, logging

### v0.3.0 - "Interactive"
- Basic shell, file system, system calls

### v0.4.0 - "Multitasking"
- Process management, scheduler, IPC

### v0.5.0 - "Usable"
- User space programs, device drivers, networking basics

### v1.0.0 - "Stable"
- Full POSIX compatibility, mature driver support, stable API

---

## Contributing

This roadmap is a living document. If you have suggestions, ideas, or want to contribute to any of these features, please:

1. Open an issue to discuss major features
2. Submit PRs for documentation improvements
3. Comment on this roadmap with your ideas
4. Join community discussions

**Last Updated:** December 2024  
**Maintainer:** FangaOS Team
