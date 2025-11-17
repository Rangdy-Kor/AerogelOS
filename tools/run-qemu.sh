#!/bin/bash

KERNEL="./kernel/target/x86_64-myos/debug/bootimage-myos-kernel.bin"

if [ ! -f "$KERNEL" ]; then
    echo "Error: Kernel not found"
    exit 1
fi

KERNEL_ABS=$(readlink -f "$KERNEL")
KERNEL_WIN_PATH=$(wslpath -w "$KERNEL_ABS")

# bootimage는 이미 부트 가능한 디스크 이미지
/mnt/d/Program\ Files/qemu/qemu-system-x86_64.exe \
    -drive format=raw,file="$KERNEL_WIN_PATH" \
    -m 512M \
    -serial stdio