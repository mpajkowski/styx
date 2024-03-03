%include "defs.inc"

org 0x1000
bits 16

cli
cld

xor ax, ax
mov ds, ax

jmp 0x0:prepare_long

prepare_long:
    mov es, ax
    mov ss, ax
    mov sp, ax

    lgdt [gdt_ptr]

    mov eax, cr4
    or eax, 1 << 5 | 1 << 7
    mov cr4, eax

    mov ecx, 0xc0000080
    rdmsr
    or eax, 1 << 11 | 1 << 8
    wrmsr

    ; set page table read from BSP
    mov edx, dword [PAGE_TABLE]
    mov cr3, edx

    mov ebx, cr0
    or ebx, 1 << 31 | 1 << 16 | 1
    mov cr0, ebx

    jmp 0x08:.longmode

bits 64

.longmode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; setup stack
    mov rsp, qword [STACK_TOP]

    ; mark as ready
    mov qword [READY_FLAG], 1

    mov rdi, qword [AP_ID]
    mov rsi, qword [STACK_TOP]

    mov rbx, qword [ENTRYPOINT]
    call rbx

align 16
gdt_ptr:
    dw .gdt_end - .gdt_start - 1  ; GDT size
    dq .gdt_start                 ; GDT start

align 16
.gdt_start:

; Null descriptor
.null_descriptor:
    dw 0x0000           ; Limit
    dw 0x0000           ; Base (low 16 bits)
    db 0x00             ; Base (mid 8 bits)
    db 00000000b        ; Access
    db 00000000b        ; Granularity
    db 0x00             ; Base (high 8 bits)

; 64 bit mode kernel
.kernel_code_64:
    dw 0x0000           ; Limit
    dw 0x0000           ; Base (low 16 bits)
    db 0x00             ; Base (mid 8 bits)
    db 10011010b        ; Access
    db 00100000b        ; Granularity
    db 0x00             ; Base (high 8 bits)

.kernel_data:
    dw 0x0000           ; Limit
    dw 0x0000           ; Base (low 16 bits)
    db 0x00             ; Base (mid 8 bits)
    db 10010010b        ; Access
    db 00000000b        ; Granularity
    db 0x00             ; Base (high 8 bits)

.gdt_end:

