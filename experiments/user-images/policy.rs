#![no_std]
#![no_main]
#[allow(dead_code)]
#[path = "../../kernel/src/policy.rs"]
mod policy;
mod runtime;
use core::{
    arch::{asm, global_asm},
    fmt::{self, Write},
    panic::PanicInfo,
};
use policy::{Broker, Identity, Request, Scope};
use runtime::*;
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
type Words = [u64; 20];
fn word(i: usize) -> u64 {
    unsafe { (0x70000000 as *const u64).add(i).read_volatile() }
}
fn config(i: usize) -> u64 {
    unsafe { (0x71000000 as *const u64).add(i).read_volatile() }
}
fn identity(actor: usize) -> Identity {
    let i = 16 + actor * 5;
    Identity {
        process: config(i),
        generation: config(i + 1),
        principal: config(i + 2),
        session: config(i + 3),
        service: config(i + 4) != 0,
    }
}
fn invoke(op: u64, a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    let e;
    let v;
    unsafe {
        asm!("syscall",inlateout("rax")op=>e,in("rdi")a,in("rsi")b,inlateout("rdx")c=>v,in("r10")d,in("r8")0u64,in("r9")0u64,lateout("rcx")_,lateout("r11")_);
    }
    (e, v)
}
fn done() -> ! {
    invoke(PROCESS_EXIT, 0, word(3), 0, 0);
    unsafe { asm!("ud2", options(noreturn)) }
}
fn require(v: bool) {
    if !v {
        log(b"policy-failed");
        invoke(PROCESS_EXIT, 99, word(3), 0, 0);
        unsafe { asm!("ud2", options(noreturn)) }
    }
}
fn ok(op: u64, a: u64, b: u64, c: u64, d: u64) -> u64 {
    let (e, v) = invoke(op, a, b, c, d);
    require(e == 0);
    v
}
fn yield_now() {
    ok(THREAD_YIELD, word(4), 0, 0, 0);
}
fn now() -> u64 {
    ok(CLOCK_NOW, 0, 0, 0, 0)
}
fn encode<const N: usize>(words: [u64; N]) -> [u8; 160] {
    let mut b = [0; 160];
    for (i, w) in words.iter().enumerate() {
        b[i * 8..i * 8 + 8].copy_from_slice(&w.to_le_bytes());
    }
    b
}
fn send(h: u64, w: Words) {
    let b = encode(w);
    let req = encode([b.as_ptr() as u64, 160, 0, 0]);
    loop {
        let (e, _) = invoke(CHANNEL_SEND, h, req.as_ptr() as u64, 32, 0);
        if e == WOULDBLOCK {
            yield_now();
            continue;
        }
        require(e == 0);
        break;
    }
}
fn wait_handle(handle: u64, interest: u64) {
    let item = encode([handle, interest, 0, 0]);
    let req = encode([item.as_ptr() as u64, 1, u64::MAX, 0]);
    let (e, _) = invoke(WAIT_MANY, req.as_ptr() as u64, 32, 0, 0);
    require(e == 0 || e == PEERCLOSED);
}
fn sleep_until(deadline: u64) {
    let timer = ok(TIMER_CREATE, word(3), 0, 0, 0);
    let arm = encode([deadline, 0, 0]);
    ok(TIMER_ARM, timer, arm.as_ptr() as u64, 24, 0);
    wait_handle(timer, 0);
    ok(HANDLE_CLOSE, timer, 0, 0, 0);
}
fn receive(h: u64) -> Option<(Words, Identity)> {
    wait_handle(h, 2);
    let mut b = [0u8; 160];
    let mut meta = [0u8; 64];
    let req = encode([
        b.as_mut_ptr() as u64,
        160,
        0,
        0,
        meta.as_mut_ptr() as u64,
        64,
    ]);
    loop {
        let (e, _) = invoke(CHANNEL_RECEIVE, h, req.as_ptr() as u64, 48, 0);
        if e == WOULDBLOCK {
            yield_now();
            continue;
        }
        if e == PEERCLOSED {
            return None;
        }
        require(e == 0);
        let m: [u64; 8] = core::array::from_fn(|i| {
            u64::from_le_bytes(meta[i * 8..i * 8 + 8].try_into().unwrap())
        });
        require(m[0] == 160 && m[1] == 0 && m[6] <= 1);
        return Some((
            core::array::from_fn(|i| u64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap())),
            Identity {
                process: m[2],
                generation: m[3],
                principal: m[4],
                session: m[5],
                service: m[6] != 0,
            },
        ));
    }
}
fn live(who: Identity) -> bool {
    let b = encode([
        who.process,
        who.generation,
        who.principal,
        who.session,
        u64::from(who.service),
    ]);
    ok(CONTEXT_LIVE, b.as_ptr() as u64, 40, 0, 0) == 1
}
struct Line {
    b: [u8; 96],
    n: usize,
}
impl Write for Line {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.n + s.len() > 96 {
            return Err(fmt::Error);
        }
        self.b[self.n..self.n + s.len()].copy_from_slice(s.as_bytes());
        self.n += s.len();
        Ok(())
    }
}
fn trace(args: fmt::Arguments) {
    let mut l = Line { b: [0; 96], n: 0 };
    require(l.write_fmt(args).is_ok());
    require(invoke(DIAGNOSTIC_LOG, l.b.as_ptr() as u64, l.n as u64, 0, 0).0 == 0);
}
fn scope(w: &Words) -> Scope {
    Scope {
        kind: w[4],
        resource: w[5],
        owner: w[6],
    }
}
fn request(w: &Words) -> Request {
    Request {
        scope: scope(w),
        operation: w[9],
        revision: w[10],
        epoch: w[11],
        deadline: w[12],
    }
}
fn gift() {
    let attach = encode([config(2), 1024, 1]); // duplicate SEND-only endpoint
    let payload = [17u8];
    let req = encode([payload.as_ptr() as u64, 1, attach.as_ptr() as u64, 1]);
    ok(CHANNEL_SEND, config(3), req.as_ptr() as u64, 32, 0);
}
fn get_gift() -> u64 {
    let mut payload = [0u8; 1];
    let mut h = [0u8; 8];
    let mut meta = [0u8; 64];
    let req = encode([
        payload.as_mut_ptr() as u64,
        1,
        h.as_mut_ptr() as u64,
        1,
        meta.as_mut_ptr() as u64,
        64,
    ]);
    ok(CHANNEL_RECEIVE, config(3), req.as_ptr() as u64, 48, 0);
    u64::from_le_bytes(h)
}
fn client() {
    // Direct local RW memory remains valid across broker invalidation/death.
    let h = ok(MEMORY_CREATE, word(3), 4096, 32 | 64, 0);
    let desc = encode([0x60000000, 0, 4096, 1]);
    ok(MEMORY_MAP, word(3), h, desc.as_ptr() as u64, 32);
    unsafe {
        (0x60000000 as *mut u64).write_volatile(0x12345678);
    }
    let mut delegated = 0;
    loop {
        let Some((job, sender)) = receive(config(2)) else {
            break;
        };
        require(sender == identity(0));
        if job[1] == 12 {
            gift();
        }
        if job[1] == 13 {
            delegated = get_gift();
        }
        if job[0] == 52 {
            let bytes = encode(job);
            let mut fingerprint = 14695981039346656037u64;
            for byte in bytes {
                fingerprint = (fingerprint ^ u64::from(byte)).wrapping_mul(1099511628211);
            }
            trace(format_args!("replay-{}-{}", config(0), fingerprint));
        }
        send(if job[1] == 13 { delegated } else { config(2) }, job);
        let (response, sender) = receive(config(2)).unwrap();
        require(sender == identity(0) && response[0] == job[0]);
        unsafe {
            require((0x60000000 as *const u64).read_volatile() == 0x12345678);
        }
    }
    unsafe {
        require((0x60000000 as *const u64).read_volatile() == 0x12345678);
    }
    trace(format_args!("direct-{}-{}-survived", config(0), config(1)));
    done()
}
fn broker() {
    let mut b = Broker::new(identity(1), 100 + config(0) * 100); // fresh trusted incarnation
    loop {
        let (mut w, admin) = receive(config(2)).unwrap();
        require(admin == identity(1));
        let actor = w[2] as usize;
        require((1..6).contains(&actor));
        if w[1] == 14 {
            trace(format_args!("broker-{}-closed", config(0)));
            unsafe { asm!("ud2", options(noreturn)) }
        }
        let caller = if actor == 1 {
            admin
        } else {
            send(config(2 + actor - 1), w);
            if w[18] != 0 {
                // Confirm actual queued request, then expire while still queued.
                let channel = config(2 + actor - 1);
                wait_handle(channel, 2);
                require(ok(CHANNEL_SIZE, channel, 0, 0, 0) == 160);
                let start = now();
                let expiry = w[18] - 20_000_000;
                require(start < expiry);
                sleep_until(w[18]);
                trace(format_args!(
                    "queued-{}-{}-{}-{}",
                    w[0],
                    start,
                    expiry,
                    now()
                ));
            }
            let incoming = if w[1] == 13 {
                config(3)
            } else {
                config(2 + actor - 1)
            };
            let (got, sender) = receive(incoming).unwrap();
            require(got == w);
            w = got;
            sender
        };
        let result = (|| {
            if w[19] != 0 {
                return Err(INCOMPATIBLEPROTOCOL);
            }
            if !live(caller) {
                return Err(policy::DENIED);
            }
            let n = now();
            if w[13] >= 6 {
                return Err(policy::INVALID);
            }
            match w[1] {
                1 => b.restore(caller).map(|_| 0),
                2 => {
                    let subject = identity(w[13] as usize);
                    if !live(subject) {
                        return Err(policy::DENIED);
                    }
                    b.assign(caller, subject, scope(&w), w[7], w[8], n)
                }
                3 | 13 => b.execute(caller, w[3], request(&w), n),
                4 => b.preview(caller, w[3], request(&w), n),
                5 => b.invalidate(caller).map(|_| 0),
                6 => b.expire_session(caller, w[8]).map(|_| 0),
                7 => {
                    let child = identity(w[13] as usize);
                    if !live(child) {
                        return Err(policy::DENIED);
                    }
                    b.delegate(caller, w[3], child, scope(&w), w[7], w[8], n)
                }
                8 => b.ticket(caller, w[3], identity(w[13] as usize), request(&w), n),
                9 => {
                    let origin = b.ticket_origin(w[17]).ok_or(policy::DENIED)?;
                    if !live(origin) {
                        return Err(policy::DENIED);
                    }
                    b.forwarded(caller, w[3], w[17], request(&w), n)
                }
                10 => {
                    let mut stale = caller;
                    stale.generation = w[14];
                    if live(stale) {
                        Ok(1)
                    } else {
                        Err(policy::DENIED)
                    }
                }
                12 => Ok(0),
                15 => b
                    .lease_session(caller, identity(w[13] as usize), w[8], n)
                    .map(|_| 0),
                _ => Err(policy::INVALID),
            }
        })();
        let (status, value) = match result {
            Ok(v) => (0, v),
            Err(e) => (e, 0),
        };
        trace(format_args!(
            "d-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}",
            config(0),
            w[0],
            actor,
            caller.process,
            caller.generation,
            w[1],
            status,
            value,
            b.epoch(),
            b.revision()
        ));
        trace(format_args!(
            "s-{}-{}-{}-{}-{}-{}-{}",
            config(0),
            w[0],
            w[4],
            w[5],
            w[6],
            w[9],
            w[3]
        ));
        let mut response = [0; 20];
        response[0] = w[0];
        response[1] = status;
        response[2] = value;
        response[3] = b.epoch();
        response[4] = b.revision();
        if actor != 1 {
            send(config(2 + actor - 1), response);
        }
        send(config(2), response);
    }
}
fn rpc(case: u64, actor: u64, command: u64, grant: u64, mut w: Words, expected: u64) -> Words {
    w[0] = case;
    w[1] = command;
    w[2] = actor;
    w[3] = grant;
    send(config(2), w);
    let (response, sender) = receive(config(2)).unwrap();
    require(sender == identity(0) && response[0] == case);
    if response[1] != expected {
        trace(format_args!("failed-{}-{}-{}", case, response[1], expected));
        require(false);
    }
    response
}
fn base(epoch: u64, revision: u64) -> Words {
    let mut w = [0; 20];
    w[4] = 1;
    w[5] = 1;
    w[6] = 101;
    w[8] = u64::MAX;
    w[9] = 1;
    w[10] = revision;
    w[11] = epoch;
    w[12] = u64::MAX;
    w[13] = 2;
    w
}
fn admin() {
    let mut epoch = 100 + config(0) * 100;
    let mut rev = 1;
    let expires = now() + 20_000_000_000;
    let mut w = base(epoch, rev);
    w[8] = expires;
    rpc(1, 2, 3, 1, w, 3); // fail closed before restoration
    rpc(2, 2, 1, 0, w, 3); // app cannot restore checkpoint authority
    rpc(3, 1, 1, 0, w, 0);
    for (case, subject) in [(60, 2), (61, 3), (62, 5)] {
        let mut lease = w;
        lease[13] = subject;
        rpc(case, 1, 15, 0, lease, 0);
    }
    w[7] = 1 | 2 | 8;
    let own = rpc(4, 1, 2, 0, w, 0)[2];
    rpc(5, 2, 3, own, w, 0);
    // Literal same 160-byte request in both incarnations; only IPC metadata changes.
    let old = base(100, 1);
    rpc(52, 2, 3, 1, old, if config(0) == 0 { 0 } else { 13 });
    rpc(6, 4, 3, own, w, 3); // same owner, second app
    rpc(7, 3, 3, own, w, 3); // cross owner
    let mut forged = w;
    forged[14] = 404;
    forged[15] = 4004;
    forged[16] = u64::MAX;
    forged[6] = 202;
    rpc(8, 4, 3, own, forged, 3); // payload role/owner/session never changes provenance
    rpc(9, 2, 2, 0, w, 3); // unauthorized role administration
    let mut bad = w;
    bad[5] = 99;
    rpc(10, 1, 2, 0, bad, 3); // broker ceiling
    let mut share = w;
    share[13] = 3;
    share[7] = 1;
    let shared = rpc(11, 1, 2, 0, share, 0)[2];
    rpc(12, 3, 3, shared, w, 0);
    let mut write = w;
    write[9] = 2;
    rpc(13, 3, 3, shared, write, 3);
    let mut wrong = w;
    wrong[5] = 2;
    wrong[6] = 202;
    rpc(14, 3, 3, shared, wrong, 3);
    let mut child = w;
    child[13] = 4;
    child[7] = 1;
    let narrowed = rpc(15, 2, 7, own, child, 0)[2];
    rpc(16, 4, 3, narrowed, w, 0);
    rpc(17, 4, 3, narrowed, write, 3);
    child[13] = 3;
    child[7] = 2;
    rpc(18, 4, 7, narrowed, child, 3);
    let mut service = w;
    service[13] = 5;
    service[7] = 1 | 2;
    let strong = rpc(19, 1, 2, 0, service, 0)[2];
    let mut ticket = w;
    ticket[13] = 5;
    let token = rpc(20, 3, 8, shared, ticket, 0)[2];
    let mut forward = w;
    forward[17] = token;
    rpc(21, 5, 9, strong, forward, 0);
    rpc(22, 5, 9, strong, forward, 3); // consumed ticket
    ticket[9] = 2;
    rpc(23, 3, 8, shared, ticket, 3); // weak caller, powerful service
    let token = rpc(24, 2, 8, own, ticket, 0)[2];
    forward[17] = token;
    rpc(25, 5, 9, strong, forward, 3); // request substitution
    service[7] = 1;
    let weak = rpc(26, 1, 2, 0, service, 0)[2];
    forward[9] = 2;
    rpc(27, 5, 9, weak, forward, 3);
    rpc(28, 5, 9, strong, forward, 0);
    rev += 1;
    w = base(epoch, rev);
    w[8] = expires;
    write = w;
    write[9] = 2;
    rpc(29, 2, 4, own, write, 0);
    rpc(30, 1, 5, 0, w, 0);
    epoch += 1;
    rpc(31, 2, 3, own, write, 3); // removed between preview and execution
    w = base(epoch, rev);
    w[8] = expires;
    w[7] = 1 | 2 | 8;
    let own = rpc(32, 1, 2, 0, w, 0)[2];
    let mut stale = w;
    stale[11] = epoch - 1;
    rpc(33, 2, 3, own, stale, 13);
    stale = w;
    stale[10] = rev - 1;
    rpc(34, 2, 3, own, stale, 16);
    stale = w;
    stale[12] = now();
    rpc(35, 2, 3, own, stale, 7);
    let mut lease = w;
    lease[7] = 1;
    lease[8] = now() + 500_000_000;
    let limited = rpc(36, 1, 2, 0, lease, 0)[2];
    let mut queued = w;
    queued[18] = lease[8] + 20_000_000;
    rpc(37, 2, 3, limited, queued, 7);
    // Synthetic privileged controls with an active ordinary session and forged dev/admin claims.
    for (case, kind) in [(41, 2), (42, 3)] {
        let mut p = w;
        p[4] = kind;
        p[9] = 4;
        p[14] = 404;
        p[15] = 4004;
        p[16] = u64::MAX;
        rpc(case, 4, 3, 0, p, 3);
    }
    let mut expired = w;
    expired[8] = identity(2).session;
    rpc(38, 1, 6, 0, expired, 0);
    rpc(39, 2, 3, own, w, 7);
    rpc(40, 1, 2, 0, w, 7); // no grants after session invalidation
    let mut disk = w;
    disk[4] = 2;
    disk[7] = 4;
    disk[9] = 4;
    disk[13] = 4;
    // A fresh separate session avoids relying on the expired A session.
    disk[13] = 3;
    let narrow = rpc(43, 1, 2, 0, disk, 0)[2];
    rpc(44, 3, 3, narrow, disk, 0);
    for (case, kind) in [(45, 3), (46, 4), (47, 5), (48, 6)] {
        let mut p = disk;
        p[4] = kind;
        rpc(case, 3, 3, narrow, p, 3);
    }
    let mut stale = w;
    stale[14] = if config(0) == 1 {
        identity(4).generation - 1
    } else {
        identity(4).generation + 1
    };
    rpc(49, 4, 10, 0, stale, 3);
    // Cross-channel delegation attributes the current sender, not creator A.
    rpc(50, 2, 12, 0, w, 0);
    rpc(51, 4, 13, own, w, 7);
    let mut bgrant = w;
    bgrant[13] = 3;
    bgrant[7] = 1;
    let bh = rpc(53, 1, 2, 0, bgrant, 0)[2];
    let mut lease = w;
    lease[13] = 3;
    lease[8] = now() + 500_000_000;
    rpc(54, 1, 15, 0, lease, 0);
    let mut queued = w;
    queued[18] = lease[8] + 20_000_000;
    rpc(55, 3, 3, bh, queued, 7);
    rpc(56, 1, 2, 0, bgrant, 7);
    let mut bad_version = w;
    bad_version[19] = 1;
    rpc(57, 1, 3, 0, bad_version, 14);
    let mut crash = [0; 20];
    crash[1] = 14;
    crash[2] = 1;
    send(config(2), crash);
    require(receive(config(2)).is_none());
    trace(format_args!("admin-{}-peer-closed", config(0)));
    done()
}
extern "C" fn main() {
    match config(1) {
        0 => broker(),
        1 => admin(),
        _ => client(),
    }
}
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    log(b"policy-panic");
    invoke(PROCESS_EXIT, 98, word(3), 0, 0);
    unsafe { asm!("ud2", options(noreturn)) }
}
