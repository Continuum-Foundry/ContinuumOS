#![no_std]
#![no_main]

mod runtime;
extern crate continuum_rt as sdk;
use core::{
    arch::{asm, global_asm},
    fmt::{self, Write},
    panic::PanicInfo,
};
use sdk::wire::Identity;

mod handshake;
mod probe;
mod smoke;
use runtime::*;
global_asm!(r#"
.section .text.fixture_entry
.global fixture_entry
.type fixture_entry,@function
fixture_entry:
    sub rsp,8
    call {main}
    call {done}
    ud2
.size fixture_entry,.-fixture_entry
"#,main=sym main,done=sym done);
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
    unsafe { sdk::syscall(op, a, b, c, d) }
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
            sdk::wire::read_words(&b).unwrap(),
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

fn worker() {
    let h = ok(MEMORY_CREATE, word(3), 4096, 32 | 64, 0);
    let d = encode([0x60000000, 0, 4096, 1]);
    ok(MEMORY_MAP, word(3), h, d.as_ptr() as u64, 32);
    unsafe {
        (0x60000000 as *mut u64).write_volatile(0x12345678);
    }
    handshake::client();
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
        send(config(2), w);
        let Some((mut answer, from)) = receive(config(2)) else {
            break;
        };
        require(from == identity(0));
        if answer[1] == 80 {
            send(config(3), w);
            let (r, s) = receive(config(3)).unwrap();
            require(s == identity(4));
            answer = r;
        }
        require(answer[0] == w[0] && answer[3] == w[3]);
        send(config(2), answer);
    }
    require(unsafe { (0x60000000 as *const u64).read_volatile() } == 0x12345678);
    trace(format_args!("direct-{}-3-survived", config(0)));
    done()
}
fn base(case: u64, op: u64, id: u64, epoch: u64, pe: u64) -> Words {
    let c = identity(3);
    let mut w = [0; 20];
    w[0] = case;
    w[1] = op;
    w[3] = id;
    w[4] = 1;
    w[5] = 0x525445;
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
extern "C" fn main(startup: *const u8) {
    require(
        sdk::wire::Startup::decode(unsafe { core::slice::from_raw_parts(startup, 64) }).is_ok(),
    );
    if cfg!(feature = "runtime-smoke") {
        unsafe {
            sdk::allocator::initialize(sdk::Handle(word(3))).unwrap();
        }
        smoke::run(word(0) - 800);
        sdk::allocator::shutdown().unwrap();
        log(b"sdk-heap-unmapped");
        return;
    }
    match config(1) {
        1 => admin(),
        3 => worker(),
        _ => panic!(),
    }
}
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    if word(0) == 802 {
        log(b"sdk-panic-contained");
    } else {
        log(b"sdk-client-panic");
    }
    invoke(PROCESS_EXIT, 98, word(3), 0, 0);
    unsafe { asm!("ud2", options(noreturn)) }
}
