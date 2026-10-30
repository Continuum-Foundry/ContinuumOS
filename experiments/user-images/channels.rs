#![no_std]
#![no_main]
mod runtime;
#[path = "../../kernel/src/ipc_wire.rs"]
mod wire;
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
use runtime::*;
const BASIC: u64 = 7;
const ENDPOINT: u64 = 7 | 512 | 1024 | 2048;
global_asm!(r#"
.section .text.fixture_entry
.global fixture_entry
.type fixture_entry,@function
fixture_entry:
    sub rsp,8
    call {main}
    ud2
.size fixture_entry,.-fixture_entry
"#,main=sym main);
fn word(i: usize) -> u64 {
    unsafe { (0x70000000 as *const u64).add(i).read_volatile() }
}
fn call4(op: u64, a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let e;
    let v;
    unsafe {
        asm!("syscall",inlateout("rax") op=>e,in("rdi") a,in("rsi") b,inlateout("rdx") c=>v,in("r10") d,in("r8") 0u64,in("r9") 0u64,lateout("rcx") _,lateout("r11") _);
    }
    (e, v)
}
fn done(code: u64) -> ! {
    call4(PROCESS_EXIT, code, word(3), 0, 0);
    unsafe {
        asm!("ud2", options(noreturn));
    }
}
fn check(v: bool) {
    if !v {
        log(b"channel-failed");
        done(99);
    }
}
fn ok(op: u64, a: u64, b: u64, c: u64) -> u64 {
    let (e, v) = call4(op, a, b, c, 0);
    check(e == 0);
    v
}
fn yield_now() {
    ok(THREAD_YIELD, word(4), 0, 0);
}
fn close(h: u64) {
    ok(HANDLE_CLOSE, h, 0, 0);
}
fn record<const N: usize>(w: [u64; N]) -> [u8; N_BYTES] {
    let mut out = [0; N_BYTES];
    for (i, v) in w.iter().enumerate() {
        out[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes());
    }
    out
}
const N_BYTES: usize = 48;
fn pair() -> [u64; 2] {
    let mut b = [0u8; 16];
    ok(CHANNEL_CREATE, word(3), b.as_mut_ptr() as u64, 16);
    [le(&b, 0), le(&b, 8)]
}
fn le(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}
fn send(h: u64, bytes: &[u8], attachments: &[(u64, u64, u32)]) -> (u64, u64) {
    let mut a = [0u8; 192];
    for (i, (h, r, m)) in attachments.iter().enumerate() {
        a[i * 24..i * 24 + 8].copy_from_slice(&h.to_le_bytes());
        a[i * 24 + 8..i * 24 + 16].copy_from_slice(&r.to_le_bytes());
        a[i * 24 + 16..i * 24 + 20].copy_from_slice(&m.to_le_bytes());
    }
    let d = record([
        bytes.as_ptr() as u64,
        bytes.len() as u64,
        a.as_ptr() as u64,
        attachments.len() as u64,
    ]);
    call4(CHANNEL_SEND, h, d.as_ptr() as u64, 32, 0)
}
fn receive(h: u64, bytes: &mut [u8], handles: &mut [u8], meta: &mut [u8]) -> (u64, u64) {
    let d = record([
        bytes.as_mut_ptr() as u64,
        bytes.len() as u64,
        handles.as_mut_ptr() as u64,
        (handles.len() / 8) as u64,
        meta.as_mut_ptr() as u64,
        meta.len() as u64,
    ]);
    call4(CHANNEL_RECEIVE, h, d.as_ptr() as u64, 48, 0)
}
fn wait(h: u64) -> ([u8; 4096], [u8; 64], [u8; 64]) {
    for _ in 0..256 {
        let mut b = [0; 4096];
        let mut a = [0; 64];
        let mut m = [0; 64];
        let (e, _) = receive(h, &mut b, &mut a, &mut m);
        if e == 0 {
            return (b, a, m);
        }
        check(e == WOULDBLOCK);
        yield_now();
    }
    done(98)
}
// Independent user encoder: explicit fields and fixed test payload, no casts.
fn envelope(op: u32, id: u64) -> [u8; 64] {
    let mut b = [0; 64];
    b[..4].copy_from_slice(&0x43504d33u32.to_le_bytes());
    b[4..6].copy_from_slice(&1u16.to_le_bytes());
    b[8..12].copy_from_slice(&op.to_le_bytes());
    b[16..20].copy_from_slice(&16u32.to_le_bytes());
    b[24..32].copy_from_slice(&id.to_le_bytes());
    b[48..56].copy_from_slice(&999u64.to_le_bytes());
    b[56..64].copy_from_slice(&777u64.to_le_bytes());
    b
}
fn validate(b: &[u8], m: &[u8], principal: u64, process: u64) {
    check(le(m, 0) == 64 && le(m, 16) == process && le(m, 24) > 0 && le(m, 32) == principal);
    check(wire::Envelope::decode(&b[..64], le(m, 8) as usize, 0x43504d33, 1, 0, 0).is_ok());
    check(b[..8] == [0x33, 0x4d, 0x50, 0x43, 1, 0, 0, 0]);
    check(le(b, 48) == 999 && le(b, 56) == 777);
}
fn local_negatives() {
    let [tx, rx] = pair();
    let mut b = [0; 4096];
    let mut hs = [0; 64];
    let mut m = [0; 64];
    check(receive(rx, &mut b, &mut hs, &mut m) == (WOULDBLOCK, 0));
    for _ in 0..8 {
        check(send(tx, &[], &[]) == (0, 0));
    }
    check(send(tx, &[], &[]) == (WOULDBLOCK, 0));
    for _ in 0..8 {
        check(receive(rx, &mut b, &mut hs, &mut m) == (0, 0));
    }
    for _ in 0..4 {
        check(send(tx, &b, &[]) == (0, 0));
    }
    check(send(tx, &[1], &[]) == (WOULDBLOCK, 0));
    for _ in 0..4 {
        check(receive(rx, &mut b, &mut hs, &mut m) == (0, 0));
    }
    check(send(tx, &[0; 4097], &[]) == (INVALIDARGUMENT, 0));
    check(send(tx, b"bad", &[(rx, BASIC, 0), (u64::MAX, BASIC, 0)]) == (BADHANDLE, 0));
    check(ok(OBJECT_INFO, rx, 0, 0) >> 32 == 4);
    check(send(tx, b"bad", &[(rx, u64::MAX, 0)]) == (ACCESSDENIED, 0));
    check(send(tx, b"bad", &[(rx, BASIC, 0), (rx, BASIC, 1)]) == (INVALIDARGUMENT, 0));
    let d = record([0xffff800000000000, 1, 0, 0]);
    check(call4(CHANNEL_SEND, tx, d.as_ptr() as u64, 32, 0) == (INVALIDARGUMENT, 0));
    check(send(tx, b"keep", &[(rx, BASIC, 1)]) == (0, 0));
    check(receive(rx, &mut [], &mut hs, &mut m) == (BUFFERTOOSMALL, 0));
    let bad = record([
        0xffff800000000000,
        4,
        hs.as_mut_ptr() as u64,
        8,
        m.as_mut_ptr() as u64,
        64,
    ]);
    check(call4(CHANNEL_RECEIVE, rx, bad.as_ptr() as u64, 48, 0) == (INVALIDARGUMENT, 0));
    // Fail at the handle and metadata copies after earlier output bytes changed.
    for (hp, mp) in [
        (0xffff800000000000, m.as_mut_ptr() as u64),
        (hs.as_mut_ptr() as u64, 0xffff800000000000),
    ] {
        let bad = record([b.as_mut_ptr() as u64, 4096, hp, 8, mp, 64]);
        check(call4(CHANNEL_RECEIVE, rx, bad.as_ptr() as u64, 48, 0) == (INVALIDARGUMENT, 0));
    }
    let mut fill = [0; 28];
    let mut count = 0;
    loop {
        let (e, v) = call4(HANDLE_DUPLICATE, tx, BASIC, 0, 0);
        if e == QUOTAEXCEEDED {
            break;
        }
        check(e == 0 && count < 28);
        fill[count] = v;
        count += 1;
    }
    check(receive(rx, &mut b, &mut hs, &mut m) == (QUOTAEXCEEDED, 0));
    count -= 1;
    close(fill[count]);
    check(receive(rx, &mut b, &mut hs, &mut m) == (0, 0));
    close(le(&hs, 0));
    for h in &fill[..count] {
        close(*h);
    }
    close(rx);
    check(send(tx, b"dead", &[]) == (PEERCLOSED, 0));
    close(tx);
    let [tx, rx] = pair();
    check(send(tx, b"cycle", &[(rx, ENDPOINT, 0)]) == (0, 0));
    close(tx);
    log(b"channel-negatives-pass");
}
fn main() -> ! {
    let role = word(2);
    let endpoint = word(5);
    let extra = word(6);
    let id = word(0);
    if role == 1 {
        local_negatives();
        let mut malformed = envelope(0, id);
        malformed[12] = 1;
        check(send(endpoint, &malformed, &[]) == (0, 0));
        let (b, _, m) = wait(endpoint);
        validate(&b, &m, 202, 1);
        check(u32::from_le_bytes(b[8..12].try_into().unwrap()) == INCOMPATIBLEPROTOCOL as u32);
        let memory = ok(MEMORY_CREATE, word(3), 4096, 32);
        let mut msg = envelope(1, id);
        msg[20..22].copy_from_slice(&2u16.to_le_bytes());
        check(
            send(
                endpoint,
                &msg,
                &[(extra, ENDPOINT, 0), (memory, BASIC | 16 | 32, 0)],
            ) == (0, 0),
        );
        check(call4(OBJECT_INFO, extra, 0, 0, 0) == (BADHANDLE, 0));
        check(call4(OBJECT_INFO, memory, 0, 0, 0) == (BADHANDLE, 0));
        let (b, _, m) = wait(endpoint);
        validate(&b, &m, 202, 1);
        let finalmemory = ok(MEMORY_CREATE, word(3), 4096, 32);
        let mut finalmsg = envelope(4, id);
        finalmsg[20..22].copy_from_slice(&1u16.to_le_bytes());
        check(send(endpoint, &finalmsg, &[(finalmemory, BASIC | 16 | 32, 0)]) == (0, 0));
        log(b"channel-client-pass");
        done(0);
    } else if role == 2 {
        let (bad, _, _) = wait(endpoint);
        check(
            wire::Envelope::decode(&bad[..64], 0, 0x43504d33, 1, 0, 0)
                == Err(wire::Error::Features),
        );
        let rejected = envelope(INCOMPATIBLEPROTOCOL as u32, id);
        check(send(endpoint, &rejected, &[]) == (0, 0));
        let (b, a, m) = wait(endpoint);
        validate(&b, &m, 101, 0);
        check(le(&m, 8) == 2);
        let delegated = le(&a, 0);
        let memory = le(&a, 8);
        check(ok(OBJECT_INFO, memory, 0, 0) == (3u64 << 32 | BASIC | 16 | 32));
        let mut request = [0; 32];
        for (i, v) in [0x50000000u64, 0, 4096, 0].iter().enumerate() {
            request[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes());
        }
        check(call4(MEMORY_MAP, word(3), memory, request.as_ptr() as u64, 32) == (0, 0));
        close(memory);
        check(unsafe { (0x50000000 as *const u64).read_volatile() } == 0);
        let msg = envelope(2, id);
        check(send(delegated, &msg, &[]) == (0, 0));
        let (b, _, m) = wait(delegated);
        validate(&b, &m, 101, 2);
        if id == 201 {
            let doomed = ok(MEMORY_CREATE, word(3), 4096, 32);
            let mut queued = envelope(5, id);
            queued[20..22].copy_from_slice(&1u16.to_le_bytes());
            check(send(delegated, &queued, &[(doomed, BASIC | 16 | 32, 0)]) == (0, 0));
            let mut closed = false;
            for _ in 0..256 {
                let (e, _) = call4(CHANNEL_SIZE, delegated, 0, 0, 0);
                if e == PEERCLOSED {
                    closed = true;
                    break;
                }
                check(e == WOULDBLOCK);
                yield_now();
            }
            check(closed);
            check(call4(OBJECT_INFO, doomed, 0, 0, 0) == (BADHANDLE, 0));
            log(b"channel-queued-peer-death-pass");
        }
        close(delegated);
        let reply = envelope(3, id);
        check(send(endpoint, &reply, &[]) == (0, 0));
        let mut peer_dead = false;
        for _ in 0..256 {
            let (e, _) = send(endpoint, &[], &[]);
            if e == PEERCLOSED {
                peer_dead = true;
                break;
            }
            check(e == 0 || e == WOULDBLOCK);
            yield_now();
        }
        check(peer_dead);
        let (b, a, m) = wait(endpoint);
        validate(&b, &m, 101, 0);
        check(le(&m, 8) == 1);
        check(ok(OBJECT_INFO, le(&a, 0), 0, 0) == (3u64 << 32 | BASIC | 16 | 32));
        close(le(&a, 0));
        log(b"channel-service-pass");
        done(0);
    } else {
        let (b, _, m) = wait(endpoint);
        validate(&b, &m, 202, 1);
        let reply = envelope(3, id);
        check(send(endpoint, &reply, &[]) == (0, 0));
        if id == 201 {
            let mut queued = false;
            for _ in 0..256 {
                let (e, v) = call4(CHANNEL_SIZE, endpoint, 0, 0, 0);
                if e == 0 {
                    check(v == (1u64 << 32 | 64));
                    queued = true;
                    break;
                }
                check(e == WOULDBLOCK);
                yield_now();
            }
            check(queued); // die with the accepted memory handle still in flight
        }
        log(b"channel-delegate-pass");
        done(0);
    }
}
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    done(97)
}
