global long_mode_start

extern koop
extern pml4_table

section .text
bits 64
long_mode_start:
	mov ax, 0
	mov ss, ax
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax

	mov rax, cr3
	or rax, 0b11
	mov [pml4_table + 511 * 8], rax

	call koop
