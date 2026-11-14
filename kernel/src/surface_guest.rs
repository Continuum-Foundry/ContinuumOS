//! Kernel-only endpoint copying for the user-mode surface broker. No surface policy here.
use crate::{Local, record};
use continuum_kernel::surface_mailbox::{BYTES, Mailbox, Peer};
use core::cell::UnsafeCell;
static MAIL: Local<Mailbox> = Local(UnsafeCell::new(Mailbox::EMPTY));
pub(crate) fn dead(peer: Peer, role: u64) {
    let m = unsafe { &mut *MAIL.0.get() };
    if role == 1 {
        m.reset();
    } else {
        m.dead(peer);
    }
    record(
        "m1.surface_transport_dead",
        "desktop",
        "ok",
        format_args!(
            "role={role} process={} incarnation={} queued={}",
            peer.process,
            peer.incarnation,
            m.live()
        ),
    );
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
    live: impl FnOnce(Peer) -> bool,
) -> Option<core::result::Result<u64, ()>> {
    if !(0x4200..=0x4204).contains(&op) {
        return None;
    }
    let result = (|| {
        let m = unsafe { &mut *MAIL.0.get() };
        let client = match role {
            3 => Some(0),
            4 => Some(1),
            _ => None,
        };
        match op {
            0x4200 => {
                let client = client.ok_or(())?;
                if b == 0 || b > BYTES as u64 || c != 0 || g != 0 {
                    return Err(());
                }
                let mut data = [0; BYTES];
                copy(a, &mut data[..b as usize], false).map_err(|_| ())?;
                let ticket = m.submit(client, peer, &data[..b as usize])?;
                if b >= 128 {
                    let word = |at| u64::from_le_bytes(data[at..at + 8].try_into().unwrap());
                    let kind = u16::from_le_bytes(data[12..14].try_into().unwrap());
                    record(
                        "m1.surface_request",
                        "desktop",
                        "ok",
                        format_args!(
                            "ticket={ticket} client={client} process={} incarnation={} principal={} authority_session={} kind={kind} sequence={} request={} resource={} output={} surface={} bytes={b}",
                            peer.process,
                            peer.incarnation,
                            peer.principal,
                            peer.session,
                            word(16),
                            word(24),
                            word(32),
                            word(40),
                            word(48)
                        ),
                    );
                }
                Ok(ticket)
            }
            0x4201 => {
                if role != 1 || b < (BYTES + 48) as u64 || c != 0 || g != 0 {
                    return Err(());
                }
                if let Some((client, peer, ticket, data)) = m.peek() {
                    let mut packet = [0; BYTES + 48];
                    for (i, v) in [
                        ticket,
                        peer.process,
                        peer.incarnation,
                        peer.principal,
                        peer.session,
                        data.len() as u64,
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        packet[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes());
                    }
                    packet[48..48 + data.len()].copy_from_slice(data);
                    copy(a, &mut packet[..48 + data.len()], true).map_err(|_| ())?;
                    let len = 48 + data.len();
                    m.reading(client);
                    Ok(len as u64)
                } else {
                    Ok(0)
                }
            }
            0x4202 => {
                if role != 1 || c > BYTES as u64 || g != 0 {
                    return Err(());
                }
                let mut data = [0; BYTES];
                copy(b, &mut data[..c as usize], false).map_err(|_| ())?;
                m.reply(a, &data[..c as usize])?;
                if c == 24 {
                    let word = |at| u64::from_le_bytes(data[at..at + 8].try_into().unwrap());
                    record(
                        "m1.surface_response",
                        "desktop",
                        "ok",
                        format_args!(
                            "ticket={a} process={} incarnation={} status={} value={} request={}",
                            peer.process,
                            peer.incarnation,
                            word(0),
                            word(8),
                            word(16)
                        ),
                    );
                }
                Ok(c)
            }
            0x4203 => {
                let client = client.ok_or(())?;
                if c < BYTES as u64 || g != 0 {
                    return Err(());
                }
                match m.response(client, peer, a)? {
                    Some(data) => {
                        let mut packet = [0; BYTES];
                        let len = data.len();
                        packet[..len].copy_from_slice(data);
                        copy(b, &mut packet[..len], true).map_err(|_| ())?;
                        m.consumed(client);
                        Ok(len as u64)
                    }
                    None => Ok(0),
                }
            }
            _ => {
                if role != 1 {
                    return Err(());
                }
                Ok(u64::from(live(Peer {
                    process: a,
                    incarnation: b,
                    principal: c,
                    session: g,
                })))
            }
        }
    })();
    if result.is_err() {
        record(
            "m1.surface_denied",
            "desktop",
            "ok",
            format_args!(
                "op={op} role={role} process={} incarnation={}",
                peer.process, peer.incarnation
            ),
        );
    }
    Some(result)
}
