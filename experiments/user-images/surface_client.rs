//! Independently linked clients. Requests are authored field-by-field, not cast structs.
use super::{check, pause, syscall, word};
fn call(op: u64, a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    unsafe { syscall(op, a, b, c, d) }
}
fn put32(b: &mut [u8], at: usize, n: u32) {
    b[at..at + 4].copy_from_slice(&n.to_le_bytes());
}
fn put64(b: &mut [u8], at: usize, n: u64) {
    b[at..at + 8].copy_from_slice(&n.to_le_bytes());
}
fn log(text: &[u8]) {
    check(
        call(
            super::DIAGNOSTIC_LOG,
            text.as_ptr() as u64,
            text.len() as u64,
            0,
            0,
        )
        .0 == 0,
    );
}
struct Client {
    sequence: u64,
    bytes: [u8; 4096],
}
impl Client {
    fn new() -> Self {
        Self {
            sequence: 0,
            bytes: [0; 4096],
        }
    }
    fn start(&mut self, kind: u16, surface: u64) {
        self.bytes.fill(0);
        self.bytes[..4].copy_from_slice(b"CDU0");
        put32(&mut self.bytes, 8, 128);
        self.bytes[12..14].copy_from_slice(&kind.to_le_bytes());
        self.sequence += 1;
        put64(&mut self.bytes, 16, self.sequence);
        put64(&mut self.bytes, 24, self.sequence + 100);
        put64(&mut self.bytes, 32, 1);
        put64(&mut self.bytes, 40, 1);
        put64(&mut self.bytes, 48, surface);
        put32(&mut self.bytes, 56, 64);
    }
    fn exchange(&mut self, len: usize) -> (u64, u64) {
        put32(&mut self.bytes, 8, len as u32);
        put32(&mut self.bytes, 56, len as u32 - 64);
        let (status, t) = call(0x4200, self.bytes.as_ptr() as u64, len as u64, 0, 0);
        check(status == 0);
        // Backpressure and source alias mutation cannot replace a queued request.
        check(call(0x4200, self.bytes.as_ptr() as u64, len as u64, 0, 0).0 == super::ACCESSDENIED);
        self.bytes.fill(0xa5);
        // A bad response destination cannot consume a completed reply.
        let bad = call(0x4203, t, 0, 4096, 0);
        check(bad == (0, 0) || bad.0 == super::ACCESSDENIED);
        loop {
            let (e, n) = call(0x4203, t, self.bytes.as_mut_ptr() as u64, 4096, 0);
            if e == super::ACCESSDENIED {
                return (99, 0);
            }
            check(e == 0);
            if n == 0 {
                pause();
                continue;
            }
            check(n == 24);
            let read = |n| u64::from_le_bytes(self.bytes[n..n + 8].try_into().unwrap());
            check(read(16) == self.sequence + 100 || read(16) == 0);
            return (read(0), read(8));
        }
    }
    fn create(&mut self) -> u64 {
        self.start(1, 0);
        put32(&mut self.bytes, 80, 160);
        put32(&mut self.bytes, 84, 120);
        put32(&mut self.bytes, 88, 656);
        put32(&mut self.bytes, 92, 1);
        put64(&mut self.bytes, 96, 656 * 120);
        let (e, s) = self.exchange(128);
        check(e == 0);
        s
    }
    fn acquire(&mut self, s: u64) -> u64 {
        self.start(2, s);
        let (e, l) = self.exchange(128);
        check(e == 0);
        l
    }
    fn commit(&mut self, s: u64, l: u64, g: u64, revision: u64) -> (u64, u64) {
        self.start(4, s);
        put64(&mut self.bytes, 64, l);
        put64(&mut self.bytes, 72, revision);
        put32(&mut self.bytes, 80, 160);
        put32(&mut self.bytes, 84, 120);
        put32(&mut self.bytes, 88, 656);
        put32(&mut self.bytes, 92, 1);
        put64(&mut self.bytes, 96, 656 * 120);
        put32(&mut self.bytes, 112, 160);
        put32(&mut self.bytes, 116, 120);
        put64(&mut self.bytes, 120, g);
        self.exchange(128)
    }
    fn geometry(&mut self, s: u64, x: i32, y: i32, z: i32, visible: bool, clip: [u32; 4]) {
        self.start(5, s);
        put32(&mut self.bytes, 80, x as u32);
        put32(&mut self.bytes, 84, y as u32);
        put32(&mut self.bytes, 88, z as u32);
        put32(&mut self.bytes, 92, visible as u32);
        for (i, v) in clip.into_iter().enumerate() {
            put32(&mut self.bytes, 104 + i * 4, v);
        }
        check(self.exchange(128).0 == 0);
    }
}
pub fn run(role: u64, scenario: u64) {
    #[cfg(user_a)]
    {
        check(role == 3);
        log(b"surface-client-a");
    }
    #[cfg(user_b)]
    {
        check(role == 4);
        log(b"surface-client-b");
    }
    // No client can acquire/capture/map actual scanout or impersonate the server.
    for op in [
        0x4100, 0x4101, 0x4102, 0x4103, 0x4104, 0x4201, 0x4202, 0x4204,
    ] {
        check(call(op, 0, 0, 0, 0).0 == super::ACCESSDENIED);
    }
    check(call(0x4200, 0, 128, 0, 0).0 == super::ACCESSDENIED);
    let mut c = Client::new();
    let s = c.create();
    let l = c.acquire(s);
    check(c.commit(s, l, 1, 1).0 == 1); // partial copy cannot publish
    let mut offset = 0;
    while offset < 656 * 120 {
        let n = (656 * 120 - offset).min(3968);
        c.start(3, s);
        put64(&mut c.bytes, 64, l);
        put64(&mut c.bytes, 72, offset as u64);
        for i in 0..n {
            let at = offset + i;
            let x = at % 656;
            let channel = x % 4;
            c.bytes[128 + i] = if x >= 640 {
                0xee
            } else if role == 3 {
                [180, 40, 20, 255][channel]
            } else {
                [20, 90, 60, 128][channel]
            };
        }
        check(c.exchange(128 + n) == (0, (offset + n) as u64));
        offset += n;
    }
    check(c.commit(s, l, 2, 1).0 == 3);
    check(c.commit(s, l, 1, 1) == (0, 1));
    c.geometry(
        s,
        if role == 3 { -10 } else { 80 },
        if role == 3 { 25 } else { 70 },
        if role == 3 { 1 } else { 2 },
        true,
        if role == 3 {
            [0, 0, 160, 120]
        } else {
            [8, 4, 144, 108]
        },
    );
    c.start(8, 0);
    check(c.exchange(128).0 == 0);
    #[cfg(input_runtime)]
    {
        input_client(&mut c, role, s);
        return;
    }
    // Snapshot ownership survives mutation of every original chunk and request.
    c.start(3, s);
    put64(&mut c.bytes, 64, l);
    c.bytes[128..132].copy_from_slice(&[0; 4]);
    check(c.exchange(132).0 == 4);
    c.start(6, s);
    put64(&mut c.bytes, 64, l);
    check(c.exchange(128).0 == 4);
    check(c.commit(s, l, 1, 2).0 == 4);
    // Other client's surface IDs cannot authorize access; all IDs share global namespace.
    let mut foreign = 0;
    for low in 1..=4 {
        let id = (s & !0xffff_ffff) | low;
        if id == s || id == l {
            continue;
        }
        c.start(2, id);
        let (e, _) = c.exchange(128);
        check(e == 2 || e == 3);
        foreign += 1;
    }
    check(foreign > 0);
    c.start(2, s);
    put64(&mut c.bytes, 40, 2);
    check(c.exchange(128).0 == 3);
    // Three leases are independent from global byte capacity. Uncommitted releases
    // return owned storage; double release must deny without changing the active frame.
    let a = c.acquire(s);
    let b = c.acquire(s);
    c.start(2, s);
    check(c.exchange(128).0 == 5);
    for lease in [a, b] {
        c.start(6, s);
        put64(&mut c.bytes, 64, lease);
        check(c.exchange(128).0 == 0);
        c.start(6, s);
        put64(&mut c.bytes, 64, lease);
        check(c.exchange(128).0 == 3);
    }
    if role == 4 {
        // Exercise the global byte ceiling, independently of per-surface lease slots.
        c.start(1, 0);
        put32(&mut c.bytes, 80, 1920);
        put32(&mut c.bytes, 84, 1080);
        put32(&mut c.bytes, 88, 8192);
        put32(&mut c.bytes, 92, 1);
        put64(&mut c.bytes, 96, 8_847_360);
        let (e, large) = c.exchange(128);
        check(e == 0);
        let one = c.acquire(large);
        let two = c.acquire(large);
        c.start(2, large);
        check(c.exchange(128).0 == 5);
        for lease in [one, two] {
            c.start(6, large);
            put64(&mut c.bytes, 64, lease);
            check(c.exchange(128).0 == 0);
        }
        c.start(7, large);
        check(c.exchange(128).0 == 0);
    }
    if role == 4 {
        match scenario {
            2 => c.geometry(s, 240, 160, 2, true, [8, 4, 144, 108]),
            3 => c.geometry(s, 80, 70, 2, false, [8, 4, 144, 108]),
            4 => {
                c.start(7, s);
                check(c.exchange(128).0 == 0);
            }
            5 => {
                c.start(5, s);
                put32(&mut c.bytes, 92, 1);
                put32(&mut c.bytes, 104, u32::MAX);
                put32(&mut c.bytes, 112, 2);
                check(c.exchange(128).0 == 1);
                c.start(4, s);
                put64(&mut c.bytes, 64, l);
                put64(&mut c.bytes, 72, 2);
                put32(&mut c.bytes, 80, 160);
                put32(&mut c.bytes, 84, 120);
                put32(&mut c.bytes, 88, 639);
                put32(&mut c.bytes, 92, 1);
                put64(&mut c.bytes, 96, 656 * 120);
                check(c.exchange(128).0 == 1);
            }
            _ => {}
        }
    }
    log(b"surface-client-negatives-passed");
    c.start(9, 0);
    let result = c.exchange(128).0;
    if scenario == 8 || (scenario == 7 && role == 3) {
        check(result == 99);
        log(b"surface-compositor-cancelled");
    } else {
        check(result == 0);
    }
    // Late source mutations after release cannot change displayed frame.
    c.bytes.fill(0x55);
    check(call(super::THREAD_YIELD, word(3), 0, 0, 0).0 == 0);
}

pub fn wrong_session() {
    #[cfg(input_runtime)]
    {
        for op in 0x4300..=0x4303 {
            check(call(op, 0, 0, 0, 0).0 == super::ACCESSDENIED);
        }
        log(b"input-wrong-session-denied");
        return;
    }
    let mut c = Client::new();
    c.start(8, 0);
    check(c.exchange(128).0 == 2);
    c.start(1, 0);
    put32(&mut c.bytes, 80, 1);
    put32(&mut c.bytes, 84, 1);
    put32(&mut c.bytes, 88, 4);
    put32(&mut c.bytes, 92, 1);
    put64(&mut c.bytes, 96, 4);
    check(c.exchange(128).0 == 2);
    log(b"surface-wrong-session-denied");
}

#[cfg(input_runtime)]
fn input_client(c: &mut Client, role: u64, s: u64) {
    // Injection and another surface's capture cannot bypass trusted endpoint roles.
    for op in [0x4300, 0x4301, 0x4303] {
        check(call(op, 0, 0, 0, 0).0 == super::ACCESSDENIED);
    }
    c.start(11, s);
    put64(&mut c.bytes, 64, 2);
    check(c.exchange(128).0 == 2);
    c.start(10, s);
    put64(&mut c.bytes, 64, 999);
    check(c.exchange(128).0 == 2);
    if role == 3 {
        c.start(10, s);
        put64(&mut c.bytes, 64, 2);
        check(c.exchange(128).0 == 0);
        log(b"input-inject-0");
    }
    let mut sequence = 0;
    let mut stage = 0;
    let mut pointer_stage = 0;
    let mut flooding = false;
    let mut overflow_done = false;
    let mut held_seen = false;
    loop {
        let mut bytes = [0u8; 96];
        let (e, n) = call(0x4302, bytes.as_mut_ptr() as u64, 96, 0, 0);
        check(e == 0);
        if n == 0 {
            pause();
            continue;
        }
        check(n == 96 && &bytes[..8] == b"CIN0\0\0\x60\0" && bytes[88..] == [0; 8]);
        let w = |n| u64::from_le_bytes(bytes[n..n + 8].try_into().unwrap());
        let d = |n| u32::from_le_bytes(bytes[n..n + 4].try_into().unwrap());
        check(w(8) > sequence && w(24) > 0 && w(32) == s);
        sequence = w(8);
        let kind = u16::from_le_bytes(bytes[64..66].try_into().unwrap());
        let code = u16::from_le_bytes(bytes[66..68].try_into().unwrap());
        let value = d(68);
        if role == 3 {
            let matches = match stage {
                0 => kind == 1 && code == 42 && value == 1,
                1 => kind == 1 && code == 54 && value == 1,
                2 => kind == 1 && code == 42 && value == 0,
                3 => kind == 2 && d(84) == 65 && value == 1,
                4 => kind == 2 && d(84) == 65 && value == 2,
                5 => kind == 1 && code == 30 && value == 0,
                6 => kind == 1 && code == 54 && value == 0,
                _ => false,
            };
            if matches {
                check(d(80) == if stage < 6 { 1 } else { 0 });
                stage += 1;
                if stage < 7 {
                    log(match stage {
                        1 => b"input-inject-1",
                        2 => b"input-inject-2",
                        3 => b"input-inject-3",
                        4 => b"input-inject-4",
                        5 => b"input-inject-5",
                        _ => b"input-inject-6",
                    });
                } else {
                    c.start(12, s);
                    put64(&mut c.bytes, 64, 2);
                    check(c.exchange(128).0 == 0);
                    log(b"input-flood-00");
                    flooding = true;
                    let (e, start) = call(super::CLOCK_NOW, 0, 0, 0, 0);
                    check(e == 0);
                    loop {
                        let (e, now) = call(super::CLOCK_NOW, 0, 0, 0, 0);
                        check(e == 0);
                        if now - start >= 2_000_000_000 {
                            break;
                        }
                        pause();
                    }
                }
            }
            if flooding && !overflow_done && kind == 5 && code == 1 {
                check(d(80) == 0);
                log(b"input-overflow-passed");
                loop {
                    c.start(10, s);
                    put64(&mut c.bytes, 64, 2);
                    let e = c.exchange(128).0;
                    if e == 0 {
                        break;
                    }
                    check(e == 4);
                    pause();
                }
                overflow_done = true;
                log(b"input-inject-7");
            }
            if overflow_done && !held_seen && kind == 1 && code == 42 && value == 1 {
                check(d(80) == 1);
                held_seen = true;
                c.start(9, 0);
                check(c.exchange(128).0 == 0);
            }
            if overflow_done && held_seen && kind == 5 && code == 6 {
                check(d(80) == 0);
                log(b"input-reset-held-key-passed");
                c.start(9, 0);
                check(c.exchange(128).0 == 0);
                log(b"input-client-passed");
                break;
            }
        } else if kind == 4 && code == 272 {
            match (pointer_stage, value) {
                (0, 1) => {
                    check(d(72) < 160 && d(76) < 120 && w(40) == 2);
                    c.start(11, s);
                    put64(&mut c.bytes, 64, 2);
                    check(c.exchange(128).0 == 0);
                    for low in 1..=4 {
                        let id = (s & !0xffff_ffff) | low;
                        if id != s {
                            c.start(11, id);
                            put64(&mut c.bytes, 64, 2);
                            check(c.exchange(128).0 == 2);
                        }
                    }
                    pointer_stage = 1;
                    log(b"input-inject-8");
                }
                (1, 0) => {
                    check(d(72) > 160 && d(76) > 120);
                    pointer_stage = 2;
                    log(b"input-client-pointer-passed");
                    log(b"input-inject-9");
                }
                (2, 1) => {
                    c.start(11, s);
                    put64(&mut c.bytes, 64, 2);
                    check(c.exchange(128).0 == 0);
                    c.geometry(s, 240, 160, 2, true, [8, 4, 144, 108]);
                    c.start(11, s);
                    put64(&mut c.bytes, 64, 2);
                    check(c.exchange(128).0 == 2);
                    pointer_stage = 3;
                    log(b"input-inject-10");
                }
                (3, 1) => {
                    check(w(40) == 3);
                    c.start(11, s);
                    put64(&mut c.bytes, 64, 3);
                    check(c.exchange(128).0 == 0);
                    log(b"input-owner-exiting");
                    super::exit(42);
                }
                _ => check(false),
            }
        }
    }
}
