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
use policy::Identity;
#[allow(dead_code)]
#[path = "../../kernel/src/routes.rs"]
mod routes;
use routes::{Authority, Reply, Request};
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
        log(b"route-failed");
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

fn endpoint(actor: usize) -> u64 {
    require(actor > 0 && actor < 6);
    config(1 + actor)
}
fn request(w: Words) -> Request {
    Request {
        caller: Identity {
            process: w[14],
            generation: w[15],
            principal: w[16],
            session: w[17],
            service: w[18] != 0,
        },
        id: w[3],
        service: w[4],
        protocol: w[5],
        major: w[6],
        features: w[7],
        epoch: w[8],
        policy_epoch: w[9],
        grant: w[10],
        deadline: w[12],
        delta: w[13],
    }
}
fn response(mut w: Words, status: u64, revision: u64, value: u64) -> Words {
    w[1] = status;
    w[2] = revision;
    w[19] = value;
    w
}
fn stop_others(victim: usize) {
    for actor in 2..6 {
        if actor != victim && live(identity(actor)) {
            let mut w = [0; 20];
            w[1] = 90;
            send(endpoint(actor), w);
        }
    }
}
fn worker() {
    let role = config(1) as usize;
    // This independent direct RW mapping intentionally survives route/policy death.
    let h = ok(MEMORY_CREATE, word(3), 4096, 32 | 64, 0);
    let desc = encode([0x60000000, 0, 4096, 1]);
    ok(MEMORY_MAP, word(3), h, desc.as_ptr() as u64, 32);
    unsafe {
        (0x60000000 as *mut u64).write_volatile(0x12345678);
    }
    loop {
        let Some((w, from)) = receive(config(2)) else {
            break;
        };
        require(from == identity(0));
        if w[1] == 90 {
            break;
        }
        if w[1] == 91 {
            unsafe { asm!("ud2", options(noreturn)) }
        }
        if role == 3 {
            send(config(2), w);
            let Some((mut answer, from)) = receive(config(2)) else {
                break;
            };
            require(from == identity(0));
            if answer[1] == 80 {
                // retained direct endpoint, not the current route
                send(config(3), w);
                let (r, s) = receive(config(3)).unwrap();
                require(s == identity(4));
                answer = r;
            }
            require(answer[0] == w[0] && answer[3] == w[3]);
            send(config(2), answer);
        } else if role == 2 {
            let target = w[2] as usize;
            require(target == 4 || target == 5);
            let path = config(target - 1);
            send(path, w);
            if w[19] == 6 {
                unsafe { asm!("ud2", options(noreturn)) }
            }
            let Some((mut answer, from)) = receive(path) else {
                if !live(identity(0)) {
                    break;
                }
                send(config(2), response(w, PEERCLOSED, 0, 0));
                continue;
            };
            require(from == identity(target));
            if w[19] == 8 {
                unsafe { asm!("ud2", options(noreturn)) }
            }
            if w[0] == 29 {
                // Inject the wrong correlation in actual returned IPC bytes.
                let original = answer[3];
                answer[3] += 1;
                trace(format_args!(
                    "corrupt-{}-{}-{}-{}",
                    config(0),
                    w[0],
                    original,
                    answer[3]
                ));
            }
            send(config(2), answer);
        } else {
            let direct = w[2] == 3;
            let path = if direct { config(4) } else { config(3) };
            let (job, from) = receive(path).unwrap();
            require(from == identity(if direct { 3 } else { 2 }));
            if w[19] == 5 {
                unsafe { asm!("ud2", options(noreturn)) }
            }
            send(config(2), job);
            let Some((answer, from)) = receive(config(2)) else {
                break;
            };
            require(from == identity(0));
            if job[19] == 6 {
                let b = encode(answer);
                let desc = encode([b.as_ptr() as u64, 160, 0, 0]);
                require(invoke(CHANNEL_SEND, path, desc.as_ptr() as u64, 32, 0).0 == PEERCLOSED);
            } else {
                send(path, answer);
            }
        }
    }
    unsafe {
        require((0x60000000 as *const u64).read_volatile() == 0x12345678);
    }
    trace(format_args!("direct-{}-{}-survived", config(0), role));
    done()
}
fn commit_path(a: &mut Authority, w: Words, r: Request, mode: u64) -> (u64, u64, u64) {
    let target = match a.forward(r, now()) {
        Ok(i) => {
            if i == identity(4) {
                4
            } else {
                5
            }
        }
        Err(e) => {
            let x = a.reject(r, e, now()).unwrap();
            return (x.status, x.revision, x.value);
        }
    };
    let direct = mode == 26;
    if !direct && !live(identity(2)) {
        let reply = a.reject(r, PEERCLOSED, now()).unwrap();
        return (reply.status, reply.revision, reply.value);
    }
    let service = if direct { 4 } else { target };
    let mut job = w;
    job[2] = if direct { 3 } else { target as u64 };
    job[19] = if [5, 6, 8].contains(&mode) { mode } else { 0 };
    send(endpoint(service), job);
    if direct {
        let mut control = w;
        control[1] = 80;
        send(endpoint(3), control);
    } else {
        send(endpoint(2), job);
    }
    if mode == 5 {
        require(receive(endpoint(4)).is_none());
        let Some((reply, from)) = receive(endpoint(2)) else {
            panic!()
        };
        require(from == identity(2) && reply[1] == PEERCLOSED);
        a.fence(identity(1)).unwrap();
        let x = a.reject(r, PEERCLOSED, now()).unwrap();
        return (x.status, x.revision, x.value);
    }
    let (got, from) = receive(endpoint(service)).unwrap();
    require(got == job || (direct && got == w));
    require(from == identity(service));
    if mode == 6 {
        require(receive(endpoint(2)).is_none());
        a.fence(identity(1)).unwrap();
        let x = a.reject(r, PEERCLOSED, now()).unwrap();
        send(
            endpoint(service),
            response(w, x.status, x.revision, x.value),
        );
        return (x.status, x.revision, x.value);
    }
    if mode == 25 {
        a.fence(identity(1)).unwrap();
        state(a, 250);
        a.grant_writer(identity(1), identity(5), 3, now()).unwrap();
        state(a, 251);
        a.publish(identity(1), identity(5), a.epoch).unwrap();
        state(a, 252);
    }
    if mode == 31 {
        let start = now();
        require(start < r.deadline);
        sleep_until(r.deadline);
        trace(format_args!("expired-{}-{}-{}", start, r.deadline, now()));
    }
    if mode == 32 {
        a.policy.invalidate(identity(1)).unwrap();
    }
    if mode == 36 {
        a.policy
            .expire_session(identity(1), identity(3).session)
            .unwrap();
    }
    let result = if !live(from) || !live(r.caller) {
        a.reject(r, policy::DENIED, now())
    } else {
        a.commit(from, w[11], r, now())
    };
    let reply = result.unwrap();
    trace(format_args!(
        "commit-{}-{}-{}-{}-{}-{}-{}-{}",
        config(0),
        w[0],
        r.id,
        service,
        reply.status,
        a.epoch,
        a.revision,
        a.value
    ));
    if mode == 7 {
        state(a, w[0]);
        trace(format_args!(
            "lost-{}-{}-{}-{}",
            config(0),
            r.id,
            a.revision,
            a.value
        ));
        unsafe { asm!("ud2", options(noreturn)) }
    }
    let answer = response(w, reply.status, reply.revision, reply.value);
    send(endpoint(service), answer);
    if direct {
        return (reply.status, reply.revision, reply.value);
    }
    let returned = receive(endpoint(2));
    if mode == 8 {
        require(returned.is_none());
        return (PEERCLOSED, reply.revision, reply.value);
    }
    let (got, from) = returned.unwrap();
    require(from == identity(2));
    let decoded = Reply {
        request: request(got),
        status: got[1],
        revision: got[2],
        value: got[19],
    };
    let checked = a.reply(r.caller, r, decoded, now());
    trace(format_args!(
        "wire-{}-{}-{}-{}",
        config(0),
        w[0],
        r.id,
        decoded.request.id
    ));
    trace(format_args!(
        "reply-{}-{}-{}",
        config(0),
        w[0],
        checked.err().unwrap_or(0)
    ));
    (
        checked.err().unwrap_or(reply.status),
        reply.revision,
        reply.value,
    )
}
fn state(a: &Authority, case: u64) {
    let writer = match a.writer {
        Some(i) if i == identity(4) => 4,
        Some(i) if i == identity(5) => 5,
        _ => 0,
    };
    trace(format_args!(
        "state-{}-{}-{}-{}-{}-{}",
        config(0),
        case,
        a.epoch,
        writer,
        a.published.map(|p| p.1).unwrap_or(0),
        a.policy.epoch()
    ));
}
fn authority() {
    let mut a = Authority::new(identity(1), 100 + config(0) * 100);
    let session_until = now() + 50_000_000_000;
    loop {
        let (w, from) = receive(endpoint(1)).unwrap();
        require(from == identity(1));
        let op = w[1];
        let mut status = 0;
        let mut result = None;
        if op == 8 {
            stop_others(99);
            send(endpoint(1), response(w, 0, a.revision, a.value));
            done();
        }
        if op == 7 {
            state(&a, w[0]);
            trace(format_args!(
                "lost-{}-0-{}-{}",
                config(0),
                a.revision,
                a.value
            ));
            unsafe { asm!("ud2", options(noreturn)) }
        }
        let admin = if op == 9 {
            send(endpoint(3), w);
            let (got, caller) = receive(endpoint(3)).unwrap();
            require(got == w && caller == identity(3));
            caller
        } else {
            from
        };
        if op < 10 {
            let r = (|| match op {
                1 => {
                    a.policy.restore(admin)?;
                    let until = session_until;
                    for actor in [3, 4, 5] {
                        a.policy
                            .lease_session(admin, identity(actor), until, now())?;
                        a.policy.assign(
                            admin,
                            identity(actor),
                            routes::SCOPE,
                            policy::WRITE,
                            until,
                            now(),
                        )?;
                    }
                    Ok(())
                }
                2 | 9 => a.fence(admin).map(|_| ()),
                3 => {
                    let i = identity(w[2] as usize);
                    if !live(i) {
                        Err(policy::DENIED)
                    } else {
                        a.grant_writer(admin, i, w[11], now())
                    }
                }
                4 => a.publish(admin, identity(w[2] as usize), w[8]),
                _ => Err(policy::INVALID),
            })();
            status = r.err().unwrap_or(0);
            if op == 9 {
                send(endpoint(3), response(w, status, a.revision, a.value));
                let (ack, sender) = receive(endpoint(3)).unwrap();
                require(sender == admin && ack[0] == w[0] && ack[1] == status);
            }
        } else if op == 16 {
            let start = now();
            sleep_until(start + routes::RETENTION);
            trace(format_args!("retention-{}-{}", start, now()));
        } else {
            send(endpoint(3), w);
            let (got, caller) = receive(endpoint(3)).unwrap();
            require(got == w && caller == identity(3));
            let r = request(got);
            let outcome = if !live(caller) {
                Err(policy::DENIED)
            } else if op == 13 {
                a.cancel(caller, r, now()).map(Some)
            } else if op == 12 {
                Ok(None)
            } else {
                a.admit(caller, r, now())
            };
            let (s, rev, value) = match outcome {
                Err(e) => (e, a.revision, a.value),
                Ok(Some(reply)) => (reply.status, reply.revision, reply.value),
                Ok(None) if op == 10 => (0, a.revision, a.value),
                Ok(None) => commit_path(&mut a, w, r, if w[19] > 0 { w[19] } else { w[0] }),
            };
            status = s;
            result = Some((rev, value));
            if w[0] != 26 {
                send(endpoint(3), response(w, s, rev, value));
            }
            let (ack, from) = receive(endpoint(3)).unwrap();
            require(from == caller && ack[0] == w[0] && ack[3] == r.id && ack[1] == s);
        }
        state(&a, w[0]);
        trace(format_args!(
            "d-{}-{}-{}-{}-{}-{}-{}-{}",
            config(0),
            w[0],
            op,
            status,
            a.epoch,
            a.revision,
            a.value,
            a.queued()
        ));
        let (rev, value) = result.unwrap_or((a.revision, a.value));
        send(endpoint(1), response(w, status, rev, value));
    }
}
fn base(case: u64, op: u64, id: u64, epoch: u64, pe: u64) -> Words {
    let c = identity(3);
    let mut w = [0; 20];
    w[0] = case;
    w[1] = op;
    w[3] = id;
    w[4] = routes::SERVICE;
    w[5] = routes::PROTOCOL;
    w[6] = 1;
    w[8] = epoch;
    w[9] = pe;
    w[10] = 1;
    w[11] = if epoch % 100 == 0 { 2 } else { 3 };
    w[12] = u64::MAX;
    w[13] = 1;
    w[14] = c.process;
    w[15] = c.generation;
    w[16] = c.principal;
    w[17] = c.session;
    w[18] = u64::from(c.service);
    w
}
fn call(w: Words, expected: u64) -> Words {
    send(config(2), w);
    let (reply, from) = receive(config(2)).unwrap();
    require(from == identity(0) && reply[0] == w[0] && reply[1] == expected);
    reply
}
fn ctl(case: u64, op: u64, target: u64, epoch: u64, grant: u64, expected: u64) {
    let mut w = base(case, op, 0, epoch, epoch);
    w[2] = target;
    w[11] = grant;
    call(w, expected);
}
fn admin() {
    let wave = config(0);
    let e = 100 + wave * 100;
    call(base(1, 10, 1, e, e), 3); // fresh incarnation cannot restore serialized authority
    ctl(2, 1, 0, e, 0, 0);
    call(base(58, 10, 1, e, e), PEERCLOSED);
    ctl(3, 4, 4, e, 2, 13);
    ctl(4, 3, 4, e, 2, 0);
    call(base(59, 10, 1, e, e), PEERCLOSED);
    ctl(5, 4, 4, e, 2, 0);
    if wave > 0 {
        let mut stale = base(53, 10, 1, 100, 100);
        stale[14] = 3;
        stale[15] = 1;
        stale[16] = 101;
        stale[17] = 1001;
        stale[18] = 0;
        let bytes = encode(stale);
        let mut hash = 14695981039346656037u64;
        for b in bytes {
            hash = (hash ^ u64::from(b)).wrapping_mul(1099511628211);
        }
        trace(format_args!("replay-{}-{}", wave, hash));
        call(stale, 3);
        let baseline = call(base(10, 11, 1, e, e), 0);
        require(baseline[2] == 1 && baseline[19] == 1);
        match wave {
            1 => {}
            2 => ctl(20, 2, 0, e, 0, 0),
            3 | 4 => {
                ctl(20, 2, 0, e, 0, 0);
                ctl(21, 3, 5, e + 1, 3, 0);
                if wave == 4 {
                    ctl(22, 4, 5, e + 1, 3, 0);
                }
            }
            5 | 6 | 8 => {
                let mut w = base(40, 11, 2, e, e);
                w[19] = wave;
                call(w, PEERCLOSED);
                if wave == 8 {
                    let r = call(base(41, 10, 2, e, e), 0);
                    require(r[2] == 2 && r[19] == 2);
                } else {
                    ctl(42, 3, 5, e + 1, 3, 0);
                    ctl(43, 4, 5, e + 1, 3, 0);
                    call(
                        base(44, 11, 3, e + 1, e),
                        if wave == 6 { PEERCLOSED } else { 0 },
                    );
                }
                ctl(99, 8, 0, e, 0, 0);
                done();
            }
            7 => {
                let mut w = base(40, 11, 2, e, e);
                w[19] = 7;
                send(config(2), w);
                require(receive(config(2)).is_none());
                trace(format_args!("admin-{}-peer-closed", wave));
                done()
            }
            _ => panic!(),
        }
        let mut crash = [0; 20];
        crash[1] = 7;
        send(config(2), crash);
        require(receive(config(2)).is_none());
        trace(format_args!("admin-{}-peer-closed", wave));
        done()
    }
    ctl(6, 9, 0, e, 0, 3);
    let mut w = base(7, 11, 1, e, e);
    w[6] = 2;
    call(w, 14);
    let mut w = base(8, 11, 1, e, e);
    w[7] = 1;
    call(w, 14);
    let mut w = base(9, 11, 1, e, e);
    w[15] += 1;
    call(w, 3);
    let mut unknown = base(54, 10, 1, e, e);
    unknown[4] = 999;
    call(unknown, 14);
    call(base(10, 11, 1, e, e), 0);
    call(base(11, 10, 1, e, e), 0);
    call(base(57, 13, 1, e, e), 0);
    let mut w = base(12, 10, 1, e, e);
    w[13] = 2;
    call(w, 3);
    call(base(13, 11, 3, e, e), 0);
    call(base(14, 10, 2, e, e), 13);
    call(base(15, 10, 4, e - 1, e), 13);
    for id in 4..=7 {
        call(base(12 + id, 10, id, e, e), 0);
        if id == 4 {
            call(base(56, 10, id, e, e), 6);
        }
    }
    call(base(20, 10, 8, e, e), 11);
    call(base(21, 13, 4, e, e), 8);
    for (case, id) in [(22, 5), (23, 6), (24, 7)] {
        call(base(case, 12, id, e, e), 0);
    }
    call(base(25, 11, 8, e, e), 13);
    let mut old = base(26, 11, 9, e + 1, e);
    old[11] = 2;
    call(old, 3);
    call(base(51, 16, 0, e, e), 0);
    call(base(52, 10, 9, e + 1, e), 13);
    call(base(27, 11, 10, e + 1, e), 0);
    let mut w = base(28, 11, 11, e + 1, e);
    w[11] = 2;
    call(w, 3);
    call(base(29, 11, 12, e + 1, e), 13);
    let mut w = base(31, 11, 13, e + 1, e);
    w[12] = now() + 300_000_000;
    call(w, 7);
    call(base(32, 11, 14, e + 1, e), 3);
    ctl(33, 1, 0, e, 0, 0);
    call(base(34, 10, 15, e + 1, e), 13);
    call(base(35, 11, 15, e + 1, e + 1), 0);
    call(base(36, 11, 16, e + 1, e + 1), 7);
    call(base(37, 10, 17, e + 1, e + 1), 7);
    ctl(99, 8, 0, e, 0, 0);
    done()
}
extern "C" fn main() {
    match config(1) {
        0 => authority(),
        1 => admin(),
        _ => worker(),
    }
}
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    log(b"route-panic");
    invoke(PROCESS_EXIT, 98, word(3), 0, 0);
    unsafe { asm!("ud2", options(noreturn)) }
}
