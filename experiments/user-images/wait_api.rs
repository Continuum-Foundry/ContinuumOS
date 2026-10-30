//! Draft no-std caller helpers, not the independently frozen P3-12 SDK. Every
//! request is encoded explicitly; no native struct layout crosses the boundary.
use super::runtime::*;
use core::arch::asm;
#[derive(Clone, Copy)]
pub struct MonotonicNs(pub u64);
#[derive(Clone, Copy)]
pub enum Deadline {
    Infinite,
    At(MonotonicNs),
}
impl Deadline {
    fn wire(self) -> u64 {
        match self {
            Self::Infinite => u64::MAX,
            Self::At(n) => n.0,
        }
    }
}
pub fn invoke(op: u64, a: u64, b: u64, c: u64) -> (u64, u64) {
    let e;
    let v;
    unsafe {
        asm!("syscall",inlateout("rax")op=>e,in("rdi")a,in("rsi")b,inlateout("rdx")c=>v,in("r10")0u64,in("r8")0u64,in("r9")0u64,lateout("rcx")_,lateout("r11")_);
    }
    (e, v)
}
fn put(out: &mut [u8], at: usize, value: u64) {
    out[at..at + 8].copy_from_slice(&value.to_le_bytes());
}
pub fn clock_now() -> Result<MonotonicNs, u64> {
    let (e, v) = invoke(CLOCK_NOW, 0, 0, 0);
    if e == 0 { Ok(MonotonicNs(v)) } else { Err(e) }
}
pub fn after(duration: u64) -> Result<Deadline, u64> {
    let now = clock_now()?.0;
    let value = now
        .checked_add(duration)
        .filter(|v| *v != u64::MAX)
        .ok_or(INVALIDARGUMENT)?;
    Ok(Deadline::At(MonotonicNs(value)))
}
pub fn wait_one(
    handle: u64,
    interest: u64,
    bytes: u64,
    attachments: u64,
    deadline: Deadline,
    cancel: u64,
) -> Result<(), u64> {
    let mut item = [0; 32];
    for (i, v) in [handle, interest, bytes, attachments]
        .into_iter()
        .enumerate()
    {
        put(&mut item, i * 8, v);
    }
    let mut request = [0; 32];
    for (i, v) in [item.as_ptr() as u64, 1, deadline.wire(), cancel]
        .into_iter()
        .enumerate()
    {
        put(&mut request, i * 8, v);
    }
    let (e, _) = invoke(WAIT_MANY, request.as_ptr() as u64, 32, 0);
    if e == 0 { Ok(()) } else { Err(e) }
}
fn attempt(
    op: u64,
    endpoint: u64,
    descriptor: &[u8],
    deadline: Deadline,
    cancel: u64,
) -> Result<(), u64> {
    let mut request = [0; 24];
    for (i, v) in [descriptor.as_ptr() as u64, deadline.wire(), cancel]
        .into_iter()
        .enumerate()
    {
        put(&mut request, i * 8, v);
    }
    let (e, _) = invoke(op, endpoint, request.as_ptr() as u64, 24);
    if e == 0 { Ok(()) } else { Err(e) }
}
pub fn send_until(
    endpoint: u64,
    bytes: &[u8],
    attachments: &[u8],
    deadline: Deadline,
    cancel: u64,
) -> Result<(), u64> {
    if bytes.len() > 4096 || attachments.len() > 192 || attachments.len() % 24 != 0 {
        return Err(INVALIDARGUMENT);
    }
    let mut descriptor = [0; 32];
    for (i, v) in [
        bytes.as_ptr() as u64,
        bytes.len() as u64,
        attachments.as_ptr() as u64,
        (attachments.len() / 24) as u64,
    ]
    .into_iter()
    .enumerate()
    {
        put(&mut descriptor, i * 8, v);
    }
    loop {
        match attempt(CHANNEL_SEND_UNTIL, endpoint, &descriptor, deadline, cancel) {
            Err(WOULDBLOCK) => wait_one(
                endpoint,
                3,
                bytes.len() as u64,
                (attachments.len() / 24) as u64,
                deadline,
                cancel,
            )?,
            value => return value,
        }
    }
}
pub fn receive_until(
    endpoint: u64,
    bytes: &mut [u8],
    handles: &mut [u8],
    metadata: &mut [u8; 64],
    deadline: Deadline,
    cancel: u64,
) -> Result<(), u64> {
    if bytes.len() > 4096 || handles.len() > 64 || handles.len() % 8 != 0 {
        return Err(INVALIDARGUMENT);
    }
    let mut descriptor = [0; 48];
    for (i, v) in [
        bytes.as_mut_ptr() as u64,
        bytes.len() as u64,
        handles.as_mut_ptr() as u64,
        (handles.len() / 8) as u64,
        metadata.as_mut_ptr() as u64,
        64,
    ]
    .into_iter()
    .enumerate()
    {
        put(&mut descriptor, i * 8, v);
    }
    loop {
        match attempt(
            CHANNEL_RECEIVE_UNTIL,
            endpoint,
            &descriptor,
            deadline,
            cancel,
        ) {
            Err(WOULDBLOCK) => wait_one(endpoint, 2, 0, 0, deadline, cancel)?,
            value => return value,
        }
    }
}
