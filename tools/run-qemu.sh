#!/bin/bash

# KERNEL 변수 정의 (파일명이 bootimage-myos-kernel.bin 임을 가정)
KERNEL="./kernel/target/x86_64-myos/debug/bootimage-myos-kernel.bin"

if [ ! -f "$KERNEL" ]; then
    echo "Error: Kernel not found at $KERNEL. Please run 'make kernel' first."
    exit 1
fi

echo "Starting MyOS in Windows QEMU..."

# 1. Linux 절대 경로를 얻습니다.
KERNEL_ABS=$(readlink -f "$KERNEL")

# 2. Windows QEMU가 이해할 수 있도록 Windows 경로로 변환합니다. (wslpath 사용)
KERNEL_WIN_PATH=$(wslpath -w "$KERNEL_ABS")

# 3. QEMU 명령: -kernel 대신 -drive 옵션으로 변경
/mnt/d/Program\ Files/qemu/qemu-system-x86_64.exe \
    -drive format=raw,file="$KERNEL_WIN_PATH" \
    -m 512M
    # NOTE: 이전의 정렬 오류는 경로 문제 해결 후 사라질 가능성이 높습니다.