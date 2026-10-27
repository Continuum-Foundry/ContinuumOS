// Only these two functions may use FP state in CPL0. No arithmetic, no lazy state.
.section .text
.global task_save_fp
.type task_save_fp,@function
task_save_fp:
    clts
    fxsave64 [rdi]
    mov rax, cr0
    or rax, 8
    mov cr0, rax
    ret
.size task_save_fp, .-task_save_fp

.global task_resume
.type task_resume,@function
task_resume:
    cli
    cld
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
    mov rsp, [rip + {kernel_sp}]
    sub rsp, 160
    mov r8, rdx
    mov rdi, rsp
    mov ecx, 20
    rep movsq
    xor eax, eax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    clts
    fxrstor64 [r8]
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
.size task_resume, .-task_resume
