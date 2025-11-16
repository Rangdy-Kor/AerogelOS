#!/bin/bash

KERNEL="./kernel/target/x86_64-myos/debug/bootimage-myos-kernel.bin"

if [ ! -f "$KERNEL" ]; then
    echo "Error: Kernel not found."
    exit 1
fi

echo "Starting MyOS in Windows QEMU..."

# 절대 경로로 변환
KERNEL_ABS=$(readlink -f "$KERNEL")

/mnt/d/Program\ Files/qemu/qemu-system-x86_64.exe \
    -drive format=raw,file="$KERNEL_ABS" \
    -m 512M