; #[repr(C)]
; #[derive(Debug, Clone, Copy, Default)]
; pub struct PreservedRegisters {
;     pub r15: u64,
;     pub r14: u64,
;     pub r13: u64,
;     pub r12: u64,
;     pub rbp: u64,
;     pub rbx: u64,
; }

%macro pop_preserved_registers 0
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
%endmacro

%macro push_preserved_registers 0
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15
%endmacro

; #[repr(C)]
; #[derive(Debug, Clone, Copy, Default)]
; pub struct ScratchRegisters {
;     pub r11: u64,
;     pub r10: u64,
;     pub r9: u64,
;     pub r8: u64,
;     pub rsi: u64,
;     pub rdi: u64,
;     pub rdx: u64,
;     pub rcx: u64,
;     pub rax: u64,
; }
%macro pop_scratch_registers 0
    pop r11
    pop r10
    pop r9
    pop r8
    pop rsi
    pop rdi
    pop rdx
    pop rcx
    pop rax
%endmacro

%macro push_scratch_registers 0
    push rcx
    push rdx
    push rdi
    push rsi
    push r8
    push r9
    push r10
    push r11
%endmacro

