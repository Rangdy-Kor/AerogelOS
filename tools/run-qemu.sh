#!/bin/bash

KERNEL="./kernel/target/x86_64-myos/debug/bootimage-myos-kernel.bin"

if [ ! -f "$KERNEL" ]; then
    echo "Error: Kernel not found. Run 'cargo bootimage' first."
    exit 1
fi

echo "Starting MyOS in QEMU..."
echo "Press Ctrl+A then X to exit QEMU"
echo ""

qemu-system-x86_64 \
    -drive format=raw,file=$KERNEL \
    -serial stdio \
    -display gtk \
    -m 512M \
    -cpu qemu64 \
    -no-reboot \
    -no-shutdown