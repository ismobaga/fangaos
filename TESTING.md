# Testing Guide for FangaOS

This document describes the testing infrastructure and how to run tests for FangaOS.

## Test Structure

FangaOS has a comprehensive testing infrastructure with three levels of testing:

### 1. Unit Tests

Unit tests verify individual components in isolation. These tests are located alongside the code they test using Rust's built-in `#[cfg(test)]` attribute.

**Tested Components:**
- Memory address types (`PhysAddr`, `VirtAddr`)
- Alignment functions
- Memory statistics tracking
- Port I/O operations

**Location:** `kernel/crates/fanga-kernel/src/**/*.rs` (inline tests)

### 2. Integration Tests

Integration tests verify interactions between multiple components. These tests are located in the `tests/` directory.

**Test Suites:**
- Memory subsystem integration
- Address type interoperability
- Page allocation patterns

**Location:** `kernel/crates/fanga-kernel/tests/`

### 3. QEMU Integration Tests

QEMU tests verify the kernel boots correctly in a virtualized environment.

**Tests:**
- Kernel boots successfully
- Framebuffer initialization
- Serial output verification

**Location:** `scripts/qemu-test.sh`

## Running Tests

### Prerequisites

Install the required tools:

```bash
# For unit and integration tests
rustup toolchain install nightly
rustup component add rust-src

# For QEMU tests
sudo apt-get install qemu-system-x86 xorriso mtools ovmf
```

### Unit and Integration Tests

Run all unit and integration tests:

```bash
make test
```

Or run tests individually:

```bash
# Unit tests only
cd kernel/crates/fanga-kernel
cargo test --lib --target x86_64-unknown-linux-gnu

# Integration tests only
cd kernel/crates/fanga-kernel
cargo test --tests --target x86_64-unknown-linux-gnu

# Architecture-specific tests
cd kernel/crates/fanga-arch-x86_64
cargo test --lib --target x86_64-unknown-linux-gnu
```

### QEMU Integration Tests

Run QEMU boot test:

```bash
make test-qemu
```

This will:
1. Build the kernel ISO
2. Boot it in QEMU
3. Monitor serial output for success markers
4. Report success or failure

The test succeeds if the kernel:
- Boots without panicking
- Initializes the framebuffer
- Produces expected serial output

### Test Coverage

Generate a test coverage report:

```bash
make coverage
```

This requires `cargo-tarpaulin`:

```bash
cargo install cargo-tarpaulin
```

Coverage reports are generated in `kernel/coverage/`.

## Continuous Integration

FangaOS uses GitHub Actions for continuous integration. The CI pipeline runs:

1. **Test Suite** (`.github/workflows/test.yml`)
   - Unit tests
   - Integration tests
   - Kernel build
   - Code formatting checks
   - Clippy lints

2. **QEMU Integration Tests** (`.github/workflows/qemu-test.yml`)
   - Full kernel boot test in QEMU
   - Serial output verification

The CI runs on:
- Every push to `main` or `develop` branches
- Every pull request

## Writing Tests

### Unit Tests

Add unit tests inline with your code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        let result = my_function();
        assert_eq!(result, expected_value);
    }
}
```

### Integration Tests

Add integration tests in `kernel/crates/fanga-kernel/tests/`:

```rust
//! Integration test for my feature

#![cfg(test)]

use fanga_kernel::memory::{PhysAddr, VirtAddr};

#[test]
fn test_integration_scenario() {
    // Test interactions between components
}
```

### QEMU Tests

To add new QEMU test scenarios, modify `scripts/qemu-test.sh` to check for different success markers or boot conditions.

## Test Configuration

### Conditional Compilation

The kernel library (`lib.rs`) uses conditional compilation to work in both test and kernel environments:

- `#![cfg_attr(not(test), no_std)]` - Use std library for tests
- `#[cfg(not(test))]` - Code only for kernel builds
- `#[cfg(test)]` - Code only for tests

### Cargo Features

The kernel binary is controlled by a feature flag:

```toml
[[bin]]
name = "fanga-kernel"
required-features = ["kernel-bin"]
```

This allows tests to run without linking the kernel entry point.

## Debugging Failed Tests

### Unit/Integration Test Failures

Run with verbose output:

```bash
cd kernel/crates/fanga-kernel
cargo test --target x86_64-unknown-linux-gnu -- --nocapture --test-threads=1
```

### QEMU Test Failures

Check the logs:

```bash
# Serial output from the kernel
cat qemu-serial.log

# QEMU system output
cat qemu-output.log
```

Run QEMU interactively for debugging:

```bash
make run
```

## Benchmarking

While FangaOS currently doesn't have benchmark tests, they can be added using Rust's built-in benchmarking:

```rust
#![feature(test)]
extern crate test;

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_allocation(b: &mut Bencher) {
        b.iter(|| {
            // Code to benchmark
        });
    }
}
```

## Best Practices

1. **Write tests first** - Consider test-driven development for new features
2. **Test edge cases** - Test boundary conditions, zero values, overflow, etc.
3. **Keep tests fast** - Unit tests should run in milliseconds
4. **Test one thing** - Each test should verify one specific behavior
5. **Use descriptive names** - Test names should describe what they test
6. **Add integration tests** - Verify components work together correctly
7. **Run tests frequently** - Run tests before committing changes
8. **Keep tests maintainable** - Refactor tests when refactoring code

## Troubleshooting

### "can't find crate for `core`"

This error occurs when building for the custom target. Always specify the host target for tests:

```bash
cargo test --target x86_64-unknown-linux-gnu
```

### "duplicate symbol: _start"

This means the kernel binary is being built for tests. Make sure to build only the library:

```bash
cargo test --lib  # Only build library, not binary
```

### QEMU test times out

Increase the timeout in `scripts/qemu-test.sh`:

```bash
TIMEOUT=60  # Increase from 30 to 60 seconds
```

## Future Improvements

- [ ] Add property-based testing with `proptest`
- [ ] Add fuzzing tests for memory allocator
- [ ] Add performance benchmarks
- [ ] Add test coverage requirements (e.g., >80%)
- [ ] Add mutation testing
- [ ] Add stress tests for memory management
- [ ] Add tests for concurrent operations
- [ ] Add regression tests for bug fixes
