//! Small fallible helpers. Handles are manually owned; copying their numeric
//! value does not duplicate authority. All receive outputs require success.
use crate::*;
fn value(v: (u64, u64)) -> Result<u64, u64> {
    if v.0 == 0 { Ok(v.1) } else { Err(v.0) }
}
pub fn close(h: Handle) -> Result<(), u64> {
    value(unsafe { syscall(HANDLE_CLOSE, h.0, 0, 0, 0) }).map(|_| ())
}
pub fn duplicate(h: Handle, rights: u64) -> Result<Handle, u64> {
    value(unsafe { syscall(HANDLE_DUPLICATE, h.0, rights, 0, 0) }).map(Handle)
}
pub fn now() -> Result<MonotonicNs, u64> {
    value(unsafe { syscall(CLOCK_NOW, 0, 0, 0, 0) }).map(MonotonicNs)
}
pub fn yield_now(thread: Handle) -> Result<(), u64> {
    value(unsafe { syscall(THREAD_YIELD, thread.0, 0, 0, 0) }).map(|_| ())
}
pub fn wait(
    handle: Handle,
    interest: u64,
    bytes: u64,
    handles: u64,
    deadline: MonotonicNs,
    cancel: Handle,
) -> Result<usize, u64> {
    let item = wire::wait_item(handle, interest, bytes, handles)?;
    let req = wire::words::<4, 32>([item.as_ptr() as u64, 1, deadline.0, cancel.0]);
    value(unsafe { syscall(WAIT_MANY, req.as_ptr() as u64, 32, 0, 0) }).map(|v| v as usize)
}
pub fn send_until(
    endpoint: Handle,
    bytes: &[u8],
    attachments: &[[u8; 24]],
    deadline: MonotonicNs,
    cancel: Handle,
) -> Result<(), u64> {
    if bytes.len() > 256 || attachments.len() > 4 {
        return Err(INVALIDARGUMENT);
    }
    for a in attachments {
        wire::validate("attachment", a)?;
    }
    let desc = wire::words::<4, 32>([
        bytes.as_ptr() as u64,
        bytes.len() as u64,
        attachments.as_ptr() as u64,
        attachments.len() as u64,
    ]);
    let attempt = wire::words::<3, 24>([desc.as_ptr() as u64, deadline.0, cancel.0]);
    loop {
        match value(unsafe {
            syscall(
                CHANNEL_SEND_UNTIL,
                endpoint.0,
                attempt.as_ptr() as u64,
                24,
                0,
            )
        }) {
            Ok(_) => return Ok(()),
            Err(WOULDBLOCK) => {
                wait(
                    endpoint,
                    3,
                    bytes.len() as u64,
                    attachments.len() as u64,
                    deadline,
                    cancel,
                )?;
            }
            Err(e) => return Err(e),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Received {
    pub bytes: usize,
    pub handles: usize,
    pub sender: wire::Identity,
}
pub fn receive_until(
    endpoint: Handle,
    bytes: &mut [u8],
    handles: &mut [[u8; 8]],
    deadline: MonotonicNs,
    cancel: Handle,
) -> Result<Received, u64> {
    let mut metadata = [0; 64];
    let desc = wire::words::<6, 48>([
        bytes.as_mut_ptr() as u64,
        bytes.len() as u64,
        handles.as_mut_ptr() as u64,
        handles.len() as u64,
        metadata.as_mut_ptr() as u64,
        64,
    ]);
    let attempt = wire::words::<3, 24>([desc.as_ptr() as u64, deadline.0, cancel.0]);
    loop {
        match value(unsafe {
            syscall(
                CHANNEL_RECEIVE_UNTIL,
                endpoint.0,
                attempt.as_ptr() as u64,
                24,
                0,
            )
        }) {
            Ok(_) => {
                wire::validate("metadata", &metadata)?;
                let w = wire::read_words::<8>(&metadata)?;
                return Ok(Received {
                    bytes: w[0] as usize,
                    handles: w[1] as usize,
                    sender: wire::Identity {
                        process: w[2],
                        generation: w[3],
                        principal: w[4],
                        session: w[5],
                        service: w[6] != 0,
                    },
                });
            }
            Err(WOULDBLOCK) => {
                wait(endpoint, 2, 0, 0, deadline, cancel)?;
            }
            Err(e) => return Err(e),
        }
    }
}
