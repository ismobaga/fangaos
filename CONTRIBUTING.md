# Contributing to FangaOS

First off, thank you for considering contributing to FangaOS! It's people like you that make FangaOS a great learning and development project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please be respectful, inclusive, and constructive in all interactions.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, include:

- **Clear title and description**
- **Steps to reproduce** the issue
- **Expected vs. actual behavior**
- **Environment details** (OS, Rust version, QEMU version)
- **Log output** (serial output from QEMU)
- **Screenshots** if applicable

Use the bug report template:

```markdown
**Description:**
Brief description of the issue

**Steps to Reproduce:**
1. Run command X
2. Observe behavior Y

**Expected Behavior:**
What should happen

**Actual Behavior:**
What actually happens

**Environment:**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., nightly-2024-12-01]
- QEMU version: [e.g., 8.0.0]

**Additional Context:**
Add any other context, logs, or screenshots
```

### Suggesting Features

Feature suggestions are welcome! Please:

1. **Check the [ROADMAP.md](ROADMAP.md)** to see if it's already planned
2. **Open an issue** with the "feature request" label
3. **Describe the feature** and its use case
4. **Explain why** it would be valuable to FangaOS
5. **Provide examples** of similar features in other projects

### Contributing Code

#### Good First Issues

Look for issues labeled `good-first-issue` - these are great starting points for new contributors:

- Documentation improvements
- Code comments and clarifications
- Simple bug fixes
- Small feature additions

#### Areas Needing Help

Check [ROADMAP.md](ROADMAP.md) for current priorities:

- **High Priority**: Memory management, keyboard driver, testing framework
- **Medium Priority**: File system, multitasking, device drivers
- **Documentation**: README improvements, code documentation, tutorials

## Getting Started

### Prerequisites

1. **Install Rust** (nightly):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default nightly
   ```

2. **Install build tools**:
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential qemu-system-x86 xorriso mtools gdisk
   
   # Arch Linux
   sudo pacman -S base-devel qemu-full xorriso mtools gptfdisk
   ```

3. **Clone the repository**:
   ```bash
   git clone https://github.com/ismobaga/fangaos.git
   cd fangaos
   ```

4. **Build and test**:
   ```bash
   make run
   ```

### Development Environment

We recommend:
- **Editor**: VS Code with rust-analyzer extension
- **Terminal**: Any modern terminal with good scrollback
- **Debugger**: GDB or LLDB for kernel debugging

### Understanding the Codebase

```
kernel/
├── crates/
│   ├── fanga-kernel/          # Main kernel (architecture-independent)
│   │   └── src/main.rs        # Kernel entry point, initialization
│   └── fanga-arch-x86_64/     # x86_64-specific code
│       └── src/
│           ├── lib.rs         # Architecture initialization
│           ├── interrupts/    # IDT, PIC, IRQ handlers
│           ├── serial.rs      # Serial port driver
│           └── port.rs        # I/O port access
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

Branch naming conventions:
- `feature/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation changes
- `refactor/*` - Code refactoring
- `test/*` - Test additions/changes

### 2. Make Changes

- Write clear, idiomatic Rust code
- Follow existing code style
- Add comments for complex logic
- Update documentation if needed

### 3. Test Your Changes

```bash
# Build and run
make run

# Check for compilation errors
cd kernel && cargo check

# Run clippy for lints
cd kernel && cargo clippy

# Format code
cd kernel && cargo fmt
```

### 4. Commit Changes

```bash
git add .
git commit -m "type: brief description"
```

See [Commit Guidelines](#commit-guidelines) below.

### 5. Push and Create PR

```bash
git push origin your-branch-name
```

Then create a pull request on GitHub.

## Coding Standards

### Rust Style

Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/):

- Use `cargo fmt` to format code (run before committing)
- Use `cargo clippy` to catch common mistakes
- Prefer standard library types when available
- Use meaningful variable names

### Kernel-Specific Guidelines

1. **Safety**: Mark unsafe blocks and explain why they're safe
   ```rust
   // SAFETY: COM1 port is initialized and valid for x86_64
   unsafe {
       outb(COM1, byte);
   }
   ```

2. **No panics in critical paths**: Use `Result` or handle errors gracefully

3. **Volatile access**: Use `read_volatile`/`write_volatile` for MMIO

4. **Atomics**: Use appropriate ordering (usually `Acquire`/`Release`)

5. **Documentation**: Document all public APIs
   ```rust
   /// Initializes the serial port COM1 for debug output.
   ///
   /// # Safety
   /// Must be called only once during kernel initialization.
   pub fn init() {
       // ...
   }
   ```

### Code Organization

- Keep files under 500 lines when possible
- Group related functionality in modules
- Use meaningful module names
- Export public APIs through `mod.rs`

## Commit Guidelines

### Commit Message Format

```
type: brief description (50 chars or less)

More detailed explanation if needed (wrap at 72 chars).
Explain what changed and why, not how.

Fixes #123
```

### Commit Types

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, no logic change)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Build system, dependencies, etc.
- `perf:` - Performance improvements

### Examples

```
feat: add keyboard interrupt handler

Implements PS/2 keyboard interrupt handler to read scancodes
from port 0x60. Handles basic key press/release events.

Relates to #42
```

```
fix: correct page table mapping for HHDM

The previous implementation had incorrect offset calculation
for higher half direct map, causing page faults on certain
memory accesses.

Fixes #87
```

## Pull Request Process

### Before Submitting

- [ ] Code compiles without warnings
- [ ] `cargo fmt` has been run
- [ ] `cargo clippy` shows no issues
- [ ] Changes are tested in QEMU
- [ ] Documentation is updated if needed
- [ ] Commit messages follow guidelines

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Refactoring

## Testing
How the changes were tested

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings
- [ ] Tested in QEMU

## Related Issues
Fixes #issue_number
```

### Review Process

1. **Automated checks** run (if CI is set up)
2. **Maintainer review** - may request changes
3. **Discussion** - address feedback and questions
4. **Approval** - maintainer approves changes
5. **Merge** - changes are merged to main branch

### After Merge

- Your changes will be in the next release
- Close any related issues
- Update any related documentation
- Thank you for contributing! 🎉

## Architecture-Specific Contributions

When adding architecture-specific code:

1. **Create new crate**: `kernel/crates/fanga-arch-{arch}/`
2. **Mirror x86_64 structure**: Keep similar organization
3. **Abstract common parts**: Use traits for shared interfaces
4. **Document differences**: Note architecture-specific quirks

Example:
```rust
// kernel/crates/fanga-arch-common/src/lib.rs
pub trait ArchInit {
    fn init();
}

// kernel/crates/fanga-arch-x86_64/src/lib.rs
impl ArchInit for X86_64 {
    fn init() { /* ... */ }
}
```

## Questions?

- **General questions**: Open a GitHub Discussion
- **Bug reports**: Open an Issue
- **Feature ideas**: Check ROADMAP.md, then open an Issue
- **Code questions**: Comment on the relevant PR or Issue

## Recognition

Contributors are recognized in:
- Git history (your commits)
- Release notes (for significant contributions)
- Special thanks in documentation (for major features)

Thank you for making FangaOS better! 🚀
