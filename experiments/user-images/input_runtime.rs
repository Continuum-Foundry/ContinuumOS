//! Compositor-owned stable input broker. Device sampling is a bounded syscall;
//! policy, hit testing and event queues are in this user process.
use super::{check, input as i, surface as s, syscall};
use core::cell::UnsafeCell;
struct Owner(UnsafeCell<i::Broker>);
unsafe impl Sync for Owner {} // one owning user thread; no callbacks/signals/reentry
static INPUT: Owner = Owner(UnsafeCell::new(i::Broker::new(0, 0)));
fn state() -> &'static mut i::Broker {
    unsafe { &mut *INPUT.0.get() }
}
pub fn initialize(w: u32, h: u32) {
    check(state().dimensions(w, h).is_ok());
}
pub fn shutdown() {
    check(state().windows([None; i::MAX]).is_ok());
}
pub fn reset() {
    check(state().reset().is_ok());
}
fn call(op: u64, a: u64, b: u64) -> (u64, u64) {
    unsafe { syscall(op, a, b, 0, 0) }
}
#[cfg(surface_probe)]
struct Fixture(UnsafeCell<(bool, u8, u8)>);
#[cfg(surface_probe)]
unsafe impl Sync for Fixture {} // same single owning thread as INPUT
#[cfg(surface_probe)]
static FIXTURE: Fixture = Fixture(UnsafeCell::new((false, 0, 0)));
#[cfg(surface_probe)]
pub fn pressure() -> bool {
    let f = unsafe { &mut *FIXTURE.0.get() };
    if f.2 == 1 {
        f.2 = 2;
        true
    } else {
        false
    }
}
#[cfg(surface_probe)]
pub fn bind(peer: s::Peer, id: u64, scene: u64) {
    check(state().focus(peer, id, scene).is_ok());
}
#[cfg(surface_probe)]
pub fn seen(device: usize) -> u64 {
    state().device_sequence(device)
}
#[cfg(surface_probe)]
fn checkpoint(n: u8) {
    let mut text = *b"input-flood-00";
    text[12] = b'0' + n / 10;
    text[13] = b'0' + n % 10;
    check(
        call(
            super::DIAGNOSTIC_LOG,
            text.as_ptr() as u64,
            text.len() as u64,
        )
        .0 == 0,
    );
}
pub fn tick(surfaces: &s::Broker) {
    let input = state();
    check(input.windows(surfaces.input_windows()).is_ok());
    // Eight completions/device per turn. Neither rendering nor raw input can
    // monopolize the execution loop; the existing timer wait remains the idle path.
    for _ in 0..16 {
        let mut bytes = [0; 32];
        let (status, n) = call(0x4300, bytes.as_mut_ptr() as u64, 32);
        if status != 0 {
            check(call(0x4303, 0, 0).0 == 0);
            check(input.reset().is_ok());
            break;
        }
        if n == 0 {
            break;
        }
        check(n == 32);
        let w = |n| u64::from_le_bytes(bytes[n..n + 8].try_into().unwrap());
        let result = input.raw(
            w(16) as usize,
            w(0),
            w(8),
            u16::from_le_bytes(bytes[24..26].try_into().unwrap()),
            u16::from_le_bytes(bytes[26..28].try_into().unwrap()),
            i32::from_le_bytes(bytes[28..32].try_into().unwrap()),
        );
        if result.is_err() {
            check(call(0x4303, 0, 0).0 == 0);
            check(input.reset().is_ok());
            break;
        }
        #[cfg(surface_probe)]
        if w(16) == 0 && bytes[24..28] == [0; 4] {
            let f = unsafe { &mut *FIXTURE.0.get() };
            if w(0) == 4 && f.2 == 0 {
                f.2 = 1;
            }
            if f.0 {
                f.1 += 1;
                checkpoint(f.1);
                if f.1 == 33 {
                    f.0 = false;
                }
            }
        }
    }
    let (status, now) = call(super::CLOCK_NOW, 0, 0);
    check(status == 0);
    check(input.idle(now).is_ok());
    for w in surfaces.input_windows().into_iter().flatten() {
        // Hidden windows retain cancellation until restored; no ordinary input
        // can be authorized while hidden. Dead windows are removed above.

        for _ in 0..4 {
            let Some(event) = input.peek(w.peer, w.id, w.scene).unwrap_or(None) else {
                break;
            };
            let mut packet = [0; 128];
            for (n, v) in [
                w.peer.process,
                w.peer.incarnation,
                w.peer.principal,
                w.peer.session,
            ]
            .into_iter()
            .enumerate()
            {
                packet[n * 8..n * 8 + 8].copy_from_slice(&v.to_le_bytes());
            }
            packet[32..].copy_from_slice(&event.encode());
            let (status, n) = call(0x4301, packet.as_ptr() as u64, 128);
            if status != 0 || n == 0 {
                break;
            }
            check(n == 96);
            check(input.consumed(w.peer, w.id, w.scene).is_ok());
        }
    }
}
pub fn request(peer: s::Peer, m: s::Message<'_>) -> s::Result<u64> {
    let input = state();
    if m.resource != 1 || m.output != 1 || m.body[8..] != [0; 56] {
        return Err(s::INVALID);
    }
    let scene = s::u64_at(m.body, 0);
    #[cfg(surface_probe)]
    {
        let f = unsafe { &mut *FIXTURE.0.get() };
        if m.kind == 12 {
            if f.0 || f.1 != 0 || peer.principal != 403 {
                return Err(s::AUTHORITY);
            }
            input
                .focus(peer, m.surface, scene)
                .map_err(|_| s::AUTHORITY)?;
            f.0 = true;
            return Ok(0);
        }
        if m.kind == 10 && f.0 {
            return Err(s::BUSY);
        }
    }
    let r = match m.kind {
        10 => input.focus(peer, m.surface, scene),
        11 => input.capture(peer, m.surface, scene),
        _ => return Err(s::INVALID),
    };
    r.map(|_| 0).map_err(|_| s::AUTHORITY)
}
