bits 64

%include "registers.inc"

extern generic_irq_handler

%macro make_irq_handler 2
[global irq_handler_%1]
irq_handler_%1:
%if %2 == 0
    push 0
%endif
    lock xchg [rsp], rax

    push_scratch_registers
    push_preserved_registers
    push rax

    ; prepare call for generic_irq_handler(idx: usize, stack *mut InterruptStack)
    mov rdi, %1
    mov rsi, rsp

    ; call the handler
    call generic_irq_handler

    ; pop error code
    add rsp, 8

    pop_preserved_registers
    pop_scratch_registers

    ; return from interrupt
    iretq
%endmacro

%macro make_irq_handler_error 1
    make_irq_handler %1, 1
%endmacro

%macro make_irq_handler_no_error 1
    make_irq_handler %1, 0
%endmacro

make_irq_handler_no_error 0
make_irq_handler_no_error 1
make_irq_handler_no_error 2
make_irq_handler_no_error 3
make_irq_handler_no_error 4
make_irq_handler_no_error 5
make_irq_handler_no_error 6
make_irq_handler_no_error 7
make_irq_handler_error    8
; reserved                9
make_irq_handler_error 10
make_irq_handler_error 11
make_irq_handler_error 12
make_irq_handler_error 13
make_irq_handler_error 14
; reserved                15
make_irq_handler_no_error 16
make_irq_handler_error    17
make_irq_handler_no_error 18
make_irq_handler_no_error 19
make_irq_handler_no_error 20
; reserved                21
; reserved                22
; reserved                23
; reserved                24
; reserved                25
; reserved                26
; reserved                27
; reserved                28
; reserved                29
make_irq_handler_error    30
; reserved                31

%assign i 32
%rep 224
    make_irq_handler_no_error i
%assign i i + 1
%endrep

section .rodata

global irq_handler_table
irq_handler_table:
    dq irq_handler_0
    dq irq_handler_1
    dq irq_handler_2
    dq irq_handler_3
    dq irq_handler_4
    dq irq_handler_5
    dq irq_handler_6
    dq irq_handler_7
    dq irq_handler_8
    dq 0 ; reserved 9
    dq irq_handler_10
    dq irq_handler_11
    dq irq_handler_12
    dq irq_handler_13
    dq irq_handler_14
    dq 0 ; reserved 15
    dq irq_handler_16
    dq irq_handler_17
    dq irq_handler_18
    dq irq_handler_19
    dq irq_handler_20
    dq 0 ; reserved 21
    dq 0 ; reserved 22
    dq 0 ; reserved 23
    dq 0 ; reserved 24
    dq 0 ; reserved 25
    dq 0 ; reserved 26
    dq 0 ; reserved 27
    dq 0 ; reserved 28
    dq 0 ; reserved 29
    dq irq_handler_30
    dq 0 ; reserved 31
%assign i 32
%rep 224
    dq irq_handler_%+i
%assign i i + 1
%endrep
