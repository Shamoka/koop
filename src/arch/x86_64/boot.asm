global start
global pml4_table
extern long_mode_start

section .rodata
gdt64:
	dq 0
.code: equ $ - gdt64
	dq (1 << 43) | ( 1 << 44 ) | (1 << 47) | (1 << 53)
.pointer:
	dw $ - gdt64 - 1
	dq gdt64



section .text
bits 32
start:
	mov esp, stack_top
	mov edi, ebx

	call check_multiboot
	call check_cpuid
	call check_long_mode

	call setup_page_tables
	call enable_paging

	lgdt [gdt64.pointer]

	jmp gdt64.code:long_mode_start


check_multiboot:
	cmp eax, 0x36d76289
	jne .no_multiboot
	ret
.no_multiboot:
	mov al, "0"
	jmp error

check_cpuid:
	pushfd
	pop eax
	mov ecx, eax
	xor eax, 1 << 21
	push eax
	popfd
	pushfd
	pop eax
	push ecx
	popfd
	cmp eax, ecx
	je .no_cpuid
	ret
.no_cpuid:
	mov al, "1"
	jmp error

check_long_mode:
	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001
	jb .no_long_mode
	mov eax, 0x80000001
	cpuid
	test edx, 1 << 29
	jz .no_long_mode
	ret
.no_long_mode:
	mov al, "2"
	jmp error

setup_page_tables:
	mov eax, pdp_table
	or eax, 0b11
	mov [pml4_table], eax
	mov eax, pd_table
	or eax, 0b11
	mov [pdp_table], eax
	mov ecx, 0
.map_pd_table:
	mov eax, 0x200000
	mul ecx
	or eax, 0b10000011
	mov [pd_table + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .map_pd_table
	ret

enable_paging:
	mov eax, pml4_table
	mov cr3, eax
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax
	ret

error:
	mov dword [0xb8000], 0x4f524f45
	mov dword [0xb8004], 0x4f3a4f52
	mov dword [0xb8008], 0x4f204f20
	mov byte [0xb800a], al
	hlt



section .bss
align 4096
pml4_table:
	resb 4096
pdp_table:
	resb 4096
pd_table:
	resb 4096
stack_bottom:
	resb 4096 * 8
stack_top:
