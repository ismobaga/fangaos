# Contributing to FangaOS

Thank you for your interest in contributing to FangaOS! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Community](#community)

---

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. All contributors are expected to:

- Be respectful and considerate
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Respect differing viewpoints and experiences
- Accept responsibility and apologize for mistakes

Unacceptable behavior includes harassment, discrimination, or any form of disrespect. Please report concerns to the project maintainers.

---

## Getting Started

### Prerequisites

Before contributing, make sure you have:

1. **Rust Nightly Toolchain**
   ```bash
   rustup toolchain install nightly
   rustup component add rust-src
   rustup default nightly
   ```

2. **Build Dependencies**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install -y xorriso mtools qemu-system-x86 ovmf lld
   
   # Fedora
   sudo dnf install -y xorriso mtools qemu-system-x86 edk2-ovmf lld
   
   # Arch Linux
   sudo pacman -S xorriso mtools qemu-system-x86 edk2-ovmf lld
   ```

3. **Recommended Tools**
   ```bash
   # Code formatting and linting
   rustup component add rustfmt clippy
   
   # Coverage reporting (optional)
   cargo install cargo-tarpaulin
   ```

### Building the Project

```bash
# Clone the repository
git clone https://github.com/ismobaga/fangaos.git
cd fangaos

# Build the kernel
make all

# Run in QEMU
make run

# Run tests
make test
make test-qemu
```

---

## Development Setup

### Recommended IDE Setup

1. **Visual Studio Code**
   - Install Rust Analyzer extension
   - Install CodeLLDB for debugging
   - Recommended settings in `.vscode/settings.json`:
     ```json
     {
       "rust-analyzer.cargo.target": "x86_64-fanga-kernel",
       "rust-analyzer.checkOnSave.allTargets": false
     }
     ```

2. **IntelliJ IDEA / CLion**
   - Install Rust plugin
   - Configure Rust toolchain to nightly

### Project Structure

```
fangaos/
├── kernel/                    # Kernel source code
│   ├── crates/
│   │   ├── fanga-kernel/     # Main kernel crate
│   │   └── fanga-arch-x86_64/# Architecture-specific code
│   ├── linker.ld             # Linker script
│   └── x86_64-fanga-kernel.json  # Custom target spec
├── scripts/                   # Build and test scripts
├── docs/                      # Documentation
├── .github/workflows/        # CI/CD configuration
└── Makefile                  # Build system
```

---

## How to Contribute

### Finding Work

1. **Check the Roadmap**: See [ROADMAP.md](ROADMAP.md) for planned features
2. **Browse Issues**: Look for issues labeled `good first issue` or `help wanted`
3. **Ask in Discussions**: Not sure where to start? Ask in GitHub Discussions

### Types of Contributions

We welcome:
- **Bug fixes**: Fix issues reported in GitHub Issues
- **New features**: Implement features from the roadmap
- **Documentation**: Improve docs, add examples, fix typos
- **Tests**: Add unit tests, integration tests, or QEMU tests
- **Performance**: Optimize existing code
- **Code review**: Review open pull requests

### Workflow

1. **Fork the Repository**
   ```bash
   # Fork on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/fangaos.git
   cd fangaos
   git remote add upstream https://github.com/ismobaga/fangaos.git
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-123
   ```

3. **Make Changes**
   - Write code following our coding standards
   - Add tests for new functionality
   - Update documentation as needed

4. **Test Your Changes**
   ```bash
   # Format code
   cd kernel && cargo fmt --all
   
   # Run linter
   cd kernel/crates/fanga-kernel && cargo clippy
   
   # Run tests
   make test
   make test-qemu
   ```

5. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   # See commit message guidelines below
   ```

6. **Push and Create Pull Request**
   ```bash
   git push origin feature/your-feature-name
   # Then create PR on GitHub
   ```

---

## Coding Standards

### Rust Style Guide

We follow the standard Rust style guide with some additions:

1. **Formatting**
   - Run `cargo fmt` before committing
   - Use default rustfmt configuration
   - Maximum line length: 100 characters

2. **Naming Conventions**
   - `snake_case` for functions, variables, modules
   - `CamelCase` for types, traits, enums
   - `SCREAMING_SNAKE_CASE` for constants
   - Descriptive names over abbreviations

3. **Code Organization**
   - One module per file when possible
   - Group related functionality
   - Keep functions small and focused (< 50 lines ideally)
   - Use meaningful comments for complex logic

4. **Error Handling**
   - Use `Result<T, E>` for recoverable errors
   - Use `Option<T>` for optional values
   - Avoid `unwrap()` and `expect()` in kernel code
   - Document error conditions

5. **Safety**
   - Minimize `unsafe` code
   - Document all `unsafe` blocks with safety justification
   - Prefer safe abstractions
   - Use `#[deny(unsafe_op_in_unsafe_fn)]`

### Examples

**Good:**
```rust
/// Allocates a physical page from the page allocator.
///
/// Returns `Some(PhysAddr)` on success, or `None` if no memory is available.
pub fn alloc_page(&mut self) -> Option<PhysAddr> {
    self.find_free_page()
        .map(|page_num| PhysAddr::new(page_num * PAGE_SIZE))
}
```

**Bad:**
```rust
// This gets a page
pub fn get_pg(&mut self) -> PhysAddr {
    let p = self.ffp().unwrap();  // Can panic!
    PhysAddr::new(p * 4096)
}
```

### Clippy Lints

Run `cargo clippy` and address warnings. We aim for zero clippy warnings. Some lints are allowed:

```rust
#![allow(clippy::missing_safety_doc)]  // Only if safety is obvious
```

---

## Testing Guidelines

### Test Coverage

- **All new features must include tests**
- **Bug fixes should include regression tests**
- Aim for >80% code coverage on new code

### Test Types

1. **Unit Tests**
   - Test individual functions and types
   - Located in same file as code (`#[cfg(test)] mod tests`)
   - Fast and isolated

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
   
       #[test]
       fn test_addr_alignment() {
           let addr = PhysAddr::new(0x1000);
           assert!(addr.is_aligned(0x1000));
       }
   }
   ```

2. **Integration Tests**
   - Test component interactions
   - Located in `kernel/crates/fanga-kernel/tests/`
   - Use public API only

   ```rust
   #[test]
   fn test_memory_allocation_flow() {
       // Test PMM + VMM interaction
   }
   ```

3. **QEMU Tests**
   - Test kernel boot and initialization
   - Modify `scripts/qemu-test.sh`
   - Verify serial output

### Running Tests

```bash
# All tests
make test

# Specific test
cd kernel/crates/fanga-kernel
cargo test --target x86_64-unknown-linux-gnu test_name

# QEMU test
make test-qemu

# Coverage report
make coverage
```

### Test Guidelines

- Use descriptive test names: `test_allocation_fails_when_out_of_memory`
- Test edge cases: zero, maximum, overflow, underflow
- Test error paths, not just success paths
- Keep tests simple and focused
- Avoid test interdependencies

---

## Documentation

### Code Documentation

1. **Public API Must Be Documented**
   ```rust
   /// Brief description of the function.
   ///
   /// Longer description with details about behavior,
   /// edge cases, and examples.
   ///
   /// # Arguments
   ///
   /// * `addr` - The physical address to map
   /// * `flags` - Page table flags
   ///
   /// # Returns
   ///
   /// Returns `Ok(())` on success, or an error if mapping fails.
   ///
   /// # Examples
   ///
   /// ```no_run
   /// let addr = PhysAddr::new(0x1000);
   /// mapper.map(addr, flags)?;
   /// ```
   ///
   /// # Safety
   ///
   /// This function is unsafe because...
   pub unsafe fn map(&mut self, addr: PhysAddr, flags: Flags) -> Result<(), Error> {
       // Implementation
   }
   ```

2. **Module Documentation**
   ```rust
   //! Memory management subsystem.
   //!
   //! This module provides physical and virtual memory management
   //! for the FangaOS kernel.
   ```

3. **Complex Logic**
   - Add inline comments for non-obvious code
   - Explain "why", not "what"
   - Link to references (specs, papers, other OSes)

### Project Documentation

When adding features, update:
- `README.md` - Add to features list if user-visible
- `docs/` - Add technical documentation
- `ROADMAP.md` - Mark items as complete
- Code examples in documentation

---

## Pull Request Process

### Before Submitting

- [ ] Code builds without errors
- [ ] All tests pass (`make test` and `make test-qemu`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Commit messages follow conventions

### PR Title Format

Use conventional commits format:

```
<type>(<scope>): <description>

Examples:
feat(memory): add slab allocator
fix(keyboard): handle key release correctly
docs(readme): update build instructions
test(pmm): add page allocation edge case tests
refactor(io): simplify console abstraction
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `test`: Adding tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `style`: Formatting, no code change
- `chore`: Build, CI, dependencies

**Scopes:**
- `memory`, `interrupt`, `syscall`, `io`, `task`, `arch`, `build`, `ci`, etc.

### PR Description

Include:
- **What**: What does this PR do?
- **Why**: Why is this change needed?
- **How**: How does it work? (for complex changes)
- **Testing**: How was it tested?
- **Closes**: `Closes #123` to auto-close issues

**Template:**
```markdown
## Description
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- Detailed list of changes
- Another change

## Testing
How was this tested?
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] QEMU test passes
- [ ] Manually tested in QEMU

## Checklist
- [ ] Code builds without errors
- [ ] Tests pass
- [ ] Code formatted with rustfmt
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] ROADMAP.md updated (if applicable)

## Related Issues
Closes #123
```

### Review Process

1. **Automated Checks**: CI must pass (build, tests, clippy)
2. **Code Review**: At least one maintainer must approve
3. **Discussion**: Address review comments
4. **Merge**: Maintainers will merge approved PRs

### Review Criteria

- Code quality and style
- Test coverage
- Documentation completeness
- Performance implications
- Security considerations
- Architectural fit

---

## Issue Reporting

### Bug Reports

Use the bug report template and include:
- **Description**: What's wrong?
- **Steps to Reproduce**: How to trigger the bug?
- **Expected**: What should happen?
- **Actual**: What actually happens?
- **Environment**: OS, Rust version, commit hash
- **Logs**: Relevant output or serial logs

### Feature Requests

Use the feature request template and include:
- **Problem**: What problem does this solve?
- **Proposed Solution**: How should it work?
- **Alternatives**: Other approaches considered?
- **Impact**: Who benefits? Breaking changes?

### Questions

For questions and discussions, use GitHub Discussions instead of issues.

---

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions, ideas, general discussion
- **Pull Requests**: Code review and technical discussion

### Getting Help

- Check existing documentation (README, docs/, ROADMAP)
- Search existing issues and discussions
- Ask in GitHub Discussions
- Reference relevant documentation in questions

### Recognition

Contributors are recognized in:
- Git commit history
- Release notes
- Contributors list (coming soon)

---

## Advanced Topics

### Working with `unsafe` Code

When `unsafe` is necessary:
1. Keep unsafe blocks minimal
2. Document safety invariants
3. Add safety comments explaining why it's safe
4. Consider safe wrapper APIs

```rust
/// # Safety
///
/// Caller must ensure `addr` points to valid, initialized memory.
pub unsafe fn read_volatile(addr: usize) -> u8 {
    // SAFETY: Caller guarantees addr is valid
    unsafe { core::ptr::read_volatile(addr as *const u8) }
}
```

### Architecture-Specific Code

- Place in `fanga-arch-x86_64` crate
- Use traits for architecture abstraction
- Document x86_64 specifics
- Consider portability for future architectures

### Performance Optimization

- Profile before optimizing
- Document performance characteristics in module docs
- Add benchmarks for critical paths (when available)
- Consider trade-offs (speed vs. code size vs. safety)

---

## License

By contributing to FangaOS, you agree that your contributions will be licensed under the same license as the project.

---

## Questions?

If you have questions about contributing, please:
1. Check this guide and other documentation
2. Search existing issues and discussions
3. Ask in GitHub Discussions

Thank you for contributing to FangaOS! 🚀
