; MyOS Bootloader
bits 16
org 0x7C00

start:
	; Clear screen
	mov ax, 0x0003
	int 0x10

	; Print message
	mov si, msg
	call print_string

	; Halt
	cli
	hlt

print_string:
	lodsb
	or al, al
	jz .done
	mov ah, 0x0E
	int 0x10
	jmp print_string
.done:
	ret

msg db 'MyOS Bootloader - Running on Windows WSL2!', 13, 10, 0

; Boot signature
times 510-($-$$) db 0
dw 0xAA55