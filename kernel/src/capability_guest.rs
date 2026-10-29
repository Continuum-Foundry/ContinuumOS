//! Rights-bearing execution adapter; historical P3-04/05 fixtures remain separate.
use super::*;
use continuum_kernel::objects::{self as o, Kind, Registry};
static REGISTRY: Local<Registry> = Local(UnsafeCell::new(Registry::EMPTY));
static ENABLED: Local<bool> = Local(UnsafeCell::new(false));
static SHARED: Local<Option<(SpaceId, u64)>> = Local(UnsafeCell::new(None));
pub(super) fn enabled() -> bool {
    unsafe { *ENABLED.0.get() }
}
pub(super) fn enable() {
    unsafe {
        *ENABLED.0.get() = true;
    }
}
pub(super) fn inventory() -> o::Inventory {
    unsafe { (&*REGISTRY.0.get()).inventory() }
}
fn status(e: o::Error) -> u64 {
    match e {
        o::Error::BadHandle => BADHANDLE,
        o::Error::WrongType => WRONGTYPE,
        o::Error::AccessDenied => ACCESSDENIED,
        o::Error::InvalidArgument => INVALIDARGUMENT,
        o::Error::QuotaExceeded => QUOTAEXCEEDED,
        o::Error::OutOfMemory => OUTOFMEMORY,
        o::Error::WouldBlock => WOULDBLOCK,
    }
}
fn result<T>(r: o::Result<T>) -> Result<T> {
    r.map_err(|_| error("capabilities", "self_check"))
}
pub(super) fn admit(space: SpaceId, module: usize) -> Result<[u64; 4]> {
    runtime_memory::with(|r, b| {
        let c = unsafe { &mut *REGISTRY.0.get() };
        let (p, t) = result(c.admit(space))?;
        let (grant, foreign) = if module == 0 {
            let h = result(c.create(r, b, space, p, PAGE, o::READ))?;
            result(c.seed(r, b, space, h, b"shared-read"))?;
            unsafe {
                *SHARED.0.get() = Some((space, h));
            }
            (h, h)
        } else {
            let (owner, h) = unsafe { (*SHARED.0.get()).unwrap() };
            (
                if module == 1 {
                    result(c.grant_read(owner, h, space))?
                } else {
                    0
                },
                h,
            )
        };
        record(
            "m1.cap_admit",
            "capabilities",
            "ok",
            format_args!(
                "module={module} process={p} thread={t} grant={grant} foreign={foreign} explicit_read={} fixed_bytes={}",
                module == 1,
                core::mem::size_of::<Registry>()
            ),
        );
        Ok([p, t, grant, foreign])
    })
}
pub(super) fn remove(space: SpaceId) -> Result<()> {
    runtime_memory::with(|r, b| result(unsafe { (&mut *REGISTRY.0.get()).remove(r, b, space) }))
}
fn access(value: u64) -> core::result::Result<Access, u64> {
    match value {
        0 => Ok(Access::Read),
        1 => Ok(Access::Write),
        2 => Ok(Access::Execute),
        _ => Err(INVALIDARGUMENT),
    }
}
fn log_call(op: u64, f: &Frame, value: core::result::Result<u64, u64>, request: Option<[u64; 4]>) {
    let c = current();
    let (status, value) = match value {
        Ok(v) => (0, v),
        Err(e) => (e, 0),
    };
    if let Some(w) = request {
        record(
            "m1.cap_call",
            "capabilities",
            "ok",
            format_args!(
                "case={} process={} generation={} operation={op} status={status} value={value} a={} b={} c={} d={} n={} x={}",
                c.case, c.id.slot, c.id.generation, f.rdi, f.rsi, w[0], w[1], w[2], w[3]
            ),
        );
        return;
    }
    record(
        "m1.cap_call",
        "capabilities",
        "ok",
        format_args!(
            "case={} process={} generation={} operation={op} status={status} value={value} a={} b={} c={}",
            c.case, c.id.slot, c.id.generation, f.rdi, f.rsi, f.rdx
        ),
    );
}
pub(super) fn dispatch(op: u64, f: &Frame) -> Option<core::result::Result<u64, u64>> {
    if !enabled() {
        return None;
    }
    if [ABI_QUERY, DIAGNOSTIC_LOG].contains(&op) {
        return None;
    }
    let space = current().space;
    if [PROCESS_EXIT, THREAD_YIELD].contains(&op) {
        let (handle, kind) = if op == PROCESS_EXIT {
            (f.rsi, Kind::Process)
        } else {
            (f.rdi, Kind::Thread)
        };
        let v = unsafe { (&*REGISTRY.0.get()).control(space, handle, kind) }
            .map(|_| 0)
            .map_err(status);
        if op != THREAD_YIELD || v.is_err() {
            log_call(op, f, v, None);
        }
        return if v.is_ok() { None } else { Some(v) };
    }
    let mut request = None;
    let v = (|| {
        // Copy/validate the complete request before borrowing the object table.
        let mut bytes = [0u8; 32];
        if op == MEMORY_MAP {
            if f.r10 != 32 {
                return Err(INVALIDARGUMENT);
            }
            copy(f.rdx, &mut bytes, false)?;
        }
        let words = core::array::from_fn::<_, 4, _>(|i| {
            u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap())
        });
        let access = if op == MEMORY_MAP {
            request = Some(words);
            access(words[3])?
        } else if op == MEMORY_PROTECT {
            access(f.rdx)?
        } else {
            Access::Read
        };
        let value = runtime_memory::with(|r, b| {
            let c = unsafe { &mut *REGISTRY.0.get() };
            match op {
                OBJECT_INFO => c.info(space, f.rdi),
                HANDLE_DUPLICATE => c.duplicate(space, f.rdi, f.rsi),
                HANDLE_CLOSE => c.close(r, b, space, f.rdi).map(|_| 0),
                MEMORY_CREATE => c.create(r, b, space, f.rdi, f.rsi, f.rdx),
                MEMORY_MAP => c
                    .map(
                        r, b, space, f.rdi, f.rsi, words[0], words[1], words[2], access,
                    )
                    .map(|_| 0),
                MEMORY_UNMAP => c.unmap(r, b, space, f.rdi, f.rsi).map(|_| 0),
                MEMORY_PROTECT => c.protect(r, b, space, f.rdi, f.rsi, access).map(|_| 0),
                _ => return Err(UNSUPPORTED),
            }
            .map_err(status)
        });
        if op == MEMORY_MAP && value.is_ok() && words[0] == 0x50000000 {
            pinned_interleave(space, f.rdi, f.rsi, words[0]).expect("pinned interleave");
            runtime_memory::with(|r, _| {
                let t = r.tables(space).unwrap();
                let mut entries = [0u64; 4];
                let mut table = 0;
                for (n, shift) in [39, 30, 21, 12].into_iter().enumerate() {
                    entries[n] = t
                        .entry(table, ((words[0] >> shift) & 511) as usize)
                        .unwrap();
                    if n < 3 {
                        table =
                            (((entries[n] & 0x000f_ffff_ffff_f000) - t.root().0) / PAGE) as usize;
                    }
                }
                let alias = crate::paging::runtime_alias(continuum_kernel::PhysAddr(
                    entries[3] & 0x000f_ffff_ffff_f000,
                ))
                .unwrap();
                record(
                    "m1.cap_map",
                    "capabilities",
                    "ok",
                    format_args!(
                        "case={} memory={} base={} e0={:#x} e1={:#x} e2={:#x} e3={:#x} alias={alias:#x}",
                        current().case,
                        f.rsi,
                        words[0],
                        entries[0],
                        entries[1],
                        entries[2],
                        entries[3]
                    ),
                );
            });
        }
        value
    })();
    log_call(op, f, v, request);
    Some(v)
}

/// A real pending IRQ with a live shared-copy pin. Every attempted mutation
/// below uses the registry entry points and must leave both inventories exact.
pub(super) fn pinned_interleave(
    space: SpaceId,
    process: u64,
    memory: u64,
    base: u64,
) -> Result<()> {
    runtime_memory::with(|r, b| {
        let c = unsafe { &mut *REGISTRY.0.get() };
        let pin = r.pin(space, base, 11, false)?;
        let before = (r.inventory(), c.inventory());
        scheduling::copy_irq(true);
        need(c.unmap(r, b, space, process, base) == Err(o::Error::WouldBlock))?;
        need(c.protect(r, b, space, process, base, Access::Read) == Err(o::Error::WouldBlock))?;
        need(c.remove(r, b, space) == Err(o::Error::WouldBlock))?;
        need(c.close(r, b, space, memory) == Err(o::Error::WouldBlock))?;
        need((r.inventory(), c.inventory()) == before)?;
        r.unpin(pin)?;
        record(
            "m1.cap_pin",
            "capabilities",
            "ok",
            format_args!(
                "unmap=6 protect=6 close=6 termination=6 unchanged=true irq_pending=true unpinned=true"
            ),
        );
        Ok(())
    })
}
