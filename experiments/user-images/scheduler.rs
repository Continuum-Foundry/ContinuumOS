#![no_std]
#![no_main]
mod runtime;
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
use runtime::*;
#[repr(C, align(16))]
struct Fx([u8; 512]);
static mut EXPECTED: Fx = Fx([0; 512]);
static mut OBSERVED: Fx = Fx([0; 512]);
static mut SAVED_FLAGS: u64 = 0;
#[cfg(user_a)]
const SEED: u64 = 0xa101;
#[cfg(user_b)]
const SEED: u64 = 0xb202;
#[cfg(service)]
const SEED: u64 = 0xc303;
global_asm!(r#"
.section .text.fixture_entry
.global fixture_entry
.type fixture_entry,@function
fixture_entry:
    sub rsp, 8
    call {main}
    add rsp, 8
    ret
.size fixture_entry, .-fixture_entry
.section .text
.global spinner
.type spinner,@function
spinner:
    mov eax, 1
    mov ecx, 2
    mov edx, 3
    mov ebx, 4
    mov ebp, 5
    mov esi, 6
    mov edi, 7
    mov r8d, 8
    mov r9d, 9
    mov r10d, 10
    mov r11d, 11
    mov r12d, 12
    mov r13d, 13
    mov r14d, 14
    mov r15d, 15
    push 0xed7
    popfq
1:
    pushfq
    pop qword ptr [rip + {flags}]
    cmp qword ptr [rip + {flags}], 0xed7
    jne 9f
    cmp rax, 1
    jne 9f
    cmp rcx, 2
    jne 9f
    cmp rdx, 3
    jne 9f
    cmp rbx, 4
    jne 9f
    cmp rbp, 5
    jne 9f
    cmp rsi, 6
    jne 9f
    cmp rdi, 7
    jne 9f
    cmp r8, 8
    jne 9f
    cmp r9, 9
    jne 9f
    cmp r10, 10
    jne 9f
    cmp r11, 11
    jne 9f
    cmp r12, 12
    jne 9f
    cmp r13, 13
    jne 9f
    cmp r14, 14
    jne 9f
    cmp r15, 15
    jne 9f
    fxsave64 [rip + {observed}]
    xor eax,eax
2:
    // Full FP image is checked by the Rust control; hogs check all 16 XMM
    // lanes and eight x87 payloads without changing their hardware state.
    lea rcx, [rip + {expected}]
    mov cl, [rcx + rax]
    lea rax, [rax + 1]
    push rdx
    lea rdx, [rip + {observed}]
    cmp cl, [rdx + rax - 1]
    pop rdx
    jne 8f
    cmp eax, 28
    jne 3f
    mov eax,32
3:
    cmp eax,416
    jb 2b
    inc qword ptr [0x70000008]
    mov eax,1
    mov ecx,2
    push 0xed7
    popfq
    jmp 1b
8:
    mov edi,94
    jmp 10f
9:
    mov edi,99
10:
    cld
    mov eax,{exit}
    xor r8d,r8d
    xor r9d,r9d
    syscall
    ud2
.size spinner, .-spinner
"#, main=sym main, flags=sym SAVED_FLAGS, expected=sym EXPECTED, observed=sym OBSERVED,
exit=const PROCESS_EXIT);
unsafe extern "C" {
    fn spinner() -> !;
}
fn check(v: bool) {
    if !v {
        exit(99);
    }
}
fn snapshot() {
    unsafe {
        asm!("fxsave64 [{}]",in(reg) core::ptr::addr_of_mut!(OBSERVED));
    }
}
fn compare() {
    snapshot();
    unsafe {
        let e = core::ptr::addr_of!(EXPECTED.0) as *const u8;
        let o = core::ptr::addr_of!(OBSERVED.0) as *const u8;
        for i in (0..28).chain(32..416) {
            if e.add(i).read_volatile() != o.add(i).read_volatile() {
                exit(1000 + i as u64);
            }
        }
    }
}
extern "C" fn main() {
    snapshot();
    unsafe {
        let b = &*core::ptr::addr_of!(OBSERVED.0);
        if !(b[0..2] == [0x7f, 3] && b[2..24].iter().all(|v| *v == 0)) {
            exit(90);
        }
        if !(b[24..28] == [0x80, 0x1f, 0, 0] && b[32..416].iter().all(|v| *v == 0)) {
            exit(91);
        }
        let mode = (0x70000000 as *const u64).read_volatile();
        // Each independently linked address space has different sentinels at
        // identical VAs. Full x87 register file, XMM0-15, flags and rounding.
        let b = &mut *core::ptr::addr_of_mut!(EXPECTED.0);
        b[0..2].copy_from_slice(&(0x37fu16 | ((SEED as u16 & 3) << 10)).to_le_bytes());
        b[2] = 1;
        b[4] = 255;
        b[24..28].copy_from_slice(&(0x1f81u32 | ((SEED as u32 & 3) << 13)).to_le_bytes());
        for i in 0..8 {
            b[32 + i * 16..40 + i * 16].copy_from_slice(
                &(0x8000000000000000u64 | SEED | ((i as u64) << 32)).to_le_bytes(),
            );
            b[40 + i * 16..42 + i * 16].copy_from_slice(&0x3fffu16.to_le_bytes());
        }
        for i in 0..32 {
            b[160 + i * 8..168 + i * 8].copy_from_slice(&(SEED + ((i as u64) << 32)).to_le_bytes());
        }
        asm!("fxrstor64 [{}]",in(reg) core::ptr::addr_of!(EXPECTED));
        compare();
        match mode {
            0 => spinner(),
            2 => {
                let cw = 0x37eu16;
                asm!("fninit","fldcw [{}]","fldz","fldz","fdivp st(1), st(0)","fwait",in(reg)&cw);
                exit(99);
            }
            3 => {
                let mx = 0x1f00u32;
                asm!("ldmxcsr [{}]","pxor xmm0,xmm0","divss xmm0,xmm0",in(reg)&mx);
                exit(99);
            }
            4 => {
                asm!("vxorps ymm0,ymm0,ymm0");
                exit(99);
            }
            5 => {
                asm!("cli");
                exit(99);
            }
            7 => exit(0),
            10 => {
                asm!(
                    "mov rsp, 0x800000000000",
                    "2:",
                    "pause",
                    "jmp 2b",
                    options(noreturn)
                );
            }
            _ => {}
        }
        for _ in 0..8 {
            // Real pinned output copy interrupted with a live pin. Kernel IRQ
            // defers; the checked syscall epilogue is the rescheduling point.
            let mut out = [0u8; 32];
            let (s, n) = call(ABI_QUERY, out.as_mut_ptr() as u64, 32);
            check(s == 0 && n == 32);
            check(u64::from_le_bytes(out[16..24].try_into().unwrap()) == 303);
            compare();
            let (s, _) = call(u64::MAX, 0, 0);
            check(s == UNSUPPORTED);
            if mode == 6 {
                let (s, p) = call(MEMORY_ALLOCATE, 4096, 16);
                check(s == 0);
                (p as *mut u64).write_volatile(SEED);
                let (s, _) = call(MEMORY_FREE, p, 0);
                check(s == 0);
            }
            let (s, _) = call(THREAD_YIELD, 0, 0);
            check(s == 0);
            compare();
            (0x70000008 as *mut u64).write_volatile((0x70000008 as *const u64).read_volatile() + 1);
        }
    }
}
#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    exit(98)
}
