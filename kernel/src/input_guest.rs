//! Private input copy transport. Only the trusted compositor reads physical events
//! and publishes them. App-provided identities cannot inject or redirect input.
use crate::{Local, record};
use continuum_kernel::surface_mailbox::Peer;
use core::cell::UnsafeCell;
struct Slot {
    peer: Option<Peer>,
    bytes: [u8; 96],
    full: bool,
}
impl Slot {
    const EMPTY: Self = Self {
        peer: None,
        bytes: [0; 96],
        full: false,
    };
}
static SLOTS: Local<[Slot; 2]> = Local(UnsafeCell::new([const { Slot::EMPTY }; 2]));
pub(crate) fn dead(peer: Peer, role: u64) {
    let slots = unsafe { &mut *SLOTS.0.get() };
    for s in slots {
        if role == 1 || s.peer == Some(peer) {
            *s = Slot::EMPTY;
        }
    }
}
pub(crate) fn call(
    op: u64,
    role: u64,
    peer: Peer,
    a: u64,
    b: u64,
    c: u64,
    g: u64,
    mut copy: impl FnMut(u64, &mut [u8], bool) -> core::result::Result<(), u64>,
    live: impl Fn(Peer) -> bool,
) -> Option<core::result::Result<u64, ()>> {
    if !(0x4300..=0x4303).contains(&op) {
        return None;
    }
    let r = (|| {
        if peer.session != 1001 || c != 0 || g != 0 {
            return Err(());
        }
        match op {
            0x4300 => {
                if role != 1 || b != 32 {
                    return Err(());
                }
                crate::device_guest::input::read(|v| copy(a, v, true)).map_err(|_| ())
            }
            0x4301 => {
                if role != 1 || b != 128 {
                    return Err(());
                }
                let mut bytes = [0; 128];
                copy(a, &mut bytes, false).map_err(|_| ())?;
                let w = |n| u64::from_le_bytes(bytes[n..n + 8].try_into().unwrap());
                let p = Peer {
                    process: w(0),
                    incarnation: w(8),
                    principal: w(16),
                    session: w(24),
                };
                if p.session != 1001 || !live(p) {
                    return Err(());
                }
                let index = match p.principal {
                    403 => 0,
                    404 => 1,
                    _ => return Err(()),
                };
                let e =
                    continuum_kernel::input_wire::Event::decode(&bytes[32..]).map_err(|_| ())?;
                let s = &mut unsafe { &mut *SLOTS.0.get() }[index];
                if s.full && e.kind != continuum_kernel::input_wire::CANCEL {
                    return Ok(0);
                } // bounded backpressure
                s.peer = Some(p);
                s.bytes.copy_from_slice(&bytes[32..]);
                s.full = true;
                #[cfg(feature = "input-test")]
                observe("m1.input_delivery", p, e);
                Ok(96)
            }
            0x4302 => {
                if b != 96 {
                    return Err(());
                }
                let i = match role {
                    3 => 0,
                    4 => 1,
                    _ => return Err(()),
                };
                let s = &mut unsafe { &mut *SLOTS.0.get() }[i];
                if !s.full {
                    return Ok(0);
                }
                if s.peer != Some(peer) {
                    return Err(());
                }
                copy(a, &mut s.bytes, true).map_err(|_| ())?;
                s.full = false;
                #[cfg(feature = "input-test")]
                observe(
                    "m1.input_received",
                    peer,
                    continuum_kernel::input_wire::Event::decode(&s.bytes).map_err(|_| ())?,
                );
                Ok(96)
            }
            _ => {
                if role != 1 || a != 0 || b != 0 {
                    return Err(());
                }
                crate::device_guest::input::stop().map_err(|_| ())?;
                crate::device_guest::input::initialize().map_err(|_| ())?;
                Ok(0)
            }
        }
    })();
    if r.is_err() {
        record(
            "m1.input_denied",
            "desktop",
            "ok",
            format_args!(
                "op={op} role={role} process={} authority_session={}",
                peer.process, peer.session
            ),
        );
    }
    Some(r)
}

#[cfg(feature = "input-test")]
fn observe(name: &str, p: Peer, e: continuum_kernel::input_wire::Event) {
    record(
        name,
        "desktop",
        "ok",
        format_args!(
            "process={} incarnation={} authority_session={} sequence={} ns={} epoch={} surface={} scene={} output={} focus={} kind={} code={} value={} x={} y={} modifiers={} text={}",
            p.process,
            p.incarnation,
            p.session,
            e.sequence,
            e.time,
            e.epoch,
            e.surface,
            e.scene,
            e.output,
            e.focus,
            e.kind,
            e.code,
            e.value,
            e.x,
            e.y,
            e.modifiers,
            e.text
        ),
    );
}
