.section .text
.global diagnostic_guard
.type diagnostic_guard,@function
diagnostic_guard:
.global diagnostic_guard_site
diagnostic_guard_site:
    mov rax, [rdi]
    ud2
.size diagnostic_guard, .-diagnostic_guard
.global diagnostic_write
.type diagnostic_write,@function
diagnostic_write:
.global diagnostic_write_site
diagnostic_write_site:
    mov byte ptr [rdi], 0
    ud2
.size diagnostic_write, .-diagnostic_write
.global diagnostic_execute
.type diagnostic_execute,@function
diagnostic_execute:
    jmp rdi
.size diagnostic_execute, .-diagnostic_execute
.global diagnostic_double
.type diagnostic_double,@function
diagnostic_double:
    mov rsp, rdi
    mov rax, 0x400000000000
.global diagnostic_double_site
diagnostic_double_site:
    mov rax, [rax]
    ud2
.size diagnostic_double, .-diagnostic_double
.section .data
.balign 4096
.global diagnostic_nx_page
diagnostic_nx_page:
    .byte 0xc3
    .zero 4095
