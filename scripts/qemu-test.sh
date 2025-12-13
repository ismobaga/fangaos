#!/bin/bash
# QEMU automated testing script for FangaOS
# This script boots the kernel in QEMU and checks for successful initialization

set -e

TIMEOUT=30
KARCH="${KARCH:-x86_64}"
IMAGE_NAME="fangaos-${KARCH}"
SUCCESS_MARKER="\[Fanga\] Framebuffer console initialized"
SERIAL_LOG="qemu-serial.log"
QEMU_LOG="qemu-output.log"

echo "==================================="
echo "FangaOS QEMU Test Runner"
echo "==================================="
echo "Architecture: ${KARCH}"
echo "Image: ${IMAGE_NAME}.iso"
echo "Timeout: ${TIMEOUT}s"
echo "==================================="

# Check if ISO exists
if [ ! -f "${IMAGE_NAME}.iso" ]; then
    echo "❌ Error: ${IMAGE_NAME}.iso not found!"
    echo "Please run 'make all' first to build the ISO."
    exit 1
fi

# Check if OVMF files exist
if [ ! -f "ovmf/ovmf-code-${KARCH}.fd" ] || [ ! -f "ovmf/ovmf-vars-${KARCH}.fd" ]; then
    echo "⚠️  OVMF firmware not found. Attempting to download..."
    make ovmf/ovmf-code-${KARCH}.fd ovmf/ovmf-vars-${KARCH}.fd
fi

# Clean up previous logs
rm -f "${SERIAL_LOG}" "${QEMU_LOG}"

echo ""
echo "🚀 Starting QEMU..."
echo "   Serial output: ${SERIAL_LOG}"
echo ""

# Run QEMU with serial output redirected to file
# Use -display none for headless operation
# Add -no-reboot to exit after panic
timeout ${TIMEOUT} qemu-system-${KARCH} \
    -M q35 \
    -m 2G \
    -drive if=pflash,unit=0,format=raw,file=ovmf/ovmf-code-${KARCH}.fd,readonly=on \
    -drive if=pflash,unit=1,format=raw,file=ovmf/ovmf-vars-${KARCH}.fd \
    -cdrom ${IMAGE_NAME}.iso \
    -serial file:${SERIAL_LOG} \
    -display none \
    -no-reboot \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    > "${QEMU_LOG}" 2>&1 &

QEMU_PID=$!

echo "   QEMU PID: ${QEMU_PID}"
echo ""

# Wait for QEMU to produce output
sleep 3

# Monitor serial output for success markers
ELAPSED=0
FOUND=0

while [ ${ELAPSED} -lt ${TIMEOUT} ]; do
    if [ ! -f "${SERIAL_LOG}" ]; then
        sleep 1
        ELAPSED=$((ELAPSED + 1))
        continue
    fi
    
    # Check for success marker
    if grep -q "${SUCCESS_MARKER}" "${SERIAL_LOG}"; then
        FOUND=1
        break
    fi
    
    # Check for panic or critical errors
    if grep -qi "panic\|fatal\|exception" "${SERIAL_LOG}"; then
        echo "❌ Kernel panic or fatal error detected!"
        break
    fi
    
    # Check if QEMU is still running
    if ! kill -0 ${QEMU_PID} 2>/dev/null; then
        echo "⚠️  QEMU process terminated"
        break
    fi
    
    sleep 1
    ELAPSED=$((ELAPSED + 1))
done

# Kill QEMU if still running
if kill -0 ${QEMU_PID} 2>/dev/null; then
    kill ${QEMU_PID} 2>/dev/null || true
    wait ${QEMU_PID} 2>/dev/null || true
fi

echo "==================================="
echo "Test Results"
echo "==================================="

# Display serial output
if [ -f "${SERIAL_LOG}" ]; then
    echo ""
    echo "📋 Serial Output:"
    echo "-----------------------------------"
    cat "${SERIAL_LOG}"
    echo "-----------------------------------"
    echo ""
fi

# Check test results
if [ ${FOUND} -eq 1 ]; then
    echo "✅ SUCCESS: Kernel booted successfully!"
    echo "   Found marker: ${SUCCESS_MARKER}"
    exit 0
elif [ ${ELAPSED} -ge ${TIMEOUT} ]; then
    echo "❌ TIMEOUT: Kernel did not boot within ${TIMEOUT}s"
    exit 1
else
    echo "❌ FAILURE: Kernel did not boot correctly"
    exit 1
fi
