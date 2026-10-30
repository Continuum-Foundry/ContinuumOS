//! Kernel-fatal diagnostics; the explicit P3-04 feature adds user containment.
//! CPU/ABI/unsafe review: exception and syscall-boundary runbooks.
use crate::{Local, record};
use core::{
    arch::{asm, global_asm},
    mem::size_of,
    ptr::addr_of_mut,
};

#[repr(C, align(4096))]
struct Stack([u8; 73728]);
static RUNTIME_STACK: Local<Stack> = Local(core::cell::UnsafeCell::new(Stack([0; 73728])));
static DOUBLE_STACK: Local<Stack> = Local(core::cell::UnsafeCell::new(Stack([0; 73728])));
static NMI_STACK: Local<Stack> = Local(core::cell::UnsafeCell::new(Stack([0; 73728])));
pub fn stacks() -> [u64; 3] {
    [
        RUNTIME_STACK.0.get() as u64,
        DOUBLE_STACK.0.get() as u64,
        NMI_STACK.0.get() as u64,
    ]
}

#[repr(C, packed)]
struct TablePointer {
    limit: u16,
    base: u64,
}
#[repr(C, packed)]
struct Tss {
    reserved0: u32,
    rsp: [u64; 3],
    reserved1: u64,
    ist: [u64; 7],
    reserved2: u64,
    reserved3: u16,
    iomap: u16,
}
static TSS: Local<Tss> = Local(core::cell::UnsafeCell::new(Tss {
    reserved0: 0,
    rsp: [0; 3],
    reserved1: 0,
    ist: [0; 7],
    reserved2: 0,
    reserved3: 0,
    iomap: size_of::<Tss>() as u16,
}));
static GDT: Local<[u64; 8]> = Local(core::cell::UnsafeCell::new([0; 8]));
#[repr(C)]
#[derive(Clone, Copy)]
struct Gate {
    low: u16,
    selector: u16,
    ist: u8,
    flags: u8,
    middle: u16,
    high: u32,
    zero: u32,
}
impl Gate {
    const EMPTY: Self = Self {
        low: 0,
        selector: 0,
        ist: 0,
        flags: 0,
        middle: 0,
        high: 0,
        zero: 0,
    };
    fn new(address: u64, ist: u8) -> Self {
        Self {
            low: address as u16,
            selector: 8,
            ist,
            flags: 0x8e,
            middle: (address >> 16) as u16,
            high: (address >> 32) as u32,
            zero: 0,
        }
    }
}
static IDT: Local<[Gate; 256]> = Local(core::cell::UnsafeCell::new([Gate::EMPTY; 256]));
const _: () =
    assert!(size_of::<Tss>() == 104 && size_of::<Gate>() == 16 && size_of::<TablePointer>() == 10);

// No Rust prologue on a hardware frame. Non-error vectors add a synthetic zero;
// all stubs add the vector. Hardware always pushes RIP, CS, RFLAGS, RSP, SS in
// 64-bit mode. RDI captures the normalized frame before RSP is aligned for CALL.
// All fifteen GPRs are saved. Kernel fatal paths never return; only the precise
// P3-04 copy-recovery path may update RIP and resume through IRETQ.
global_asm!(include_str!("exception-entry.S"),
    runtime = sym RUNTIME_STACK, run = sym run, dispatch = sym dispatch);
unsafe extern "C" {
    fn exception_start(mode: u64) -> !;
    fn exception_load_gdt(pointer: *const TablePointer);
    static exception_vectors: [u64; 32];
    fn exception_unexpected();
    #[cfg(feature = "exception-test")]
    fn diagnostic_invalid_opcode() -> !;
    #[cfg(feature = "exception-test")]
    fn diagnostic_page_fault() -> !;
    fn diagnostic_panic_report(rip: *mut u64, rsp: *mut u64);
}

pub fn mode(fixture: &str) -> Option<u64> {
    match fixture {
        "exception-invalid-opcode" => Some(1),
        "exception-page-fault" => Some(2),
        "panic" => Some(3),
        "exception-emergency" => Some(4),
        _ => None,
    }
}
pub fn start(mode: u64) -> ! {
    // All static BSS is covered by the validated kernel image reservation and
    // loader PT_LOAD mapping. Bootstrap memory remains retained after switching.
    unsafe { exception_start(mode) }
}

unsafe fn install() {
    // Single CPU, IF clear, one-time initialization; only raw writes until loaded.
    unsafe {
        let tss = TSS.0.get();
        addr_of_mut!((*tss).rsp).write_unaligned([RUNTIME_STACK.0.get() as u64 + 69632, 0, 0]);
        addr_of_mut!((*tss).ist).write_unaligned([
            DOUBLE_STACK.0.get() as u64 + 69632,
            NMI_STACK.0.get() as u64 + 69632,
            0,
            0,
            0,
            0,
            0,
        ]);
        let base = tss as u64;
        let limit = (size_of::<Tss>() - 1) as u64;
        // Null, ring-0 long code/data, TSS (two slots), null, user long code/data.
        GDT.0.get().write([
            0,
            0x00af9a000000ffff,
            0x00cf92000000ffff,
            limit | ((base & 0xffffff) << 16) | (0x89 << 40) | (((base >> 24) & 0xff) << 56),
            base >> 32,
            0,
            0x00affa000000ffff, // 0x33 user long code
            0x00cff2000000ffff, // 0x3b user data
        ]);
        for i in 0..256 {
            let entry = if i < 32 {
                exception_vectors[i]
            } else {
                exception_unexpected as *const () as u64
            };
            (*IDT.0.get())[i] = Gate::new(
                entry,
                if i == 8 {
                    1
                } else if i == 2 {
                    2
                } else {
                    0
                },
            );
        }
        (*IDT.0.get())[64] = Gate::new(crate::timer::timer_vector as *const () as u64, 0);
        (*IDT.0.get())[255] = Gate::new(crate::timer::spurious_vector as *const () as u64, 0);
        exception_load_gdt(&TablePointer {
            limit: (size_of::<[u64; 8]>() - 1) as u16,
            base: GDT.0.get() as u64,
        });
        asm!("ltr ax", in("ax") 24u16, options(nostack, preserves_flags));
        asm!("lidt [{}]", in(reg) &TablePointer { limit: (size_of::<[Gate; 256]>() - 1) as u16, base: IDT.0.get() as u64 }, options(nostack, preserves_flags));
        crate::out8(0x21, 0xff);
        crate::out8(0xa1, 0xff);
    }
}

extern "C" fn run(mode: u64) -> ! {
    unsafe {
        install();
    }
    record(
        "m1.exceptions_ready",
        "exceptions",
        "ok",
        format_args!(
            "idt_entries=256 runtime_base={:#x} double_base={:#x} nmi_base={:#x} stack_bytes=65536 double_ist=1 nmi_ist=2 guards=false if=false",
            RUNTIME_STACK.0.get() as u64 + 4096,
            DOUBLE_STACK.0.get() as u64 + 4096,
            NMI_STACK.0.get() as u64 + 4096
        ),
    );
    // Switch while already running on the final runtime stack. The two end
    // pages of every stack are excluded from the new mapping, including aliases.
    crate::paging::establish(mode).unwrap_or_else(|e| crate::reject(e));
    crate::runtime_memory::establish(mode).unwrap_or_else(|e| crate::reject(e));
    #[cfg(feature = "exception-test")]
    match mode {
        1 => unsafe { diagnostic_invalid_opcode() },
        2 => unsafe { diagnostic_page_fault() },
        3 => panic!("controlled P1-04 panic"),
        4 => {
            // Simulate an interrupted logger owning its gate, then take a real #UD.
            crate::LOG_BUSY.store(true, core::sync::atomic::Ordering::SeqCst);
            unsafe { diagnostic_invalid_opcode() }
        }
        _ => {}
    }
    let _ = mode;
    crate::timer::establish(mode).unwrap_or_else(|e| {
        record(
            "m1.platform_rejected",
            "timer",
            "rejected",
            format_args!("field={} reason={}", e.field, e.reason),
        );
        crate::failure_exit()
    });
    #[cfg(feature = "user-boundary-test")]
    if (32..=35).contains(&mode) {
        crate::user_entry::run(mode - 32).unwrap_or_else(|e| crate::reject(e));
    }
    #[cfg(feature = "scheduler-test")]
    if (36..=38).contains(&mode) {
        crate::user_entry::scheduling::run(mode == 36).unwrap_or_else(|e| crate::reject(e));
    }
    #[cfg(feature = "capability-test")]
    if mode == 39 {
        crate::user_entry::scheduling::run_objects(false).unwrap_or_else(|e| crate::reject(e));
    }
    #[cfg(feature = "channel-test")]
    if mode == 40 {
        crate::user_entry::scheduling::run_objects(true).unwrap_or_else(|e| crate::reject(e));
    }
    record(
        "m1.boot_complete",
        "boot",
        "ok",
        format_args!("reclaimed_bytes=0"),
    );
    record(
        "m1.ready",
        "ready",
        "ok",
        format_args!("baseline=boot-memory-timer reclaimed_bytes=0"),
    );
    // Explicit acceptance configuration only. The normal build has no injected
    // faults; its ordinary fixture continues into idle. Port 0xf4 requires the
    // declared emulator device and is not physical shutdown support.
    if mode == 100 {
        crate::test_exit(0x10);
    }
    #[cfg(feature = "emulator-test")]
    crate::test_exit(0x10);
    #[cfg(not(feature = "emulator-test"))]
    crate::timer::idle()
}

#[repr(C)]
pub(crate) struct Frame {
    pub vector: u64,
    pub error: u64,
    pub rip: u64,
    pub cs: u64,
    pub flags: u64,
    pub rsp: u64,
    pub ss: u64,
}
const _: () = assert!(size_of::<Frame>() == 56);
extern "C" fn dispatch(frame: *mut Frame, cr2: u64) {
    #[cfg(feature = "user-boundary-test")]
    if crate::user_entry::exception(unsafe { &mut *frame }, cr2) {
        return;
    }
    // CPU/stub-created frame on a retained stack; alignment is at least 8 bytes.
    // Copy scalars before reporting; there are no mutable frame references.
    let f = unsafe { frame.read() };
    crate::fatal(
        "m1.exception",
        "exceptions",
        format_args!(
            "vector={} rip={:#x} error={:#x} address={:#x} rsp={:#x} cs={:#x} flags={:#x} frame={:#x}",
            f.vector,
            f.rip,
            f.error,
            if f.vector == 14 { cr2 } else { 0 },
            f.rsp,
            f.cs,
            f.flags,
            frame as u64
        ),
    )
}

#[cfg(feature = "user-boundary-test")]
pub(crate) fn user_stack(top: u64) {
    unsafe {
        addr_of_mut!((*TSS.0.get()).rsp).write_unaligned([top, 0, 0]);
    }
}

pub fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let mut rip = 0;
    let mut rsp = 0;
    // Current reporting context, not an unwinder or fabricated panic caller RIP.
    unsafe {
        diagnostic_panic_report(&mut rip, &mut rsp);
    }
    let line = info.location().map_or(0, |l| l.line());
    crate::fatal(
        "m1.panic",
        "panic",
        format_args!(
            "vector=none rip={rip:#x} error=0x0 address=0x0 rsp={rsp:#x} location_line={line} reason=panic"
        ),
    )
}
