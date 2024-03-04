global trampoline_size
global load_trampoline
global is_ap_ready
global prepare_ap_launch
global trampoline_size
extern _x86_64_ap_entrypoint

section .data

%include "defs.inc"

%define trampoline_sz trampoline_end - trampoline_begin
trampoline_begin: incbin "../target/trampoline.bin"
trampoline_end:

section .text

%define TRAMPOLINE_ADDR 0x1000

load_trampoline:
    mov rsi, trampoline_begin
    mov rdi, TRAMPOLINE_ADDR
    mov rcx, trampoline_sz
    rep movsb

    mov rax, trampoline_sz
    ret

prepare_ap_launch:
    mov qword [PAGE_TABLE], rdi
    mov qword [STACK_TOP], rsi
    mov qword [AP_ID], rdx
    mov qword [READY_FLAG], 0

    mov qword [ENTRYPOINT], _x86_64_ap_entrypoint

    ret

is_ap_ready:
    mov al, byte [READY_FLAG]
    ret
