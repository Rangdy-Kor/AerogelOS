ASM = nasm
ASMFLAGS = -f bin

all: bootloader.bin

bootloader.bin: src/boot.asm
	$(ASM) $(ASMFLAGS) src/boot.asm -o bootloader.bin

clean:
	rm -f bootloader.bin

.PHONY: all clean