//! Rights-bearing execution adapter; historical P3-04/05 fixtures remain separate.
use super::*;
use continuum_kernel::objects::{self as o, Kind, Registry};
static REGISTRY: Local<Registry> = Local(UnsafeCell::new(Registry::EMPTY));
static ENABLED: Local<bool> = Local(UnsafeCell::new(false));
static SHARED: Local<Option<(SpaceId, u64)>> = Local(UnsafeCell::new(None));
static CHANNELS: Local<bool> = Local(UnsafeCell::new(false));
static ENDPOINTS: Local<[u64; 2]> = Local(UnsafeCell::new([0; 2]));
static ENTERED: Local<bool> = Local(UnsafeCell::new(false));
struct RegistryEntry;
impl RegistryEntry {
    fn acquire() -> Self {
        let flags: u64;
        unsafe {
            asm!("pushfq","pop {}",out(reg)flags);
        }
        assert!(flags & 0x200 == 0 && !unsafe { *ENTERED.0.get() });
        unsafe {
            *ENTERED.0.get() = true;
        }
        Self
    }
}
impl Drop for RegistryEntry {
    fn drop(&mut self) {
        unsafe {
            *ENTERED.0.get() = false;
        }
    }
}
pub(super) fn quiescent() -> bool {
    !unsafe { *ENTERED.0.get() }
}
#[cfg(feature = "wait-test")]
pub(super) fn registry<R>(f: impl FnOnce(&mut Registry) -> R) -> R {
    let _entry = RegistryEntry::acquire();
    let flags: u64;
    unsafe {
        asm!("pushfq", "pop {}", out(reg) flags);
    }
    assert!(flags & 0x200 == 0);
    f(unsafe { &mut *REGISTRY.0.get() })
}
pub(super) fn channels() -> bool {
    unsafe { *CHANNELS.0.get() }
}
pub(super) fn enable_channels() {
    unsafe {
        *CHANNELS.0.get() = true;
    }
}
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
pub(super) fn status(e: o::Error) -> u64 {
    match e {
        o::Error::BadHandle => BADHANDLE,
        o::Error::WrongType => WRONGTYPE,
        o::Error::AccessDenied => ACCESSDENIED,
        o::Error::InvalidArgument => INVALIDARGUMENT,
        o::Error::QuotaExceeded => QUOTAEXCEEDED,
        o::Error::OutOfMemory => OUTOFMEMORY,
        o::Error::WouldBlock => WOULDBLOCK,
        o::Error::PeerClosed => PEERCLOSED,
        o::Error::BufferTooSmall => BUFFERTOOSMALL,
    }
}
fn result<T>(r: o::Result<T>) -> Result<T> {
    r.map_err(|_| error("capabilities", "self_check"))
}
pub(super) fn admit(
    space: SpaceId,
    module: usize,
    id: ProcessId,
    context: Context,
) -> Result<[u64; 4]> {
    runtime_memory::with(|r, b| {
        let c = unsafe { &mut *REGISTRY.0.get() };
        if channels() {
            use o::channels::{ENDPOINT_RIGHTS, Sender};
            let (p, t) = result(c.admit_sender(
                space,
                Sender {
                    process: id.slot as u64,
                    generation: id.generation,
                    principal: context.principal,
                    session: context.session,
                    service: context.service,
                },
            ))?;
            let (grant, extra) = if module == 0 {
                let mut a = [0; 2];
                let mut z = [0; 2];
                result(c.channel_pair(space, p, |v| {
                    a = v;
                    Ok(())
                }))?;
                result(c.channel_pair(space, p, |v| {
                    z = v;
                    Ok(())
                }))?;
                unsafe {
                    *SHARED.0.get() = Some((space, a[1]));
                    *ENDPOINTS.0.get() = [a[1], z[1]];
                }
                (a[0], z[0])
            } else {
                let source = unsafe { (*SHARED.0.get()).unwrap().0 };
                let h = unsafe { (*ENDPOINTS.0.get())[module - 1] };
                let granted = result(c.grant_endpoint(source, h, space, ENDPOINT_RIGHTS))?;
                result(c.close(r, b, source, h))?;
                (granted, 0)
            };
            record(
                "m1.ipc_admit",
                "channels",
                "ok",
                format_args!(
                    "module={module} process={} generation={} principal={} authority_session={} self_handle={p} thread={t} endpoint={grant} extra={extra}",
                    id.slot, id.generation, context.principal, context.session
                ),
            );
            return Ok([p, t, grant, extra]);
        }
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
fn channel_dispatch(op: u64, f: &Frame) -> Option<core::result::Result<u64, u64>> {
    if !(13..=16).contains(&op) {
        return None;
    }
    use o::channels::{ATTACHMENTS, Attachment, INLINE, Mode};
    let space = current().space;
    let v = (|| {
        if op == 13 {
            if f.rdx != 16 {
                return Err(INVALIDARGUMENT);
            }
            let c = unsafe { &mut *REGISTRY.0.get() };
            c.channel_pair(space, f.rdi, |h| {
                let mut bytes = [0u8; 16];
                for i in 0..2 {
                    bytes[i * 8..i * 8 + 8].copy_from_slice(&h[i].to_le_bytes());
                }
                copy(f.rsi, &mut bytes, true).map_err(|_| o::Error::InvalidArgument)
            })
            .map_err(status)?;
        } else if op == 16 {
            let (bytes, handles) =
                unsafe { (&*REGISTRY.0.get()).channel_size(space, f.rdi) }.map_err(status)?;
            return Ok((handles as u64) << 32 | bytes as u64);
        } else {
            let n = if op == 14 { 32 } else { 48 };
            if f.rdx != n {
                return Err(INVALIDARGUMENT);
            }
            let mut request = [0; 48];
            copy(f.rsi, &mut request[..n as usize], false)?;
            let w: [u64; 6] = core::array::from_fn(|i| {
                u64::from_le_bytes(request[i * 8..i * 8 + 8].try_into().unwrap())
            });
            if w[1] > INLINE as u64 || w[3] > ATTACHMENTS as u64 {
                return Err(INVALIDARGUMENT);
            }
            if op == 14 {
                let mut bytes = [0; INLINE];
                copy(w[0], &mut bytes[..w[1] as usize], false)?;
                let mut descriptors = [0u8; 24 * ATTACHMENTS];
                copy(w[2], &mut descriptors[..w[3] as usize * 24], false)?;
                let mut attachments = [Attachment {
                    handle: 0,
                    rights: 0,
                    mode: Mode::Move,
                }; ATTACHMENTS];
                for i in 0..w[3] as usize {
                    let d = &descriptors[i * 24..i * 24 + 24];
                    if d[20..24] != [0; 4] {
                        return Err(INVALIDARGUMENT);
                    }
                    attachments[i] = Attachment {
                        handle: u64::from_le_bytes(d[..8].try_into().unwrap()),
                        rights: u64::from_le_bytes(d[8..16].try_into().unwrap()),
                        mode: match u32::from_le_bytes(d[16..20].try_into().unwrap()) {
                            0 => Mode::Move,
                            1 => Mode::Duplicate,
                            _ => return Err(INVALIDARGUMENT),
                        },
                    };
                }
                #[cfg(feature = "wait-test")]
                super::waiting::check_attempt_deadline()?;
                unsafe {
                    (&mut *REGISTRY.0.get()).channel_send_checked(
                        space,
                        f.rdi,
                        &bytes[..w[1] as usize],
                        &attachments[..w[3] as usize],
                        || {
                            #[cfg(feature = "wait-test")]
                            super::waiting::check_attempt_deadline()
                                .map_err(|_| o::Error::WouldBlock)?;
                            Ok(())
                        },
                    )
                }
                .map_err(status)?;
            } else {
                if w[5] < 64 {
                    return Err(BUFFERTOOSMALL);
                }
                let spans = [(w[0], w[1]), (w[2], w[3] * 8), (w[4], 64)];
                for i in 0..3 {
                    for j in 0..i {
                        let (a, n) = spans[i];
                        let (b, m) = spans[j];
                        let end = a.checked_add(n).ok_or(INVALIDARGUMENT)?;
                        let other = b.checked_add(m).ok_or(INVALIDARGUMENT)?;
                        if n != 0 && m != 0 && a < other && b < end {
                            return Err(INVALIDARGUMENT);
                        }
                    }
                }
                let mut delivered = None;
                let c = unsafe { &mut *REGISTRY.0.get() };
                c.channel_receive(
                    space,
                    f.rdi,
                    w[1] as usize,
                    w[3] as usize,
                    |bytes, handles, sender| {
                        let mut data = [0; INLINE];
                        data[..bytes.len()].copy_from_slice(bytes);
                        let mut hs = [0; ATTACHMENTS * 8];
                        for (i, h) in handles.iter().enumerate() {
                            hs[i * 8..i * 8 + 8].copy_from_slice(&h.to_le_bytes());
                        }
                        let mut meta = [0; 64];
                        for (i, v) in [
                            bytes.len() as u64,
                            handles.len() as u64,
                            sender.process,
                            sender.generation,
                            sender.principal,
                            sender.session,
                            u64::from(sender.service),
                            0,
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            meta[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes());
                        }
                        copy(w[0], &mut data[..bytes.len()], true)
                            .map_err(|_| o::Error::InvalidArgument)?;
                        copy(w[2], &mut hs[..handles.len() * 8], true)
                            .map_err(|_| o::Error::InvalidArgument)?;
                        copy(w[4], &mut meta, true).map_err(|_| o::Error::InvalidArgument)?;
                        delivered = Some((sender, bytes.len(), handles.len()));
                        #[cfg(feature = "wait-test")]
                        super::waiting::check_attempt_deadline()
                            .map_err(|_| o::Error::WouldBlock)?;
                        Ok(())
                    },
                )
                .map_err(status)?;
                if let Some((sender, bytes, handles)) = delivered {
                    record(
                        "m1.ipc_sender",
                        "channels",
                        "ok",
                        format_args!(
                            "receiver={} generation={} sender={} sender_generation={} principal={} authority_session={} bytes={bytes} handles={handles}",
                            current().id.slot,
                            current().id.generation,
                            sender.process,
                            sender.generation,
                            sender.principal,
                            sender.session
                        ),
                    );
                }
            }
        }
        runtime_memory::with(|r, b| unsafe { (&mut *REGISTRY.0.get()).sweep(r, b) })
            .map_err(status)?;
        Ok(0)
    })();
    let u = unsafe { (&*REGISTRY.0.get()).channel_usage() };
    let inventory = inventory();
    let (e, value) = match v {
        Ok(value) => (0, value),
        Err(e) => (e, 0),
    };
    record(
        "m1.ipc_call",
        "channels",
        "ok",
        format_args!(
            "process={} generation={} operation={op} status={e} value={value} pairs={} messages={} bytes={} held={} objects={} handles={}",
            current().id.slot,
            current().id.generation,
            u.pairs,
            u.messages,
            u.bytes,
            u.handles,
            inventory.objects,
            inventory.handles
        ),
    );
    Some(v)
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
    let _entry = RegistryEntry::acquire();
    let space = current().space;
    if channels() {
        if let Some(value) = channel_dispatch(op, f) {
            return Some(value);
        }
    }
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
                MEMORY_CREATE => {
                    if channels() {
                        c.create_transferable(r, b, space, f.rdi, f.rsi, f.rdx)
                    } else {
                        c.create(r, b, space, f.rdi, f.rsi, f.rdx)
                    }
                }
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
