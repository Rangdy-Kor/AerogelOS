.PHONY: all kernel bootloader run clean

all: kernel bootloader

bootloader:
	cd bootloader && make

kernel:
	cd kernel && cargo bootimage

run: kernel
	./tools/run-qemu.sh

clean:
	cd bootloader && make clean
	cd kernel && cargo clean

help:
	@echo "MyOS Build System (Windows WSL2)"
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build everything"
	@echo "  kernel     - Build kernel only"
	@echo "  bootloader - Build bootloader only"
	@echo "  run        - Build and run in QEMU"
	@echo "  clean      - Clean all build artifacts"