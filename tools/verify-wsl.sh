#!/bin/bash

echo "=== MyOS Development Environment Verification (WSL2) ==="
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

check_command() {
    if command -v $1 &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1: $(command -v $1)"
        return 0
    else
        echo -e "${RED}✗${NC} $1: Not found"
        return 1
    fi
}

echo "Checking WSL version..."
if grep -qi microsoft /proc/version; then
    echo -e "${GREEN}✓${NC} Running on WSL"
else
    echo -e "${RED}✗${NC} Not running on WSL"
fi

echo ""
echo "Checking build tools..."
check_command gcc
check_command make
check_command nasm
check_command ld

echo ""
echo "Checking Rust..."
check_command rustc
check_command cargo
rustup show | grep x86_64-unknown-none && echo -e "${GREEN}✓${NC} x86_64-unknown-none target installed" || echo -e "${RED}✗${NC} Target not installed"

echo ""
echo "Checking QEMU..."
check_command qemu-system-x86_64

echo ""
echo "Checking .NET..."
check_command dotnet

echo ""
echo "=== Verification Complete ==="
EOF