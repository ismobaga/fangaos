# FangaOS Development Roadmap

## Vision

FangaOS aims to be a modern, secure, and efficient x86_64 operating system kernel written entirely in Rust. The project leverages Rust's memory safety guarantees and modern development practices to create a maintainable, well-tested OS suitable for both educational purposes and real-world applications.

## Current State

### ✅ Implemented Features

FangaOS has successfully implemented a solid foundation for an operating system kernel:

#### Memory Management
- **Physical Memory Manager (PMM)**: Bitmap-based allocator for physical memory
- **Virtual Memory Manager (VMM)**: Full paging support with page table management
- **Heap Allocator**: Dynamic memory allocation using Rust's allocator trait
- **Memory Statistics**: Debug tracking and monitoring of memory usage

#### Interrupt Handling
- **IDT Setup**: Complete Interrupt Descriptor Table configuration
- **Exception Handlers**: CPU exception handling (page faults, general protection, etc.)
- **Hardware Interrupts**: Support for both PIC and APIC interrupt controllers
- **IRQ Management**: Proper interrupt routing and handling

#### System Calls
- **SYSCALL/SYSRET**: Fast system call interface using modern x86_64 instructions
- **Basic Syscalls**: Implementation of read, write, and exit syscalls
- **Argument Validation**: Security-focused validation of syscall parameters
- **Error Handling**: Robust error codes and return value handling

#### Input/Output System
- **Framebuffer Console**: Custom font rendering with scrolling support
- **Serial Port**: RS-232 communication for debugging and logging
- **Keyboard Driver**: PS/2 keyboard with full scancode translation
- **Console Abstraction**: Unified interface for multiple output devices
- **Logging Framework**: Four-level logging (DEBUG, INFO, WARN, ERROR) with color coding

#### Architecture Support
- **x86_64 Primary**: Full support for 64-bit x86 architecture
- **UEFI Boot**: Boot support via Limine bootloader
- **GDT/TSS**: Global Descriptor Table and Task State Segment setup

#### Testing & CI/CD
- **Unit Tests**: Comprehensive unit testing of individual components
- **Integration Tests**: Cross-component interaction testing
- **QEMU Tests**: Automated boot testing in virtualized environment
- **Coverage Reports**: Test coverage tracking with tarpaulin
- **GitHub Actions**: Automated testing, building, and quality checks

### 🚧 In Progress

#### Process Management (Basic)
- Initial task structures
- Context switching framework (partial)

---

## Short-Term Goals (1-3 Months)

### Priority 1: Process Management

**Goal**: Implement a complete process management system

- [ ] **Process Control Block (PCB)**: Design and implement process metadata structure
- [ ] **Process Scheduler**: Round-robin or priority-based scheduler
- [ ] **Context Switching**: Full x86_64 context save/restore
- [ ] **Process Creation/Termination**: `fork()` and `exit()` syscalls
- [ ] **Process States**: Running, ready, waiting, terminated
- [ ] **Multi-tasking Demo**: Run at least 2-3 concurrent processes

**Estimated Time**: 3-4 weeks

### Priority 2: Enhanced Keyboard Input

**Goal**: Make keyboard input fully interactive

- [ ] **Input Echo**: Echo keyboard input to framebuffer console
- [ ] **Line Editing**: Backspace, delete, cursor movement
- [ ] **Input Buffering**: Proper line-based input handling
- [ ] **Special Keys**: Ctrl+C, Ctrl+D, arrow keys

**Estimated Time**: 1-2 weeks

### Priority 3: Basic Shell/REPL

**Goal**: Create an interactive command-line interface

- [ ] **Command Parser**: Parse user input into commands
- [ ] **Built-in Commands**: help, clear, echo, memory, ps, exit
- [ ] **Command History**: Up/down arrow for previous commands
- [ ] **Tab Completion**: Basic command name completion
- [ ] **Prompt**: Customizable command prompt

**Estimated Time**: 2-3 weeks

### Priority 4: Timer and Time Management

**Goal**: Implement system timing and scheduling support

- [ ] **PIT or APIC Timer**: Configure hardware timer
- [ ] **Timer IRQ Handler**: Handle timer interrupts
- [ ] **Tick Counter**: System uptime tracking
- [ ] **Sleep/Delay**: Process sleeping and time delays
- [ ] **Preemptive Scheduling**: Time-slice based scheduling

**Estimated Time**: 1-2 weeks

---

## Medium-Term Goals (3-6 Months)

### Phase 1: Inter-Process Communication (IPC)

**Goal**: Enable processes to communicate

- [ ] **Message Passing**: Send/receive message queues
- [ ] **Shared Memory**: Shared memory segments between processes
- [ ] **Pipes**: Anonymous and named pipes
- [ ] **Signals**: Basic signal handling mechanism
- [ ] **IPC Syscalls**: Complete set of IPC system calls

**Estimated Time**: 4-6 weeks

### Phase 2: Virtual File System (VFS)

**Goal**: Implement a file system abstraction layer

- [ ] **VFS Architecture**: Pluggable file system interface
- [ ] **In-Memory FS**: Simple RAM-based file system (first implementation)
- [ ] **File Operations**: open, close, read, write, seek
- [ ] **Directory Operations**: mkdir, rmdir, readdir
- [ ] **File Descriptors**: Per-process file descriptor table
- [ ] **Path Resolution**: Absolute and relative path handling

**Estimated Time**: 6-8 weeks

### Phase 3: Storage and File Systems

**Goal**: Support persistent storage

- [ ] **ATA/AHCI Driver**: Hard disk/SSD access
- [ ] **Partition Support**: MBR and GPT partition tables
- [ ] **FAT32 Support**: Read/write FAT32 file system
- [ ] **Simple FS**: Custom simple file system (optional)
- [ ] **Disk Caching**: Buffer cache for disk I/O

**Estimated Time**: 6-8 weeks

### Phase 4: Memory Management Enhancements

**Goal**: Improve memory management capabilities

- [ ] **Copy-on-Write (CoW)**: Efficient memory sharing
- [ ] **Memory Mapping**: mmap/munmap syscalls
- [ ] **Demand Paging**: Load pages only when needed
- [ ] **Page Replacement**: LRU or similar algorithm
- [ ] **Memory Protection**: Better isolation between processes
- [ ] **Swapping**: Basic swap space support (optional)

**Estimated Time**: 4-5 weeks

---

## Long-Term Goals (6-12 Months)

### Phase 1: User Space

**Goal**: Support for user-mode applications

- [ ] **User/Kernel Mode Separation**: Proper privilege separation
- [ ] **User-space Applications**: Sample applications in user mode
- [ ] **Dynamic Linker**: Basic ELF dynamic linking
- [ ] **Standard Library Port**: Minimal libc or newlib port
- [ ] **Application Loader**: Load and execute ELF binaries

**Estimated Time**: 8-10 weeks

### Phase 2: Networking

**Goal**: Basic networking capabilities

- [ ] **Network Card Driver**: E1000 or virtio-net driver
- [ ] **Ethernet Layer**: Frame handling
- [ ] **ARP Protocol**: Address resolution
- [ ] **IP Stack**: IPv4 support
- [ ] **UDP Protocol**: Connectionless networking
- [ ] **TCP Protocol**: Reliable connections
- [ ] **Socket API**: BSD-style socket interface
- [ ] **DHCP Client**: Automatic network configuration

**Estimated Time**: 10-12 weeks

### Phase 3: Advanced I/O

**Goal**: Enhanced input/output capabilities

- [ ] **USB Stack**: USB host controller support
- [ ] **USB Keyboard/Mouse**: USB HID device support
- [ ] **Multiple Keyboard Layouts**: Support for non-US keyboards
- [ ] **Mouse Support**: PS/2 and USB mouse input
- [ ] **Framebuffer Improvements**: Multiple resolutions, double buffering
- [ ] **Virtual Terminals**: Multiple virtual consoles (Alt+F1, F2, etc.)

**Estimated Time**: 8-10 weeks

### Phase 4: Advanced Process Features

**Goal**: Production-ready process management

- [ ] **Multi-threading**: Kernel threads and user threads
- [ ] **Thread Synchronization**: Mutexes, semaphores, condition variables
- [ ] **Real-time Scheduling**: Priority-based scheduling with real-time support
- [ ] **Process Groups**: Job control and sessions
- [ ] **Signals (Advanced)**: Full POSIX-like signal support
- [ ] **Core Dumps**: Process state dumps for debugging

**Estimated Time**: 6-8 weeks

---

## Future Ideas & Exploration

### Advanced Features (12+ Months)

These are aspirational features for consideration after core functionality is stable:

#### Graphics & UI
- [ ] **GUI Framework**: Basic windowing system
- [ ] **Graphics Driver**: VGA, VESA, or modern GPU support
- [ ] **Compositor**: Window compositing and effects
- [ ] **Widget Toolkit**: Basic UI controls and widgets
- [ ] **Multi-monitor**: Multiple display support

#### Scalability & Performance
- [ ] **SMP Support**: Multi-core CPU support
- [ ] **CPU Affinity**: Process pinning to specific cores
- [ ] **NUMA Awareness**: Non-uniform memory access optimization
- [ ] **Performance Profiling**: Built-in profiling tools
- [ ] **Kernel Preemption**: Fully preemptible kernel

#### Security Enhancements
- [ ] **Capabilities**: Fine-grained privilege system
- [ ] **SELinux-style Security**: Mandatory access control
- [ ] **Secure Boot**: Verified boot chain
- [ ] **ASLR**: Address space layout randomization
- [ ] **Stack Protection**: Canaries and NX bit enforcement
- [ ] **Audit Logging**: Security event logging

#### Advanced Storage
- [ ] **ext2/ext4 Support**: Linux file system compatibility
- [ ] **NTFS Support**: Windows file system read support
- [ ] **File System Journaling**: Crash recovery
- [ ] **RAID Support**: Software RAID implementation
- [ ] **Encryption**: File system encryption (dm-crypt style)
- [ ] **NVMe Driver**: Modern SSD support

#### Networking Advanced
- [ ] **IPv6 Support**: Modern IP protocol
- [ ] **TLS/SSL**: Secure networking
- [ ] **HTTP Client/Server**: Web protocols
- [ ] **DNS Client**: Domain name resolution
- [ ] **Firewall**: Packet filtering
- [ ] **Network Stack Optimization**: Zero-copy, TSO, etc.

#### Virtualization
- [ ] **KVM-style Hypervisor**: Type-1 hypervisor capabilities
- [ ] **Paravirtualization**: Guest drivers for VMs
- [ ] **Container Support**: Lightweight process isolation

#### Developer Experience
- [ ] **Debugger Support**: GDB/LLDB integration improvements
- [ ] **Kernel Modules**: Loadable kernel modules
- [ ] **eBPF-style System**: Programmable kernel extensions
- [ ] **Crash Analysis**: Kernel crash dumps and analysis
- [ ] **Performance Counters**: PMU access and monitoring

#### Platform Support
- [ ] **ARM64 Port**: AArch64 architecture support
- [ ] **RISC-V Port**: RISC-V architecture support
- [ ] **Raspberry Pi**: Support for popular ARM boards

---

## Development Priorities

### Core Principles

1. **Memory Safety First**: Leverage Rust's safety guarantees
2. **Test-Driven**: Maintain high test coverage (>80%)
3. **Documentation**: Keep documentation in sync with code
4. **Performance**: Optimize hot paths, but prioritize correctness
5. **Modularity**: Keep components loosely coupled
6. **Standards**: Follow POSIX where applicable

### Quality Metrics

- **Test Coverage**: Maintain >80% code coverage
- **CI/CD**: All tests must pass before merging
- **Code Review**: All changes require review
- **Documentation**: All public APIs must be documented
- **Security**: Zero known security vulnerabilities
- **Linting**: Zero clippy warnings on code changes

---

## Release Strategy

### Version Milestones

#### v0.1.0 - "Foundation" (Current)
- ✅ Memory management
- ✅ Interrupt handling
- ✅ System calls
- ✅ Basic I/O
- ✅ Testing infrastructure

#### v0.2.0 - "Interactive" (Short-term goal)
- Process management
- Interactive shell
- Timer support
- Enhanced keyboard input

#### v0.3.0 - "Communication" (Medium-term goal)
- IPC mechanisms
- Virtual file system
- Storage drivers
- Memory management enhancements

#### v0.4.0 - "Practical" (Long-term goal)
- User space support
- Basic networking
- Advanced I/O
- Multi-threading

#### v0.5.0 - "Production" (Future)
- SMP support
- Advanced networking
- Security features
- Performance optimizations

#### v1.0.0 - "Stable" (Vision)
- Feature complete for general use
- Stable APIs
- Production-ready reliability
- Comprehensive documentation

---

## Contributing to the Roadmap

### How to Get Involved

1. **Choose a Task**: Pick an item from the roadmap
2. **Discuss**: Open an issue to discuss your approach
3. **Implement**: Follow the contribution guidelines
4. **Test**: Write comprehensive tests
5. **Document**: Update relevant documentation
6. **Review**: Submit PR and address feedback

### Skill Levels

- **Beginner**: Documentation, testing, bug fixes
- **Intermediate**: Driver development, syscalls, I/O
- **Advanced**: Memory management, scheduling, networking
- **Expert**: Architecture changes, SMP, security

---

## Resources & References

### Learning Resources
- [OSDev Wiki](https://wiki.osdev.org/)
- [Intel Software Developer Manuals](https://www.intel.com/sdm)
- [AMD Architecture Programmer's Manual](https://www.amd.com/en/support/tech-docs)
- [Writing an OS in Rust](https://os.phil-opp.com/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)

### Inspirational Projects
- [Redox OS](https://www.redox-os.org/) - Unix-like OS in Rust
- [Tock OS](https://www.tockos.org/) - Embedded OS in Rust
- [Linux Kernel](https://kernel.org/) - Reference implementation
- [SerenityOS](https://serenityos.org/) - From-scratch Unix-like OS

---

## Contact & Community

- **Repository**: https://github.com/ismobaga/fangaos
- **Issues**: Use GitHub Issues for bugs and feature requests
- **Discussions**: Use GitHub Discussions for questions and ideas

---

## Changelog

- **2025-12-14**: Initial roadmap created with comprehensive short/medium/long-term goals
