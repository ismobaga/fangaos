# FangaOS Development Guide

A practical guide for developers working on FangaOS.

## Table of Contents
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Building and Testing](#building-and-testing)
- [Debugging](#debugging)
- [Common Tasks](#common-tasks)
- [Tips and Tricks](#tips-and-tricks)

## Getting Started

### Prerequisites

Before you begin, ensure you have:

1. **Rust Nightly** toolchain installed
2. **Build dependencies** for your platform
3. **QEMU** for testing
4. **Basic knowledge** of:
   - Rust programming language
   - x86_64 assembly (helpful but not required)
   - Operating systems concepts

### Quick Setup

```bash
# Clone the repository
git clone https://github.com/ismobaga/fangaos.git
cd fangaos

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly

# Install build dependencies (Ubuntu/Debian)
sudo apt install build-essential qemu-system-x86 xorriso mtools gdisk curl

# Build and run
make run
```

## Development Setup

### Recommended Editor Setup

#### Visual Studio Code

1. Install extensions:
   - **rust-analyzer**: Rust language support
   - **CodeLLDB**: Debugging support
   - **Even Better TOML**: TOML syntax highlighting

2. Create `.vscode/settings.json`:
```json
{
    "rust-analyzer.cargo.target": "x86_64-fanga-kernel.json",
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.checkOnSave.allTargets": false,
    "rust-analyzer.checkOnSave.extraArgs": [
        "--target",
        "x86_64-fanga-kernel.json",
        "-Zbuild-std=core,compiler_builtins,alloc"
    ]
}
```

#### Vim/Neovim

Use `coc.nvim` with `coc-rust-analyzer` or native LSP with `rust-analyzer`.

### Understanding the Project Structure

```
fangaos/
├── kernel/                  # All kernel code
│   ├── crates/
│   │   ├── fanga-kernel/    # Main kernel (arch-independent)
│   │   └── fanga-arch-x86_64/  # x86_64-specific code
│   ├── x86_64-fanga-kernel.json  # Target spec
│   └── linker.ld            # Linker script
├── limine.conf              # Bootloader config
├── Makefile                 # Build system
└── docs/                    # Documentation
```

## Building and Testing

### Basic Commands

```bash
# Build the kernel only
cd kernel && cargo build --release \
    -Z build-std=core,compiler_builtins,alloc \
    -Z build-std-features=compiler-builtins-mem \
    --target x86_64-fanga-kernel.json

# Build and create ISO
make

# Build and run in QEMU
make run

# Clean build artifacts
make clean

# Clean everything including downloaded dependencies
make distclean
```

### Build Options

```bash
# Build for different architectures
make KARCH=x86_64 run      # x86_64 (default)
make KARCH=aarch64 run     # ARM64
make KARCH=riscv64 run     # RISC-V

# Customize QEMU parameters
make run QEMUFLAGS="-m 4G -smp 2"  # 4GB RAM, 2 CPUs

# Build HDD image instead of ISO
make all-hdd
make run-hdd

# Legacy BIOS boot (x86_64 only)
make run-bios
```

### Code Quality Checks

```bash
# Run Clippy (linter)
cd kernel
cargo clippy --target x86_64-fanga-kernel.json \
    -Z build-std=core,compiler_builtins,alloc

# Format code
cargo fmt

# Check formatting without changing files
cargo fmt -- --check
```

## Debugging

### Serial Output

The kernel outputs debug information via COM1 (serial port), which is displayed in the terminal when running `make run`.

Add debug prints anywhere in the kernel:
```rust
use fanga_arch_x86_64::serial_println;

serial_println!("Debug: value = {}", value);
```

### GDB Debugging

1. **Start QEMU with GDB server**:
```bash
make run QEMUFLAGS="-s -S"
# -s: GDB server on localhost:1234
# -S: Pause at startup
```

2. **In another terminal, start GDB**:
```bash
gdb kernel/target/x86_64-fanga-kernel/release/fanga-kernel

(gdb) target remote :1234
(gdb) break _start
(gdb) continue
```

3. **Useful GDB commands**:
```gdb
# Set breakpoint
break function_name
break main.rs:100

# Step through code
step        # Step into
next        # Step over
continue    # Continue execution

# Inspect variables
print variable_name
info registers
x/10x $rsp  # Examine stack

# Backtrace
backtrace
```

### QEMU Monitor

Access QEMU monitor with `Ctrl+Alt+2` (switch back with `Ctrl+Alt+1`):

```
# Useful commands
info registers    # Show CPU registers
info mem          # Show memory mappings
info pic          # Show PIC state
info mtree        # Show memory tree
```

## Common Tasks

### Adding a New Feature

1. **Plan the feature**:
   - Check if it's in ROADMAP.md
   - Open an issue to discuss
   - Design the API

2. **Create a branch**:
```bash
git checkout -b feature/your-feature-name
```

3. **Implement the feature**:
   - Write code following style guidelines
   - Add debug output for testing
   - Document public APIs

4. **Test thoroughly**:
```bash
make run
# Check serial output for errors
```

5. **Submit a PR**:
   - Follow CONTRIBUTING.md guidelines
   - Write clear commit messages
   - Include test results

### Adding Architecture-Specific Code

For x86_64-specific code, add to `kernel/crates/fanga-arch-x86_64/`:

```rust
// kernel/crates/fanga-arch-x86_64/src/new_feature.rs
pub fn arch_specific_function() {
    // Implementation
}

// Expose in lib.rs
pub mod new_feature;
```

For portable code, add to `kernel/crates/fanga-kernel/`:

```rust
// kernel/crates/fanga-kernel/src/lib.rs
pub mod new_module;
```

### Adding a New Device Driver

1. **Create driver module**:
```rust
// kernel/crates/fanga-arch-x86_64/src/drivers/keyboard.rs
pub struct Keyboard {
    // Driver state
}

impl Keyboard {
    pub fn new() -> Self {
        // Initialize
    }
    
    pub fn read_scancode(&self) -> u8 {
        // Read from hardware
    }
}
```

2. **Register interrupt handler**:
```rust
// In interrupts/idt.rs
idt[IRQ_KEYBOARD].set_handler_fn(keyboard_handler);

extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    let scancode = inb(0x60);
    // Handle scancode
    pic::send_eoi(IRQ_KEYBOARD);
}
```

3. **Initialize in kernel**:
```rust
// In fanga-kernel/src/main.rs
let keyboard = Keyboard::new();
```

## Tips and Tricks

### Faster Development Cycle

1. **Use `cargo check` instead of full builds**:
```bash
cd kernel
cargo check --target x86_64-fanga-kernel.json \
    -Z build-std=core,compiler_builtins,alloc
```

2. **Keep QEMU window open**: Faster to restart with existing window

3. **Use serial output heavily**: Faster than switching to QEMU window

### Debugging Tips

1. **Add verbose output**: When debugging, add serial_println! everywhere

2. **Test incrementally**: Test each small change before moving on

3. **Use int3**: Add `asm!("int3")` to trigger debugger breakpoint

4. **Check memory map**: Use Limine's memory map to debug allocation issues

5. **Verify hardware state**: Print register values, PIC masks, IDT entries

### Common Pitfalls

1. **Forgetting to unmask interrupts**:
```rust
// After setting up IDT and PIC
unsafe { asm!("sti"); }  // Enable interrupts
```

2. **Not sending EOI**: Always send End-Of-Interrupt to PIC:
```rust
pic::send_eoi(irq_number);
```

3. **Unsafe without documentation**: Always comment why unsafe is safe:
```rust
// SAFETY: Port 0x3F8 is COM1 and safe to write
unsafe { outb(0x3F8, byte); }
```

4. **Volatile access for MMIO**: Always use volatile for memory-mapped I/O:
```rust
unsafe { ptr.write_volatile(value); }
```

### Performance Tips

1. **Profile with QEMU**: Use `-icount` for deterministic timing

2. **Minimize serial output**: Serial I/O is slow, remove in release builds

3. **Use inline assembly for critical paths**:
```rust
#[inline(always)]
unsafe fn fast_function() {
    asm!("...");
}
```

## Useful Resources

### Documentation
- [OSDev Wiki](https://wiki.osdev.org/) - Comprehensive OS dev resource
- [Intel SDM](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html) - CPU reference manual
- [Rust Embedded Book](https://rust-embedded.github.io/book/)

### Similar Projects
- [blog_os](https://github.com/phil-opp/blog_os) - Teaching OS in Rust
- [Redox OS](https://gitlab.redox-os.org/redox-os/redox) - Unix-like OS in Rust

### Community
- [OSDev Forums](https://forum.osdev.org/)
- [r/osdev](https://www.reddit.com/r/osdev/)
- [Rust OS Dev Discord](https://discord.gg/rust-osdev)

## Getting Help

1. **Check documentation**: README, ROADMAP, ARCHITECTURE
2. **Search existing issues**: Someone may have had the same problem
3. **Ask in discussions**: GitHub Discussions for questions
4. **Open an issue**: For bugs or unclear documentation

## Contributing Back

Once you've made changes:

1. **Test thoroughly**: Make sure it works
2. **Format code**: Run `cargo fmt`
3. **Run linter**: Run `cargo clippy`
4. **Update docs**: If you changed APIs
5. **Submit PR**: Follow CONTRIBUTING.md

Happy hacking! 🚀
