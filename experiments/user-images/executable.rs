#![no_std]
#![no_main]
mod runtime;
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
use runtime::*;
#[used]
static LABEL: [u8; 7] = *b"healthy";
#[cfg(user_a)]
const SEED: u64 = 0xa101;
#[cfg(user_b)]
const SEED: u64 = 0xb202;
#[cfg(service)]
const SEED: u64 = 0xc303;
#[used]
static mut DATA: u64 = SEED;
#[used]
static mut BSS: [u8; 8193] = [0; 8193];
global_asm!(r#"
.section .text.fixture_entry
.global fixture_entry
.type fixture_entry,@function
fixture_entry:
    // Verify pristine integer state before a compiler prologue can overwrite it.
    or rax, rbx
    or rax, rcx
    or rax, rdx
    or rax, rbp
    or rax, rsi
    or rax, r8
    or rax, r9
    or rax, r10
    or rax, r11
    or rax, r12
    or rax, r13
    or rax, r14
    or rax, r15
    test rax, rax
    jnz 8f
    cmp rdi, 0x7ffd0000
    jne 8f
    mov rax, rsp
    and eax, 15
    cmp eax, 8
    jne 8f
    mov ax, cs
    cmp ax, 0x33
    jne 8f
    mov ax, ss
    cmp ax, 0x3b
    jne 8f
    pushfq
    pop rax
    and eax, 0x3700
    cmp eax, 0x200
    jne 8f
    sub rsp, 8
    call {main}
    add rsp, 8
    ret
8:
    mov edi, 99
    mov eax, {exit}
    xor r8d, r8d
    xor r9d, r9d
    syscall
    ud2
.size fixture_entry, .-fixture_entry

.section .text
.global register_check
.type register_check,@function
register_check:
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15
    mov eax, {yield_id}
    mov edi, 7
    mov esi, 6
    mov r10d, 10
    xor r8d, r8d
    xor r9d, r9d
    mov ebx, 4
    mov ebp, 5
    mov r12d, 12
    mov r13d, 13
    mov r14d, 14
    mov r15d, 15
    mov ecx, 200000
1:
    pause
    loop 1b
    std
    syscall
    test rax, rax
    jnz 2f
    cmp rbx, 4
    jne 2f
    cmp rbp, 5
    jne 2f
    cmp rdi, 7
    jne 2f
    cmp rsi, 6
    jne 2f
    cmp r10, 10
    jne 2f
    test r8, r8
    jnz 2f
    test r9, r9
    jnz 2f
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
    test eax, 0x200
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
.size register_check, .-register_check
.global user_fp_forbidden
.type user_fp_forbidden,@function
user_fp_forbidden:
    pxor xmm0,xmm0
    ret
.size user_fp_forbidden, .-user_fp_forbidden
"#,main=sym main,exit=const PROCESS_EXIT,yield_id=const THREAD_YIELD);
unsafe extern "C" {
    fn register_check() -> u64;
    fn user_fp_forbidden();
}
fn pointers() {
    check(call(MEMORY_ALLOCATE, 0, 16) == (0, 0));
    check(call(MEMORY_FREE, 0, 0) == (0, 0));
    let p = allocate(8192);
    for (a, n) in [
        (0, 1),
        (0x80000000 - 1, 2),
        (u64::MAX - 3, 8),
        (0xffff800000000000, 1),
        (0x60000000, 1),
        (0x12345000, 1),
        (p as u64 + 8191, 2),
    ] {
        check(call(DIAGNOSTIC_LOG, a, n).0 == INVALIDARGUMENT);
    }
    for a in [
        0,
        0x7ffd0000,
        0x400000,
        0xffff800000000000,
        0x60000000,
        p as u64 + 8192 - 16,
        u64::MAX - 15,
    ] {
        check(call(ABI_QUERY, a, 32).0 == INVALIDARGUMENT);
    }
    for i in 0..8192 {
        unsafe {
            check(p.add(i).read_volatile() == 0);
            p.add(i).write_volatile(b'a');
        }
    }
    check(call(DIAGNOSTIC_LOG, p as u64 + 4095, 2) == (0, 2));
    check(call(ABI_QUERY, p as u64 + 4096 - 16, 32) == (0, 32));
    // A failed mixed-page output validation must leave the valid prefix intact.
    check(call(ABI_QUERY, p as u64 + 8192 - 16, 32).0 == INVALIDARGUMENT);
    for i in 8192 - 16..8192 {
        unsafe {
            check(p.add(i).read_volatile() == b'a');
        }
    }
    check(call(ABI_QUERY, p as u64, 31) == (BUFFERTOOSMALL, 32));
    check(call(DIAGNOSTIC_LOG, u64::MAX, 0) == (0, 0));
    check(call(DIAGNOSTIC_LOG, p as u64, 256) == (0, 256));
    check(call(DIAGNOSTIC_LOG, p as u64, 257).0 == INVALIDARGUMENT);
    check(call_version(THREAD_YIELD, 0, 0, 1, 0).0 == INCOMPATIBLEPROTOCOL);
    check(call_version(THREAD_YIELD, 0, 0, 0, 1).0 == UNSUPPORTED);
    check(call(u64::MAX, 0, 0) == (UNSUPPORTED, 0));
    check(call(MEMORY_ALLOCATE, u64::MAX, 16).0 == INVALIDARGUMENT);
    check(call(MEMORY_ALLOCATE, 1, 3).0 == INVALIDARGUMENT);
    check(call(MEMORY_FREE, p as u64 + 1, 0).0 == INVALIDARGUMENT);
    unsafe { release(p, 8192) };
    check(call(MEMORY_FREE, p as u64, 0).0 == INVALIDARGUMENT);
    log(b"pointers-passed");
}
extern "C" fn main(startup: *const u8) {
    for i in 0..64 {
        let expected = match i {
            0..=7 => b"CNTM3RT0"[i],
            12 => 64,
            33 => 16,
            _ => 0,
        };
        check(unsafe { startup.add(i).read_volatile() } == expected);
    }
    let case = unsafe { (0x70000000 as *const u64).read_volatile() };
    match case {
        0 => {
            let mut info = [0u8; 32];
            check(call(ABI_QUERY, info.as_mut_ptr() as u64, 32) == (0, 32));
            check(u64::from_le_bytes(info[8..16].try_into().unwrap()) == 4096);
            check(u64::from_le_bytes(info[16..24].try_into().unwrap()) == 101);
            check(u64::from_le_bytes(info[24..32].try_into().unwrap()) == 1001);
            info.fill(0xff); // fabricated payload identity cannot change attribution
            check(call(ABI_QUERY, info.as_mut_ptr() as u64, 32) == (0, 32));
            check(u64::from_le_bytes(info[16..24].try_into().unwrap()) == 101);
            check(unsafe { register_check() } == 1);
            let index = unsafe { (0x70000008 as *const u64).read_volatile() };
            if index == 0 {
                allocator_check();
                pointers();
            } else {
                let p = allocate(16);
                unsafe {
                    release(p, 16);
                }
            }
            log(b"healthy");
            // Normal Rust function return reaches the loader's process_exit stub.
        }
        1 => unsafe {
            asm!("mov rax, [0x12345000]",out("rax") _);
        },
        2 => unsafe {
            asm!("mov rax, [0x7ffdffff]",out("rax") _);
        },
        3 => unsafe {
            asm!("ud2", options(noreturn));
        },
        4 => unsafe {
            asm!("cli");
        },
        5 => unsafe {
            asm!("out dx, al",in("dx") 0xf4u16,in("al") 0u8);
        },
        6 => panic!("bounded user panic"),
        7 => {
            let _ = allocate(33 * 1024 * 1024);
        }
        8 => unsafe {
            asm!("mov rsp, 0x800000000000","mov eax, {yield_id}","xor r8d,r8d","xor r9d,r9d","syscall","ud2",yield_id=const THREAD_YIELD,options(noreturn));
        },
        9 => unsafe {
            asm!("mov rsp, 0x12345000","mov eax, {yield_id}","xor r8d,r8d","xor r9d,r9d","syscall","ud2",yield_id=const THREAD_YIELD,options(noreturn));
        },
        10 => unsafe {
            asm!("mov byte ptr [0x7ffd0000], 1");
        },
        11 => unsafe {
            user_fp_forbidden();
        }, // must #NM: no FP execution gate yet
        12 | 13 => {
            let _ = call(THREAD_YIELD, 0, 0);
            exit(99);
        }
        14 => unsafe {
            asm!("pushfq","or qword ptr [rsp], 0x40000","popfq","mov eax, {yield_id}","xor r8d,r8d","xor r9d,r9d","syscall","ud2",yield_id=const THREAD_YIELD,options(noreturn));
        },
        15 => unsafe {
            asm!("int 0x40");
        },
        16 => {
            for _ in 0..32 {
                let _ = allocate(1024 * 1024);
            }
            exit(99);
        }
        _ => exit(99),
    }
}
#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    log(b"panic");
    exit(1)
}
