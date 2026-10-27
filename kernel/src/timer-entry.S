.section .text
.macro returning name, vector
.global \name
.type \name,@function
\name:
    push rax
    push rcx
    push rdx
    push rbx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    cld
    mov rax, cr0
    push rax
    or rax, 8
    mov cr0, rax
    lea rbx, [rsp + 8]
    and rsp, -16
    mov edi, \vector
    mov rsi, [rbx + 128] // saved hardware CS after 15 integer registers
    mov rdx, rbx
    call {interrupt}
    mov rax, [rbx - 8]
    mov cr0, rax
    mov rsp, rbx
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax
    iretq
.size \name, .-\name
.endm
returning timer_vector, 64
returning spurious_vector, 255

// Exercise every GPR and DF across a real timer interrupt. Preserve the calling
// ABI and clear DF before returning to Rust. STI/HLT is the idle continuation.
.global timer_sentinels
.type timer_sentinels,@function
timer_sentinels:
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15
    mov rax, 1
    mov rcx, 2
    mov rdx, 3
    mov rbx, 4
    mov rbp, 5
    mov rsi, 6
    mov rdi, 7
    mov r8, 8
    mov r9, 9
    mov r10, 10
    mov r11, 11
    mov r12, 12
    mov r13, 13
    mov r14, 14
    mov r15, 15
    std
    sti
    hlt
    cli
    cmp rax, 1
    jne 2f
    cmp rcx, 2
    jne 2f
    cmp rdx, 3
    jne 2f
    cmp rbx, 4
    jne 2f
    cmp rbp, 5
    jne 2f
    cmp rsi, 6
    jne 2f
    cmp rdi, 7
    jne 2f
    cmp r8, 8
    jne 2f
    cmp r9, 9
    jne 2f
    cmp r10, 10
    jne 2f
    cmp r11, 11
    jne 2f
    cmp r12, 12
    jne 2f
    cmp r13, 13
    jne 2f
    cmp r14, 14
    jne 2f
    cmp r15, 15
    jne 2f
    pushfq
    pop rax
    test eax, 0x400
    jz 2f
    mov eax, 1
    jmp 3f
2:
    xor eax, eax
3:
    cld
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    ret
.size timer_sentinels, .-timer_sentinels
