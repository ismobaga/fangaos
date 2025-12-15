#!/bin/bash
# Build script for user-space applications

set -e
set -u

USERSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$USERSPACE_DIR/build"

# Create build directory
mkdir -p "$BUILD_DIR"

echo "Building user-space applications..."

# Build hello world
echo "  - Building hello..."
rustc \
    --target x86_64-unknown-none \
    --crate-type bin \
    --edition 2021 \
    -C panic=abort \
    -C relocation-model=static \
    -C code-model=small \
    -C link-arg=-T"$USERSPACE_DIR/user.ld" \
    -C link-arg=--no-dynamic-linker \
    -C link-arg=-nostdlib \
    -O \
    -o "$BUILD_DIR/hello" \
    "$USERSPACE_DIR/hello.rs"

echo "User-space applications built successfully!"
echo "Output directory: $BUILD_DIR"
