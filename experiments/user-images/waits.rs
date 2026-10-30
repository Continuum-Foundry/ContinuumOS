#![no_std]
#![no_main]
mod runtime;
mod wait_api;
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
use runtime::*;
use wait_api::{Deadline, MonotonicNs};
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
fn invoke(op: u64, a: u64, b: u64, c: u64) -> (u64, u64) {
    let e;
    let v;
    unsafe {
        asm!("syscall",inlateout("rax")op=>e,in("rdi")a,in("rsi")b,inlateout("rdx")c=>v,in("r10")0u64,in("r8")0u64,in("r9")0u64,lateout("rcx")_,lateout("r11")_);
    }
    (e, v)
}
fn done(code: u64) -> ! {
    invoke(PROCESS_EXIT, code, word(3), 0);
    unsafe {
        asm!("ud2", options(noreturn));
    }
}
fn check(v: bool) {
    if !v {
        log(b"wait-failed");
        done(99);
    }
}
fn ok(op: u64, a: u64, b: u64, c: u64) -> u64 {
    let (e, v) = invoke(op, a, b, c);
    check(e == 0);
    v
}
fn now() -> u64 {
    ok(CLOCK_NOW, 0, 0, 0)
}
fn encode<const N: usize>(w: [u64; N], out: &mut [u8]) {
    for (i, v) in w.iter().enumerate() {
        out[i * 8..i * 8 + 8].copy_from_slice(&v.to_le_bytes());
    }
}
fn le(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}
fn wait1(h: u64, interest: u64, bytes: u64, deadline: u64, cancel: u64) -> (u64, u64) {
    let mut item = [0; 32];
    encode([h, interest, bytes, 0], &mut item);
    let mut request = [0; 32];
    encode([item.as_ptr() as u64, 1, deadline, cancel], &mut request);
    invoke(WAIT_MANY, request.as_ptr() as u64, 32, 0)
}
fn pause(ns: u64) {
    let timer = ok(TIMER_CREATE, word(3), 0, 0);
    let mut arm = [0; 24];
    encode([now() + ns, 0, 0], &mut arm);
    ok(TIMER_ARM, timer, arm.as_ptr() as u64, 24);
    check(wait1(timer, 0, 0, u64::MAX, 0) == (0, 0));
    ok(HANDLE_CLOSE, timer, 0, 0);
}
fn send(h: u64, payload: &[u8], handles: &[u64]) -> (u64, u64) {
    let mut attachments = [0; 48];
    for (i, h) in handles.iter().enumerate() {
        encode(
            [*h, 7 | 256 | 512, 1],
            &mut attachments[i * 24..i * 24 + 24],
        );
    }
    let mut d = [0; 32];
    encode(
        [
            payload.as_ptr() as u64,
            payload.len() as u64,
            attachments.as_ptr() as u64,
            handles.len() as u64,
        ],
        &mut d,
    );
    invoke(CHANNEL_SEND, h, d.as_ptr() as u64, 32)
}
fn receive(h: u64) -> ([u8; 16], [u8; 16]) {
    let mut bytes = [0; 16];
    let mut handles = [0; 16];
    let mut meta = [0; 64];
    let mut d = [0; 48];
    encode(
        [
            bytes.as_mut_ptr() as u64,
            16,
            handles.as_mut_ptr() as u64,
            2,
            meta.as_mut_ptr() as u64,
            64,
        ],
        &mut d,
    );
    check(
        wait_api::receive_until(
            h,
            &mut bytes,
            &mut handles,
            &mut meta,
            Deadline::At(MonotonicNs(now() + 1_000_000_000)),
            0,
        )
        .is_ok(),
    );
    (bytes, handles)
}
fn local_checks() {
    check(ok(CLOCK_RESOLUTION, 0, 0, 0) > 0);
    let e = ok(EVENT_CREATE, word(3), 1, 0);
    let not_cancel = ok(TIMER_CREATE, word(3), 0, 0);
    check(wait1(e, 0, 0, u64::MAX, not_cancel) == (WRONGTYPE, 0));
    ok(HANDLE_CLOSE, not_cancel, 0, 0);
    let mut invalid = [0; 32];
    encode([0, 9, u64::MAX, 0], &mut invalid);
    check(invoke(WAIT_MANY, invalid.as_ptr() as u64, 32, 0) == (INVALIDARGUMENT, 0));
    encode([0, 0, u64::MAX, 0], &mut invalid);
    check(invoke(WAIT_MANY, invalid.as_ptr() as u64, 32, 0) == (INVALIDARGUMENT, 0));
    let mut duplicates = [0; 64];
    encode([e, 0, 0, 0], &mut duplicates[..32]);
    encode([e, 0, 0, 0], &mut duplicates[32..]);
    encode([duplicates.as_ptr() as u64, 2, u64::MAX, 0], &mut invalid);
    check(invoke(WAIT_MANY, invalid.as_ptr() as u64, 32, 0) == (INVALIDARGUMENT, 0));
    check(wait1(e, 2, 0, u64::MAX, 0) == (WRONGTYPE, 0));
    check(wait1(e, 0, 0, u64::MAX, 0) == (0, 0));
    check(wait1(e, 0, 0, 0, 0) == (TIMEDOUT, 0));
    ok(EVENT_SET, e, 0, 0);
    check(wait1(e, 0, 0, now() + 10_000_000, 0) == (TIMEDOUT, 0));
    let cancel = ok(EVENT_CREATE, word(3), 1, 0);
    check(wait1(e, 0, 0, 0, cancel) == (CANCELLED, 0));
    check(invoke(EVENT_SET, e, 2, 0) == (INVALIDARGUMENT, 0));
    let readonly = ok(HANDLE_DUPLICATE, e, 7, 0);
    check(invoke(EVENT_SET, readonly, 1, 0) == (ACCESSDENIED, 0));
    ok(HANDLE_CLOSE, readonly, 0, 0);
    ok(HANDLE_CLOSE, e, 0, 0);
    check(wait1(e, 0, 0, u64::MAX, 0) == (BADHANDLE, 0));
    ok(HANDLE_CLOSE, cancel, 0, 0);
    log(b"wait-local-ok");
}
fn pair() -> [u64; 2] {
    let mut out = [0; 16];
    ok(CHANNEL_CREATE, word(3), out.as_mut_ptr() as u64, 16);
    [le(&out, 0), le(&out, 8)]
}
fn boundary_checks() {
    for phase in 0..3u64 {
        let e = ok(EVENT_CREATE, word(3), 0, 0);
        unsafe {
            (0x70000008 as *mut u64).write_volatile(700 + phase);
        }
        check(wait1(e, 0, 0, now() + 1_000_000_000, 0) == (0, 0));
        ok(HANDLE_CLOSE, e, 0, 0);
        let [tx, rx] = pair();
        unsafe {
            (0x70000038 as *mut u64).write_volatile(tx);
            (0x70000008 as *mut u64).write_volatile(710 + phase);
        }
        check(wait1(rx, 2, 0, now() + 1_000_000_000, 0) == (0, 0));
        let _ = receive(rx);
        ok(HANDLE_CLOSE, tx, 0, 0);
        ok(HANDLE_CLOSE, rx, 0, 0);
        let e = ok(EVENT_CREATE, word(3), 0, 0);
        unsafe {
            (0x70000008 as *mut u64).write_volatile(720 + phase);
        }
        check(wait1(e, 0, 0, now() + 1_000_000_000, 0) == (PEERCLOSED, 0));
    }
    log(b"wait-boundaries-ok");
    let event = ok(EVENT_CREATE, word(3), 0, 0);
    unsafe {
        (0x70000008 as *mut u64).write_volatile(752);
    }
    check(wait1(event, 0, 0, now() + 1_000_000_000, 0) == (0, 0));
    check(wait1(event, 0, 0, now() + 10_000_000, 0) == (TIMEDOUT, 0));
    ok(HANDLE_CLOSE, event, 0, 0);
    log(b"wait-pulse-reset-ok");
}
fn pressure(role: u64, endpoint: u64) {
    if role == 1 {
        for _ in 0..8 {
            check(send(endpoint, &[], &[]) == (0, 0));
        }
        let gift = ok(EVENT_CREATE, word(3), 0, 0);
        let mut attached = [0; 24];
        encode([gift, 7 | 256 | 512, 0], &mut attached);
        let deadline = wait_api::after(20_000_000).unwrap();
        check(wait_api::send_until(endpoint, b"gift", &attached, deadline, 0) == Err(TIMEDOUT));
        check(ok(OBJECT_INFO, gift, 0, 0) >> 32 == 5);
        let cancel = ok(EVENT_CREATE, word(3), 1, 0);
        check(
            wait_api::send_until(endpoint, b"gift", &attached, Deadline::Infinite, cancel)
                == Err(CANCELLED),
        );
        check(ok(OBJECT_INFO, gift, 0, 0) >> 32 == 5);
        check(
            wait_api::send_until(
                endpoint,
                b"gift",
                &attached,
                wait_api::after(1_000_000_000).unwrap(),
                0,
            )
            .is_ok(),
        );
        check(invoke(OBJECT_INFO, gift, 0, 0) == (BADHANDLE, 0));
        log(b"wait-pressure-sender-ok");
    } else if role == 2 {
        pause(200_000_000);
        for _ in 0..8 {
            let _ = receive(endpoint);
        }
        let (bytes, handles) = receive(endpoint);
        check(&bytes[..4] == b"gift" && ok(OBJECT_INFO, le(&handles, 0), 0, 0) >> 32 == 5);
        log(b"wait-pressure-receiver-ok");
    } else {
        pause(300_000_000);
        log(b"wait-pressure-control-ok");
    }
}
fn delegate(endpoint: u64, gift: u64) {
    let mut attached = [0; 24];
    encode([gift, 7 | 512 | 1024 | 2048, 0], &mut attached);
    let mut d = [0; 32];
    encode([0, 0, attached.as_ptr() as u64, 1], &mut d);
    check(invoke(CHANNEL_SEND, endpoint, d.as_ptr() as u64, 32) == (0, 0));
}
fn bounded_cycle(role: u64, endpoint: u64, three: bool) {
    if !three && role == 3 {
        pause(200_000_000);
        log(b"wait-cycle-control-ok");
        return;
    }
    let waiting_on;
    if !three {
        check(send(endpoint, b"request", &[]) == (0, 0));
        let _ = receive(endpoint);
        waiting_on = endpoint;
    } else if role == 1 {
        let [a, b] = pair();
        delegate(endpoint, a);
        delegate(word(6), b);
        check(send(endpoint, b"request", &[]) == (0, 0));
        let _ = receive(word(6));
        waiting_on = endpoint;
    } else if role == 2 {
        let (_, handles) = receive(endpoint);
        let next = le(&handles, 0);
        let _ = receive(endpoint);
        check(send(next, b"request", &[]) == (0, 0));
        waiting_on = next;
    } else {
        let (_, handles) = receive(endpoint);
        let previous = le(&handles, 0);
        let _ = receive(previous);
        check(send(endpoint, b"request", &[]) == (0, 0));
        waiting_on = endpoint;
    }
    let result = wait1(waiting_on, 2, 0, now() + 50_000_000, 0);
    check(result == (TIMEDOUT, 0) || result == (PEERCLOSED, 0));
    if three {
        log(b"wait-cycle-three-ended");
    } else {
        log(b"wait-cycle-two-ended");
    }
}
fn lifecycle(role: u64, endpoint: u64) {
    if role == 2 {
        let event = ok(EVENT_CREATE, word(3), 0, 0);
        log(b"wait-victim-ready");
        unsafe {
            (0x70000008 as *mut u64).write_volatile(900);
        }
        let _ = wait1(event, 0, 0, u64::MAX, 0);
        done(99);
    } else if role == 1 {
        check(wait1(endpoint, 2, 0, now() + 1_000_000_000, 0) == (PEERCLOSED, 0));
        log(b"wait-peer-death-ok");
    } else {
        pause(100_000_000);
        log(b"wait-healthy-control-ok");
    }
}
fn cancel_boundaries() {
    for phase in 0..3u64 {
        let event = ok(EVENT_CREATE, word(3), 0, 0);
        let cancel = ok(EVENT_CREATE, word(3), 0, 0);
        unsafe {
            (0x70000038 as *mut u64).write_volatile(cancel);
            (0x70000008 as *mut u64).write_volatile(730 + phase);
        }
        check(wait1(event, 0, 0, now() + 1_000_000_000, cancel) == (CANCELLED, 0));
        ok(HANDLE_CLOSE, event, 0, 0);
        ok(HANDLE_CLOSE, cancel, 0, 0);
    }
    let event = ok(EVENT_CREATE, word(3), 1, 0);
    let cancel = ok(EVENT_CREATE, word(3), 0, 0);
    unsafe {
        (0x70000038 as *mut u64).write_volatile(cancel);
        (0x70000008 as *mut u64).write_volatile(732);
    }
    check(wait1(event, 0, 0, u64::MAX, cancel) == (0, 0));
    ok(HANDLE_CLOSE, event, 0, 0);
    ok(HANDLE_CLOSE, cancel, 0, 0);
    let event = ok(EVENT_CREATE, word(3), 0, 0);
    let deadline = now() + 10_000_000;
    unsafe {
        (0x70000008 as *mut u64).write_volatile(741);
    }
    check(wait1(event, 0, 0, deadline, 0) == (TIMEDOUT, 0));
    ok(HANDLE_CLOSE, event, 0, 0);
    let timer = ok(TIMER_CREATE, word(3), 0, 0);
    let mut arm = [0; 24];
    encode([now() + 10_000_000, 5_000_000, 0], &mut arm);
    ok(TIMER_ARM, timer, arm.as_ptr() as u64, 24);
    pause(40_000_000);
    check(wait1(timer, 0, 0, u64::MAX, 0) == (0, 0));
    ok(TIMER_CANCEL, timer, 0, 0);
    check(wait1(timer, 0, 0, now() + 10_000_000, 0) == (TIMEDOUT, 0));
    encode([now() + 10_000_000, 0, 0], &mut arm);
    ok(TIMER_ARM, timer, arm.as_ptr() as u64, 24);
    check(wait1(timer, 0, 0, u64::MAX, 0) == (0, 0));
    ok(HANDLE_CLOSE, timer, 0, 0);
    log(b"wait-cancel-delivery-ok");
}
extern "C" fn main(_: u64) -> ! {
    local_checks();
    pause(500_000_000);
    let role = word(2);
    let endpoint = word(5);
    let wave = word(0) - 200;
    if wave == 2 || wave == 3 {
        bounded_cycle(role, endpoint, wave == 3);
        log(b"wait-role-ok");
        done(0);
    }
    if wave == 4 {
        lifecycle(role, endpoint);
        log(b"wait-role-ok");
        done(0);
    }
    if wave == 5 {
        if role == 1 {
            cancel_boundaries();
        } else {
            pause(150_000_000);
        }
        log(b"wait-role-ok");
        done(0);
    }
    if role == 1 && wave == 0 {
        boundary_checks();
    }
    if role == 1 {
        let event = ok(EVENT_CREATE, word(3), 0, 0);
        let cancel = ok(EVENT_CREATE, word(3), 0, 0);
        check(send(endpoint, b"watch", &[event, cancel]) == (0, 0));
        check(send(word(6), b"watch", &[event, cancel]) == (0, 0));
        let expected = if wave == 0 { 0 } else { CANCELLED };
        check(wait1(event, 0, 0, now() + 1_000_000_000, cancel) == (expected, 0));
    } else {
        let (_, handles) = receive(endpoint);
        let event = le(&handles, 0);
        let cancel = le(&handles, 8);
        if role == 3 {
            pause(20_000_000);
            ok(EVENT_SET, if wave == 0 { event } else { cancel }, 1, 0);
            if wave == 1 {
                ok(EVENT_SET, cancel, 0, 0);
                log(b"wait-pulse-cancel-ok");
            }
        } else {
            let expected = if wave == 0 { 0 } else { CANCELLED };
            check(wait1(event, 0, 0, now() + 1_000_000_000, cancel) == (expected, 0));
        }
    }
    if wave == 0 {
        pressure(role, endpoint);
    }
    log(b"wait-role-ok");
    done(0)
}
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    done(97)
}
