// Single CPU, no preemption: all globals are supervisor-only, RIP-relative.
// Save the surviving SysV continuation before installing an owned user root.
.section .text
.global user_enter
.type user_enter,@function
user_enter:
    cli
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    mov [rip + {continuation}], rsp
    mov rax, cr3
    mov [rip + {kernel_root}], rax
    mov cr3, rdi
    xor eax, eax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    push 0x3b
    push rdx
    push 0x202
    push 0x33
    push rsi
    mov edi, 0x7ffd0000
    xor eax, eax
    xor ebx, ebx
    xor ecx, ecx
    xor edx, edx
    xor ebp, ebp
    xor esi, esi
    xor r8d, r8d
    xor r9d, r9d
    xor r10d, r10d
    xor r11d, r11d
    xor r12d, r12d
    xor r13d, r13d
    xor r14d, r14d
    xor r15d, r15d
    iretq
.size user_enter, .-user_enter

.global user_leave
.type user_leave,@function
user_leave:
    cli
    cld
    mov rax, [rip + {kernel_root}]
    mov cr3, rax
    mov rsp, [rip + {continuation}]
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret
.size user_leave, .-user_leave

.global user_syscall
.type user_syscall,@function
user_syscall:
    // SYSCALL masks IF/DF/TF/AC/NT. It does NOT change RSP.
    mov [rip + {user_sp}], rsp
    mov rsp, [rip + {kernel_sp}]
    push rax
    mov rax, cr0
    or rax, 8
    mov cr0, rax
    pop rax
    push 0x3b
    push qword ptr [rip + {user_sp}]
    push r11
    push 0x33
    push rcx
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
    mov rdi, rsp
    // 5 hardware-shaped + 15 software slots = 160, top is 16 aligned.
    call {dispatch}
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
.size user_syscall, .-user_syscall

// Copy only after full pin + hardware validation, IF clear. The two precise
// fault sites recover via the normal exception entry; all other kernel #PF fatal.
.global user_copy
.type user_copy,@function
user_copy:
    test rdx, rdx
    jz 2f
1:
.global user_copy_read
user_copy_read:
    mov al, [rsi]
.global user_copy_write
user_copy_write:
    mov [rdi], al
    inc rsi
    inc rdi
    dec rdx
    jnz 1b
2:
    xor eax, eax
    ret
.global user_copy_failed
user_copy_failed:
    mov eax, 1
    ret
.size user_copy, .-user_copy
