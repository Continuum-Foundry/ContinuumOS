.section .text
.global exception_start
.type exception_start,@function
exception_start:
    cli
    lea rsp, [rip + {runtime} + 69632]
    xor ebp, ebp
    call {run}
    ud2
.size exception_start, .-exception_start

.global exception_load_gdt
.type exception_load_gdt,@function
exception_load_gdt:
    lgdt [rdi]
    push 8
    lea rax, [rip + 1f]
    push rax
    retfq
1:
    mov ax, 16
    mov ds, ax
    mov es, ax
    mov ss, ax
    ret
.size exception_load_gdt, .-exception_load_gdt

.type exception_0,@function
exception_0:
    push 0
    push 0
    jmp exception_common
.size exception_0, .-exception_0

.type exception_1,@function
exception_1:
    push 0
    push 1
    jmp exception_common
.size exception_1, .-exception_1

.type exception_2,@function
exception_2:
    push 0
    push 2
    jmp exception_common
.size exception_2, .-exception_2

.type exception_3,@function
exception_3:
    push 0
    push 3
    jmp exception_common
.size exception_3, .-exception_3

.type exception_4,@function
exception_4:
    push 0
    push 4
    jmp exception_common
.size exception_4, .-exception_4

.type exception_5,@function
exception_5:
    push 0
    push 5
    jmp exception_common
.size exception_5, .-exception_5

.type exception_6,@function
exception_6:
    push 0
    push 6
    jmp exception_common
.size exception_6, .-exception_6

.type exception_7,@function
exception_7:
    push 0
    push 7
    jmp exception_common
.size exception_7, .-exception_7

.type exception_8,@function
exception_8:
    push 8
    jmp exception_common
.size exception_8, .-exception_8

.type exception_9,@function
exception_9:
    push 0
    push 9
    jmp exception_common
.size exception_9, .-exception_9

.type exception_10,@function
exception_10:
    push 10
    jmp exception_common
.size exception_10, .-exception_10

.type exception_11,@function
exception_11:
    push 11
    jmp exception_common
.size exception_11, .-exception_11

.type exception_12,@function
exception_12:
    push 12
    jmp exception_common
.size exception_12, .-exception_12

.type exception_13,@function
exception_13:
    push 13
    jmp exception_common
.size exception_13, .-exception_13

.type exception_14,@function
exception_14:
    push 14
    jmp exception_common
.size exception_14, .-exception_14

.type exception_15,@function
exception_15:
    push 0
    push 15
    jmp exception_common
.size exception_15, .-exception_15

.type exception_16,@function
exception_16:
    push 0
    push 16
    jmp exception_common
.size exception_16, .-exception_16

.type exception_17,@function
exception_17:
    push 17
    jmp exception_common
.size exception_17, .-exception_17

.type exception_18,@function
exception_18:
    push 0
    push 18
    jmp exception_common
.size exception_18, .-exception_18

.type exception_19,@function
exception_19:
    push 0
    push 19
    jmp exception_common
.size exception_19, .-exception_19

.type exception_20,@function
exception_20:
    push 0
    push 20
    jmp exception_common
.size exception_20, .-exception_20

.type exception_21,@function
exception_21:
    push 21
    jmp exception_common
.size exception_21, .-exception_21

.type exception_22,@function
exception_22:
    push 0
    push 22
    jmp exception_common
.size exception_22, .-exception_22

.type exception_23,@function
exception_23:
    push 0
    push 23
    jmp exception_common
.size exception_23, .-exception_23

.type exception_24,@function
exception_24:
    push 0
    push 24
    jmp exception_common
.size exception_24, .-exception_24

.type exception_25,@function
exception_25:
    push 0
    push 25
    jmp exception_common
.size exception_25, .-exception_25

.type exception_26,@function
exception_26:
    push 0
    push 26
    jmp exception_common
.size exception_26, .-exception_26

.type exception_27,@function
exception_27:
    push 0
    push 27
    jmp exception_common
.size exception_27, .-exception_27

.type exception_28,@function
exception_28:
    push 0
    push 28
    jmp exception_common
.size exception_28, .-exception_28

.type exception_29,@function
exception_29:
    push 29
    jmp exception_common
.size exception_29, .-exception_29

.type exception_30,@function
exception_30:
    push 30
    jmp exception_common
.size exception_30, .-exception_30

.type exception_31,@function
exception_31:
    push 0
    push 31
    jmp exception_common
.size exception_31, .-exception_31

.global exception_unexpected
.type exception_unexpected,@function
exception_unexpected:
    push 0
    push 255
    jmp exception_common
.size exception_unexpected, .-exception_unexpected

.type exception_common,@function
exception_common:
    cli
    cld
    mov rdi, rsp
    mov rsi, cr2
    and rsp, -16
    call {dispatch}
    ud2
.size exception_common, .-exception_common

.global diagnostic_invalid_opcode
.type diagnostic_invalid_opcode,@function
diagnostic_invalid_opcode:
    ud2
.size diagnostic_invalid_opcode, .-diagnostic_invalid_opcode

.global diagnostic_page_fault
.type diagnostic_page_fault,@function
diagnostic_page_fault:
    mov rax, 0x400000000000
.global diagnostic_page_fault_site
diagnostic_page_fault_site:
    mov rax, [rax]
    ud2
.size diagnostic_page_fault, .-diagnostic_page_fault

.global diagnostic_panic_report
.type diagnostic_panic_report,@function
diagnostic_panic_report:
    lea rax, [rip + diagnostic_panic_report]
    mov [rdi], rax
    lea rax, [rsp + 8]
    mov [rsi], rax
    ret
.size diagnostic_panic_report, .-diagnostic_panic_report

.section .rodata
.balign 8
.global exception_vectors
exception_vectors:
    .quad exception_0
    .quad exception_1
    .quad exception_2
    .quad exception_3
    .quad exception_4
    .quad exception_5
    .quad exception_6
    .quad exception_7
    .quad exception_8
    .quad exception_9
    .quad exception_10
    .quad exception_11
    .quad exception_12
    .quad exception_13
    .quad exception_14
    .quad exception_15
    .quad exception_16
    .quad exception_17
    .quad exception_18
    .quad exception_19
    .quad exception_20
    .quad exception_21
    .quad exception_22
    .quad exception_23
    .quad exception_24
    .quad exception_25
    .quad exception_26
    .quad exception_27
    .quad exception_28
    .quad exception_29
    .quad exception_30
    .quad exception_31
