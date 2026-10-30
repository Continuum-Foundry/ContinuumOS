//! P3-04 single-process supervisor. No user references/borrows survive a dispatch.
use crate::{Local, record, runtime_memory};
use continuum_kernel::{
    Result, VirtAddr, error,
    memory::PAGE,
    owned_memory::{Access, AllocationId, SpaceId},
    user_boundary::*,
    user_image::*,
};
use core::{
    arch::{asm, global_asm},
    cell::UnsafeCell,
};
include!(env!("CONTINUUM_TRUSTED_IMAGES"));
#[allow(dead_code)]
mod ids {
    include!(env!("CONTINUUM_RUNTIME_IDS"));
}
use ids::*;
#[cfg(feature = "capability-test")]
#[path = "capability_guest.rs"]
pub(crate) mod capabilities;
#[path = "user_fixture.rs"]
mod fixture;
#[cfg(feature = "scheduler-test")]
#[path = "scheduling.rs"]
pub(crate) mod scheduling;
#[cfg(feature = "wait-test")]
#[path = "waiting_guest.rs"]
pub(crate) mod waiting;
static CONTINUATION: Local<u64> = Local(UnsafeCell::new(0));
static KERNEL_ROOT: Local<u64> = Local(UnsafeCell::new(0));
static KERNEL_SP: Local<u64> = Local(UnsafeCell::new(0));
static USER_SP: Local<u64> = Local(UnsafeCell::new(0));
static LAUNCHER: Local<Launcher<16>> = Local(UnsafeCell::new(Launcher::EMPTY));
#[derive(Clone, Copy)]
struct Current {
    id: ProcessId,
    space: SpaceId,
    context: Context,
    case: u64,
}
static CURRENT: Local<Option<Current>> = Local(UnsafeCell::new(None));
static RESULT: Local<u64> = Local(UnsafeCell::new(u64::MAX));
static COPY: Local<Option<(u64, u64)>> = Local(UnsafeCell::new(None));
// Provisional private allocation records; addresses are NOT authority/handles.
#[derive(Clone, Copy)]
struct Heap {
    base: u64,
    bytes: u64,
    id: AllocationId,
}
static HEAP: Local<[Option<Heap>; 32]> = Local(UnsafeCell::new([None; 32]));
static CALLS: Local<u64> = Local(UnsafeCell::new(0));
static PEAK: Local<u64> = Local(UnsafeCell::new(0));
global_asm!(include_str!("user-entry.S"), continuation=sym CONTINUATION,
    kernel_root=sym KERNEL_ROOT, kernel_sp=sym KERNEL_SP, user_sp=sym USER_SP,
    dispatch=sym dispatch);
unsafe extern "C" {
    fn user_enter(root: u64, ip: u64, sp: u64);
    fn user_leave() -> !;
    fn user_syscall();
    fn user_copy(dst: u64, src: u64, bytes: u64) -> u64;
    fn user_copy_read();
    fn user_copy_write();
    fn user_copy_failed();
}
pub(crate) fn kernel_root() -> u64 {
    unsafe { KERNEL_ROOT.0.get().read() }
}
fn need(ok: bool) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(error("user_boundary", "self_check"))
    }
}
fn current() -> Current {
    unsafe { CURRENT.0.get().read() }.expect("trusted current")
}
unsafe fn wrmsr(id: u32, value: u64) {
    unsafe {
        asm!("wrmsr", in("ecx") id, in("eax") value as u32, in("edx") (value >> 32) as u32, options(nostack));
    }
}
fn install() -> Result<()> {
    let cpu = core::arch::x86_64::__cpuid(0x80000001);
    need(cpu.edx & (1 << 11) != 0)?;
    unsafe {
        let lo: u32;
        let hi: u32;
        asm!("rdmsr", in("ecx") 0xc0000080u32, out("eax") lo, out("edx") hi, options(nostack));
        wrmsr(0xc0000080, ((hi as u64) << 32) | lo as u64 | 1);
        wrmsr(0xc0000081, 8u64 << 32); // kernel CS/SS; SYSRET unused
        wrmsr(0xc0000082, user_syscall as *const () as u64);
        wrmsr(0xc0000084, 0x200 | 0x400 | 0x100 | 0x40000 | 0x4000);
        for msr in [0xc0000100, 0xc0000101, 0xc0000102] {
            wrmsr(msr, 0);
        }
        // Enable FX state support but keep TS set in kernel Rust and the
        // integer-only fixture; eager scheduler resume clears TS for its owner.
        let features = core::arch::x86_64::__cpuid(1).edx;
        need(features & (7 << 24) == 7 << 24)?;
        let cr0: u64;
        let cr4: u64;
        asm!("mov {}, cr0",out(reg) cr0);
        asm!("mov {}, cr4",out(reg) cr4);
        asm!("mov cr4, {}",in(reg) cr4 | (3 << 9));
        asm!("mov cr0, {}",in(reg) (cr0 & !4) | 10);
        asm!("xor eax, eax", "mov ds, ax", "mov es, ax", "mov fs, ax", "mov gs, ax", out("eax") _, options(nostack));
    }
    Ok(())
}
// Exact assembly layout, all integer GPRs plus an IRETQ frame. No wire casts.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Frame {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rbp: u64,
    rbx: u64,
    rdx: u64,
    rcx: u64,
    rax: u64,
    ip: u64,
    cs: u64,
    flags: u64,
    sp: u64,
    ss: u64,
}
const _: () =
    assert!(core::mem::size_of::<Frame>() == 160 && core::mem::offset_of!(Frame, ip) == 120);
fn finish(reason: &'static str, result: u64, vector: u64, ip: u64, address: u64) -> ! {
    #[cfg(feature = "scheduler-test")]
    if scheduling::active() {
        scheduling::finish(reason, result, vector);
    }
    let c = current();
    record(
        "m1.user_end",
        "user-boundary",
        "ok",
        format_args!(
            "case={} slot={} generation={} principal={} authority_session={} reason={reason} result={result} vector={vector} ip={ip:#x} address={address:#x}",
            c.case, c.id.slot, c.id.generation, c.context.principal, c.context.session
        ),
    );
    unsafe {
        RESULT.0.get().write(result);
        user_leave()
    }
}
pub(crate) fn exception(f: &mut crate::exceptions::Frame, address: u64) -> bool {
    // Recovery is allowed only at the two audited copy instructions, with an
    // active bounded range, CPL0 #PF and CR2 inside that exact user range.
    if f.vector == 14 && f.cs == 8 {
        if let Some((start, end)) = unsafe { COPY.0.get().read() } {
            if (start..end).contains(&address)
                && [
                    user_copy_read as *const () as u64,
                    user_copy_write as *const () as u64,
                ]
                .contains(&f.rip)
            {
                f.rip = user_copy_failed as *const () as u64;
                return true;
            }
        }
    }
    if f.cs == USER_CS
        && f.ss == USER_SS
        && unsafe { CURRENT.0.get().read().is_some() }
        && [0, 1, 3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 16, 17, 19].contains(&f.vector)
    {
        let address = if f.vector == 14 { address } else { 0 };
        record(
            "m1.user_fault",
            "user-boundary",
            "ok",
            format_args!(
                "case={} vector={} error={:#x} cs={:#x} ss={:#x} flags={:#x} sp={:#x} ip={:#x} address={address:#x}",
                current().case,
                f.vector,
                f.error,
                f.cs,
                f.ss,
                f.flags,
                f.rsp,
                f.rip
            ),
        );
        finish(
            "fault",
            0x100 + f.vector,
            f.vector,
            f.rip,
            if f.vector == 14 { address } else { 0 },
        );
    }
    false // NMI, double fault, machine check and every other kernel fault fatal
}
fn copy(address: u64, bytes: &mut [u8], output: bool) -> core::result::Result<(), u64> {
    if bytes.is_empty() {
        return Ok(());
    }
    let c = current();
    runtime_memory::with(|r, _| {
        let pin = r
            .pin(c.space, address, bytes.len() as u64, output)
            .map_err(|_| INVALIDARGUMENT)?;
        // The pin blocks map/protect/free/destroy, and IF stays clear. Nothing
        // in an interrupt, NMI or another CPU can mutate the mapping lifetime.
        unsafe {
            COPY.0
                .get()
                .write(Some((address, address + bytes.len() as u64)));
        }
        let kernel = bytes.as_mut_ptr() as u64;
        #[cfg(feature = "scheduler-test")]
        {
            assert!(r.inventory().active_pins > 0);
            (fixture::hooks().copy_observer)(true);
        }
        let failed = unsafe {
            if output {
                user_copy(address, kernel, bytes.len() as u64)
            } else {
                user_copy(kernel, address, bytes.len() as u64)
            }
        };
        unsafe {
            COPY.0.get().write(None);
        }
        r.unpin(pin).expect("balanced copy pin");
        if failed == 0 {
            Ok(())
        } else {
            Err(INVALIDARGUMENT)
        }
    })
}
fn valid_return(f: &Frame) -> bool {
    if !return_state(f.ip, f.sp, f.cs, f.ss, f.flags) {
        return false;
    }
    runtime_memory::with(|r, _| {
        r.tables(current().space)
            .and_then(|t| {
                Ok(t.user_access(VirtAddr(f.ip), false, true)?
                    && t.user_access(VirtAddr(f.sp - 1), true, false)?)
            })
            .unwrap_or(false)
    })
}
fn allocate(bytes: u64, alignment: u64) -> core::result::Result<u64, u64> {
    if !alignment.is_power_of_two() || alignment > 2 * 1024 * 1024 {
        return Err(INVALIDARGUMENT);
    }
    if bytes == 0 {
        return Ok(0);
    }
    let rounded = bytes.checked_add(PAGE - 1).ok_or(INVALIDARGUMENT)? & !(PAGE - 1);
    let heap = unsafe { &mut *HEAP.0.get() };
    let used: u64 = heap.iter().flatten().map(|h| h.bytes).sum();
    if rounded > 32 * 1024 * 1024 - used {
        return Err(QUOTAEXCEEDED);
    }
    let slot = heap.iter().position(Option::is_none).ok_or(QUOTAEXCEEDED)?;
    let base = 0x10000000 + slot as u64 * 32 * 1024 * 1024;
    let space = current().space;
    let id = runtime_memory::with(|r, store| {
        let a = r.allocate(store, space, rounded, alignment)?.unwrap();
        if let Err(e) = r.map(store, space, a, base, 0, rounded, Access::Write) {
            r.free(store, space, a)?;
            return Err(e);
        }
        Ok(a)
    })
    .map_err(|e| {
        if e.reason == "quota" {
            QUOTAEXCEEDED
        } else {
            OUTOFMEMORY
        }
    })?;
    heap[slot] = Some(Heap {
        base,
        bytes: rounded,
        id,
    });
    let committed = runtime_memory::with(|r, _| r.inventory().committed);
    unsafe {
        PEAK.0.get().write((*PEAK.0.get()).max(committed));
    }
    Ok(base)
}
fn free(base: u64) -> core::result::Result<u64, u64> {
    if base == 0 {
        return Ok(0);
    }
    let heap = unsafe { &mut *HEAP.0.get() };
    let slot = heap
        .iter()
        .position(|h| h.is_some_and(|h| h.base == base))
        .ok_or(INVALIDARGUMENT)?;
    let h = heap[slot].unwrap();
    let s = current().space;
    runtime_memory::with(|r, b| {
        r.unmap(b, s, h.base)?;
        r.free(b, s, h.id)
    })
    .map_err(|_| INTERNALFAILURE)?;
    heap[slot] = None;
    Ok(0)
}
extern "C" fn dispatch(ptr: *mut Frame) {
    let hooks = fixture::hooks();
    let entered = (hooks.entry_time)();
    let f = unsafe { &mut *ptr };
    let c = current();
    let flags: u64;
    let sp: u64;
    let root: u64;
    unsafe {
        asm!("pushfq","pop {}","mov {}, rsp","mov {}, cr3",out(reg) flags,out(reg) sp,out(reg) root);
    }
    let expected = runtime_memory::with(|r, _| {
        Ok::<_, continuum_kernel::Error>((r.kernel_stack(c.space)?, r.root(c.space)?.0))
    })
    .unwrap();
    assert!(
        flags & (0x200 | 0x400 | 0x100 | 0x40000 | 0x4000) == 0
            && expected.0.base <= sp
            && sp < expected.0.end()
            && root == expected.1
    );
    unsafe {
        *CALLS.0.get() += 1;
    }
    (hooks.entry)(f, entered);
    if !valid_return(f) {
        record(
            "m1.user_bad_return",
            "user-boundary",
            "ok",
            format_args!(
                "case={} ip={:#x} sp={:#x} cs={:#x} ss={:#x} flags={:#x} operation={}",
                c.case, f.ip, f.sp, f.cs, f.ss, f.flags, f.rax
            ),
        );
        finish("return-state", 0x200, 0, f.ip, 0);
    }
    let operation = f.rax;
    (hooks.before_operation)();
    f.rax = 0;
    let value = if f.r9 != 0 {
        Err(INCOMPATIBLEPROTOCOL)
    } else if f.r8 != 0 {
        Err(UNSUPPORTED)
    } else {
        let cap = (hooks.operation)(operation, f);
        if let Some(result) = cap {
            result
        } else {
            match operation {
                ABI_QUERY => {
                    let mut bytes = [0u8; 32];
                    for (i, v) in [0, PAGE, c.context.principal, c.context.session]
                        .into_iter()
                        .enumerate()
                    {
                        bytes[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes());
                    }
                    if f.rsi < 32 {
                        Err(BUFFERTOOSMALL)
                    } else {
                        copy(f.rdi, &mut bytes, true).map(|_| 32)
                    }
                }
                DIAGNOSTIC_LOG => {
                    if f.rsi > 256 {
                        Err(INVALIDARGUMENT)
                    } else {
                        let mut bytes = [0u8; 256];
                        let n = f.rsi as usize;
                        copy(f.rdi, &mut bytes[..n], false).and_then(|_| {
                            if bytes[..n]
                                .iter()
                                .any(|b| !b.is_ascii_alphanumeric() && *b != b'-')
                            {
                                return Err(INVALIDARGUMENT);
                            }
                            for offset in (0..n.max(1)).step_by(96) {
                                let end = (offset + 96).min(n);
                                record(
                                    "m1.user_log",
                                    "user-boundary",
                                    "ok",
                                    format_args!(
                                        "case={} principal={} offset={offset} total={n} token={}",
                                        c.case,
                                        c.context.principal,
                                        if n == 0 {
                                            "empty"
                                        } else {
                                            core::str::from_utf8(&bytes[offset..end]).unwrap()
                                        }
                                    ),
                                );
                            }
                            Ok(n as u64)
                        })
                    }
                }
                MEMORY_ALLOCATE => allocate(f.rdi, f.rsi),
                MEMORY_FREE => free(f.rdi),
                THREAD_YIELD => {
                    // No structural borrow/pin is held. Exercise a real nested
                    // timer IRQ on this owned kernel stack, then mask again.
                    let before = crate::timer::uptime().unwrap().0;
                    unsafe {
                        asm!("sti", "hlt", "cli");
                    }
                    let after = crate::timer::uptime().unwrap().0;
                    assert!(after > before);
                    record(
                        "m1.user_irq",
                        "user-boundary",
                        "ok",
                        format_args!("case={} before={before} after={after} nested=true", c.case),
                    );
                    Ok(0) // The boundary fixture resumes this owner after its nested IRQ.
                }
                PROCESS_EXIT => finish("exit", f.rdi, 0, f.ip, 0),
                _ => Err(UNSUPPORTED),
            }
        }
    };
    match value {
        Ok(v) => f.rdx = v,
        Err(e) => {
            f.rax = e;
            f.rdx = if operation == ABI_QUERY && e == BUFFERTOOSMALL {
                32
            } else {
                0
            };
        }
    }
    if !(hooks.scheduled)() {
        record(
            "m1.user_call",
            "user-boundary",
            "ok",
            format_args!(
                "case={} slot={} generation={} operation={operation} status={} value={} a={:#x} b={} flags={:#x} entry_flags={flags:#x}",
                c.case, c.id.slot, c.id.generation, f.rax, f.rdx, f.rdi, f.rsi, f.flags
            ),
        );
    }
    // Map/free can change CR3. Validate again immediately before the assembly
    // IRETQ; no locks, user calls, preemption or mutations intervene afterwards.
    if !valid_return(f) {
        finish("return-state", 0x200, 0, f.ip, 0);
    }
    (hooks.return_to_user)(f, operation);
}
pub fn run(group: u64) -> Result<()> {
    fixture::install_boundary();
    unsafe {
        asm!("cli");
        let root: u64;
        asm!("mov {}, cr3",out(reg) root);
        KERNEL_ROOT.0.get().write(root);
    }
    install()?;
    let empty = runtime_memory::with(|r, _| r.inventory());
    let frames = crate::paging::runtime_frames();
    let extra = core::mem::size_of::<Launcher<16>>()
        + core::mem::size_of::<Option<Current>>()
        + core::mem::size_of::<[Option<Heap>; 32]>()
        + core::mem::size_of::<Option<(u64, u64)>>()
        + core::mem::size_of::<fixture::Hooks>()
        + 8 * 8; // seven entry counters/slots plus the timer user-IRQ counter
    need(
        empty.metadata_bytes + empty.table_storage_bytes + 16 * 73728 + extra <= 16 * 1024 * 1024,
    )?;
    record(
        "m1.user_runtime",
        "user-boundary",
        "ok",
        format_args!(
            "extra_bytes={extra} fixed_total={} ceiling=16777216 heap_ceiling=33554432 slots=32 fp_ts=true fp_em=false fp_osfxsr=true",
            empty.metadata_bytes + empty.table_storage_bytes + 16 * 73728 + extra
        ),
    );
    // All faults have an immediately following healthy replacement, no VM reset.
    let first = group * 4 + 1;
    let cases = [0, first, 0, first + 1, 0, first + 2, 0, first + 3, 0];
    for (index, case) in cases.into_iter().enumerate() {
        let binding = if index % 2 == 0 {
            Binding::UserA
        } else {
            Binding::UserB
        };
        let module = if binding == Binding::UserA { 0 } else { 1 };
        let (digest, size) = IMAGES[module];
        if case == 16 {
            // The allocator only permits quota changes while empty. Admit both
            // image plans, the foreign/control pages and one additional MiB.
            let staged = parse(crate::paging::module(module)?)?.commitment
                + parse(crate::paging::module(2)?)?.commitment;
            let limit = (staged / PAGE) as usize + 2 + 256;
            crate::paging::runtime_quota(limit)?;
            record(
                "m1.user_oom_injection",
                "user-boundary",
                "ok",
                format_args!(
                    "case=16 additional_pages=256 frame_limit={limit} source=trusted-supervisor"
                ),
            );
        }

        let id = runtime_memory::with(|r, b| unsafe {
            (&mut *LAUNCHER.0.get()).bootstrap_return(
                r,
                b,
                TrustedImage {
                    bytes: crate::paging::module(module)?,
                    digest,
                    size,
                },
                binding,
                EXIT_TRAMPOLINE,
            )
        })?;
        // A disjoint foreign mapping exists only in the other staged process.
        let other = runtime_memory::with(|r, b| unsafe {
            (&mut *LAUNCHER.0.get()).bootstrap_return(
                r,
                b,
                TrustedImage {
                    bytes: crate::paging::module(2)?,
                    digest: IMAGES[2].0,
                    size: IMAGES[2].1,
                },
                Binding::Service,
                EXIT_TRAMPOLINE,
            )
        })?;
        runtime_memory::with(|r, b| {
            let p = unsafe { (&*LAUNCHER.0.get()).get(other)? };
            let a = r.allocate(b, p.space, PAGE, PAGE)?.unwrap();
            r.map(b, p.space, a, 0x60000000, 0, PAGE, Access::Write)
        })?;
        let p = unsafe { (&*LAUNCHER.0.get()).get(id)? };
        let c = Current {
            id,
            space: p.space,
            context: p.context,
            case,
        };
        let (_, stack) = runtime_memory::with(|r, _| {
            Ok::<_, continuum_kernel::Error>((r.root(p.space)?.0, r.kernel_stack(p.space)?))
        })?;
        let entry = p.entry;
        let rsp = p.stack;
        need(return_state(entry, rsp, USER_CS, USER_SS, 0x202))?;
        unsafe {
            CURRENT.0.get().write(Some(c));
            KERNEL_SP.0.get().write(stack.end());
            RESULT.0.get().write(u64::MAX);
            HEAP.0.get().write([None; 32]);
        }
        crate::exceptions::user_stack(stack.end());
        // Case is untrusted fixture input in an owned RW page, never provenance.
        runtime_memory::with(|r, b| {
            let a = r.allocate(b, c.space, PAGE, PAGE)?.unwrap();
            r.write_staging(b, c.space, a, 0, &case.to_le_bytes())?;
            r.write_staging(b, c.space, a, 8, &(index as u64).to_le_bytes())?;
            r.map(b, c.space, a, 0x70000000, 0, PAGE, Access::Write)
        })?;
        let root = runtime_memory::with(|r, _| r.root(c.space))?.0;
        let before = crate::timer::uptime()?.0;
        // A previous user can alter permitted segment state. Fresh launches
        // reset it even though these audited integer fixtures do not use TLS.
        unsafe {
            for msr in [0xc0000100, 0xc0000101, 0xc0000102] {
                wrmsr(msr, 0);
            }
        }
        record(
            "m1.user_start",
            "user-boundary",
            "ok",
            format_args!(
                "index={index} case={case} slot={} generation={} principal={} authority_session={} kernel_stack={:#x} root={root:#x} ticks={before} cpl=3",
                id.slot, id.generation, c.context.principal, c.context.session, stack.base
            ),
        );
        let _ = root;
        unsafe {
            user_enter(root, entry, rsp);
        }
        // This is back on the retained M1 stack and root. Only now reap storage.
        let result = unsafe { RESULT.0.get().read() };
        let expected = match case {
            0 => 0,
            1 | 2 => 0x10e,
            3 => 0x106,
            4 | 5 => 0x10d,
            6 => 1,
            7 | 16 => 2,
            8 | 9 | 12 | 13 => 0x200,
            10 => 0x10e,
            11 => 0x107,
            14 => 0x200,
            15 => 0x10d,
            _ => u64::MAX,
        };
        need(result == expected)?;
        unsafe {
            CURRENT.0.get().write(None);
            HEAP.0.get().write([None; 32]);
        }
        crate::exceptions::user_stack(crate::exceptions::stacks()[0] + 69632);
        runtime_memory::with(|r, b| unsafe {
            let l = &mut *LAUNCHER.0.get();
            l.destroy(r, b, id)?;
            l.destroy(r, b, other)
        })?;
        if case == 16 {
            crate::paging::runtime_quota(usize::MAX)?;
        }
        let inventory = runtime_memory::with(|r, _| r.inventory());
        record(
            "m1.user_inventory",
            "user-boundary",
            "ok",
            format_args!(
                "index={index} spaces={} allocations={} committed={} resident={} mappings={} tables={} stacks={} pins={}",
                inventory.spaces,
                inventory.allocations,
                inventory.committed,
                inventory.resident,
                inventory.mappings,
                inventory.page_tables,
                inventory.kernel_stack_bytes,
                inventory.active_pins
            ),
        );
        need(
            runtime_memory::with(|r, _| r.inventory()) == empty
                && crate::paging::runtime_frames() == frames,
        )?;
        let after = crate::timer::uptime()?.0;
        need(after >= before)?;
        record(
            "m1.user_settlement",
            "user-boundary",
            "ok",
            format_args!(
                "index={index} case={case} result={result} settled=true pins=0 ticks_before={before} ticks_after={after} frames={frames}"
            ),
        );
    }
    // Deliberate kernel copy fault proves the narrow recovery path, separately
    // from prevalidated pointer denials; no pin is bypassed in a public operation.
    let mut byte = 0u8;
    unsafe {
        COPY.0.get().write(Some((0x60000000, 0x60000001)));
    }
    need(unsafe { user_copy(&mut byte as *mut u8 as u64, 0x60000000, 1) } == 1)?;
    need(unsafe { user_copy(0x60000000, &byte as *const u8 as u64, 1) } == 1)?;
    unsafe {
        COPY.0.get().write(None);
    }
    need(crate::timer::user_interrupts() > 0)?;
    record(
        "m1.user_ready",
        "user-boundary",
        "ok",
        format_args!(
            "runs=9 group={group} settled=true user_execution=true copy_recovery=true user_irqs={} peak_committed={} calls={}",
            crate::timer::user_interrupts(),
            unsafe { *PEAK.0.get() },
            unsafe { *CALLS.0.get() }
        ),
    );
    unsafe {
        asm!("sti");
    }
    Ok(())
}
