# FangaOS Development Roadmap

This document outlines the development roadmap for FangaOS, a modern x86_64 operating system kernel written in Rust.

## Current Status (v0.1.0)

### ✅ Completed Features

#### Core Systems
- **Memory Management**
  - Physical memory manager (bitmap-based PMM)
  - Virtual memory manager with 4-level paging
  - Dynamic heap allocation with linked-list allocator
  - Memory statistics and debugging utilities
  - Memory region tracking

- **Interrupt Handling**
  - IDT (Interrupt Descriptor Table) configuration
  - Exception handlers (divide by zero, page fault, etc.)
  - Hardware interrupt support (PIC and APIC)
  - IRQ handling infrastructure

- **System Calls**
  - SYSCALL/SYSRET instruction support
  - Basic syscalls: read, write, exit
  - Syscall argument validation
  - Error handling and return codes

- **I/O Subsystem**
  - Framebuffer console with custom font rendering
  - Serial port communication (COM1)
  - PS/2 keyboard driver with scancode translation
  - Console abstraction (unified serial + screen output)
  - Logging framework (DEBUG, INFO, WARN, ERROR levels)

- **Architecture Support**
  - x86_64 primary architecture
  - UEFI boot via Limine bootloader
  - GDT (Global Descriptor Table) setup
  - Basic CPU features detection

- **Infrastructure**
  - Comprehensive test suite (unit, integration, QEMU tests)
  - CI/CD pipeline with GitHub Actions
  - Code coverage reporting
  - Documentation system

---

## Short-Term Goals (v0.2.0 - Next 3-6 months)

### Process Management
- [ ] **Process Control Block (PCB)**
  - Implement full process state tracking
  - Process creation and termination
  - Process scheduling with priority levels
  
- [ ] **Context Switching**
  - Save/restore CPU state
  - FPU/SSE state management
  - Switch between user and kernel mode

- [ ] **Scheduler**
  - Round-robin scheduling algorithm
  - Priority-based scheduling
  - CPU affinity support
  - Sleep/wakeup mechanisms

- [ ] **Multitasking**
  - Preemptive multitasking
  - Timer-based task switching
  - Yield system call
  - Fork/exec implementation (basic)

### User Space
- [ ] **User Mode Support**
  - Switch to ring 3
  - User space memory isolation
  - Copy to/from user space utilities
  
- [ ] **ELF Loader**
  - Parse ELF64 format
  - Load executable sections
  - Set up user space stack and heap
  - Program argument passing

- [ ] **Basic Shell**
  - Command line interface
  - Built-in commands (ls, cat, echo, help)
  - Command history
  - Tab completion

### File System (Initial)
- [ ] **VFS (Virtual File System)**
  - VFS layer abstraction
  - File, directory, inode abstractions
  - Mount point management
  
- [ ] **InitramFS**
  - Read-only initial RAM filesystem
  - Bundle files with kernel
  - Basic file operations (open, read, close)

### Memory Improvements
- [ ] **Slab Allocator**
  - Object caching for frequently allocated types
  - Reduce heap fragmentation
  - Improve allocation performance

- [ ] **Copy-on-Write (COW)**
  - Implement COW for fork()
  - Reduce memory usage
  - Share read-only pages

---

## Medium-Term Goals (v0.3.0 - 6-12 months)

### Advanced Process Management
- [ ] **Threading**
  - Kernel threads
  - User threads (1:1 model)
  - Thread local storage (TLS)
  - Mutex, semaphore, condition variables

- [ ] **IPC (Inter-Process Communication)**
  - Pipes (anonymous and named)
  - Message queues
  - Shared memory
  - Signals

- [ ] **Advanced Scheduling**
  - Multi-core support
  - Load balancing across CPUs
  - Real-time scheduling classes
  - Priority inheritance

### File System
- [ ] **FAT32 Support**
  - Read FAT32 partitions
  - Write support
  - Directory operations
  - Long filename support

- [ ] **Block Device Layer**
  - Generic block device abstraction
  - Buffer cache
  - I/O scheduling
  - Device mapper

- [ ] **Storage Drivers**
  - ATA/IDE driver
  - AHCI/SATA driver
  - NVMe driver (basic)
  - Partition table parsing (MBR, GPT)

### Device Drivers
- [ ] **USB Support**
  - USB host controller drivers (UHCI, EHCI, xHCI)
  - USB device enumeration
  - USB keyboard and mouse
  - USB mass storage

- [ ] **Network Stack (Basic)**
  - Ethernet driver (e1000, virtio-net)
  - ARP protocol
  - IP protocol (IPv4)
  - ICMP (ping)
  - UDP protocol

- [ ] **Graphics**
  - VESA/GOP framebuffer improvements
  - Double buffering
  - Hardware acceleration (basic)
  - Multiple display support

### System Enhancements
- [ ] **Virtual Memory Improvements**
  - Demand paging
  - Page cache
  - Memory-mapped files (mmap)
  - Swap support (basic)

- [ ] **Security Features**
  - Address Space Layout Randomization (ASLR)
  - Kernel Address Space Layout Randomization (KASLR)
  - Stack canaries
  - NX bit enforcement

---

## Long-Term Goals (v1.0.0+ - 12+ months)

### Advanced Networking
- [ ] **TCP/IP Stack**
  - TCP protocol implementation
  - Socket API
  - DNS resolver
  - DHCP client

- [ ] **Network Applications**
  - HTTP client
  - SSH client
  - Basic web server
  - Network utilities (netstat, ifconfig)

### Advanced File Systems
- [ ] **ext2/ext3/ext4 Support**
  - Read and write support
  - Journaling support (ext3/ext4)
  - Extended attributes
  - Access control lists (ACLs)

- [ ] **Modern File System**
  - Consider implementing or porting:
    - Btrfs (copy-on-write, snapshots)
    - ZFS (data integrity, RAID)
    - Or design custom FS optimized for FangaOS

### Desktop Environment
- [ ] **Window System**
  - Display server
  - Window management
  - Event system
  - Compositing

- [ ] **GUI Framework**
  - Widget toolkit
  - Graphics primitives
  - Font rendering improvements
  - Theme system

- [ ] **Desktop Applications**
  - File manager
  - Text editor
  - Terminal emulator
  - System monitor

### System Services
- [ ] **Service Manager**
  - Init system
  - Service dependencies
  - Service monitoring and restart
  - Logging infrastructure

- [ ] **Power Management**
  - ACPI support
  - CPU frequency scaling
  - Sleep/hibernate states
  - Battery management

### Developer Tools
- [ ] **Debugging Tools**
  - Kernel debugger
  - System call tracer
  - Memory profiler
  - Performance profiler

- [ ] **Development Environment**
  - GCC/Clang port
  - Rust toolchain support
  - Make/CMake support
  - Git client

### Hardware Support
- [ ] **Architecture Ports**
  - ARM64 (AArch64) support
  - RISC-V support
  - Consider other architectures

- [ ] **Advanced Devices**
  - Sound drivers (AC97, HDA)
  - Bluetooth support
  - WiFi drivers
  - GPU drivers (Intel, AMD, NVIDIA basic)

---

## Experimental & Research Ideas

### Performance Optimization
- [ ] Kernel preemption
- [ ] Tickless kernel
- [ ] Zero-copy I/O
- [ ] io_uring-style async I/O
- [ ] eBPF support

### Advanced Memory Management
- [ ] Transparent huge pages
- [ ] Memory compression (zswap)
- [ ] NUMA optimization
- [ ] Memory deduplication (KSM)

### Modern Features
- [ ] Containerization support (namespaces, cgroups)
- [ ] Virtualization support (KVM-like)
- [ ] Microkernel exploration
- [ ] Capability-based security

### Rust-Specific Features
- [ ] Async/await in kernel
- [ ] Safe driver development framework
- [ ] Formal verification of critical components
- [ ] Zero-cost abstractions evaluation

---

## Milestones

### Milestone 1: Basic Usability (v0.2.0)
**Target**: Q2 2025
- Process management working
- Basic shell available
- Can run simple user programs
- InitramFS with bundled utilities

### Milestone 2: Storage & Networking (v0.3.0)
**Target**: Q4 2025
- FAT32 file system support
- Storage device drivers
- Basic network stack (ping works)
- USB keyboard/mouse support

### Milestone 3: Self-Hosting (v0.5.0)
**Target**: Q2 2026
- Development tools running on FangaOS
- Can compile and run Rust programs
- Advanced file system support
- TCP/IP networking

### Milestone 4: Desktop Experience (v1.0.0)
**Target**: Q4 2026
- Window system working
- GUI applications
- Service manager
- Stable enough for daily use (for enthusiasts)

---

## Contributing to the Roadmap

We welcome input on the roadmap! If you have:
- **Feature requests**: Open an issue with the `enhancement` label
- **Implementation ideas**: Share in discussions or issues
- **Want to contribute**: Check CONTRIBUTING.md and pick an item from the roadmap

### Priority Labels
- 🔴 **Critical**: Blocking other features, must be done soon
- 🟡 **Important**: Needed for milestone but not blocking
- 🟢 **Nice-to-have**: Would improve the system but not critical
- 🔵 **Experimental**: Research/exploration, may not be implemented

---

## Version History

- **v0.1.0** (Current): Core kernel functionality, memory, interrupts, basic I/O
- **v0.2.0** (Planned): Process management, user mode, basic shell
- **v0.3.0** (Planned): File systems, storage drivers, networking basics
- **v0.5.0** (Future): Self-hosting capability
- **v1.0.0** (Future): Production-ready for enthusiast use

---

## Resources & References

- [OSDev Wiki](https://wiki.osdev.org/) - Comprehensive OS development resources
- [The Rust Programming Language](https://doc.rust-lang.org/book/) - Rust language guide
- [Writing an OS in Rust](https://os.phil-opp.com/) - Excellent Rust OS tutorial
- [Intel Software Developer Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) - x86_64 reference
- [Redox OS](https://www.redox-os.org/) - Rust OS for inspiration
- [Linux Kernel](https://www.kernel.org/) - Reference implementation
- [seL4](https://sel4.systems/) - Formal verification inspiration

---

**Last Updated**: December 2024
**Status**: Active Development
**License**: [To be determined]
