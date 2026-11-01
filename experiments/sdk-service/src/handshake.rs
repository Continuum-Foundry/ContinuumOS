//! Independent reference decoder and producer. Deliberately does not call the
//! production Header/words/attachment codecs. Only syscall numbers are shared.
use super::*;
fn u(b: &[u8], at: usize, width: usize) -> u64 {
    let mut n = 0;
    for i in 0..width {
        n |= u64::from(b[at + i]) << (8 * i);
    }
    n
}
fn put(b: &mut [u8], at: usize, width: usize, n: u64) {
    for i in 0..width {
        b[at + i] = (n >> (8 * i)) as u8;
    }
}
pub fn service() {
    if config(0) != 0 {
        return;
    }
    let path = config(if config(1) == 5 { 3 } else { 4 });
    for n in 1..=10 {
        wait_handle(path, 2);
        let mut b = [0; 64];
        let mut meta = [0; 64];
        let mut handle = [0; 8];
        let desc = encode([
            b.as_mut_ptr() as u64,
            64,
            handle.as_mut_ptr() as u64,
            1,
            meta.as_mut_ptr() as u64,
            64,
        ]);
        ok(CHANNEL_RECEIVE, path, desc.as_ptr() as u64, 48, 0);
        let who = identity(3);
        require(
            u(&meta, 0, 8) == 64
                && u(&meta, 16, 8) == who.process
                && u(&meta, 24, 8) == who.generation
                && u(&meta, 32, 8) == who.principal
                && u(&meta, 40, 8) == who.session
                && u(&meta, 48, 8) == 0
                && live(who),
        );
        let status = if u(&b, 0, 4) != 0x53444b
            || u(&b, 4, 2) != 1
            || u(&b, 12, 4) & !u64::from(cfg!(feature = "minor-1")) != 0
        {
            14
        } else if u(&b, 22, 2) != 0
            || u(&b, 16, 4) != 16
            || !(1..=3).contains(&u(&b, 8, 4))
            || u(&b, 20, 2) != u(&meta, 8, 8)
            || u(&b, 48, 8) != u(&b, 24, 8)
            || u(&b, 56, 8) != 0
        {
            4
        } else {
            0
        };
        require(u(&b, 24, 8) == n);
        if status == 0 && u(&b, 8, 4) == 3 {
            require(u(&meta, 8, 8) == 1);
            let h = u(&handle, 0, 8);
            let d = encode([0x61000000, 0, 4096, 0]);
            ok(MEMORY_MAP, word(3), h, d.as_ptr() as u64, 32);
            for i in 0..4096 {
                require(unsafe { (0x61000000 as *const u8).add(i).read_volatile() } == 0);
            }
            require(invoke(MEMORY_PROTECT, word(3), 0x61000000, 1, 0).0 == ACCESSDENIED);
            ok(MEMORY_UNMAP, word(3), 0x61000000, 0, 0);
            ok(HANDLE_CLOSE, h, 0, 0, 0);
            trace(format_args!(
                "sdk-transfer-{}-{}-attenuated",
                config(0),
                config(1)
            ));
        } else if u(&meta, 8, 8) == 1 {
            require(status != 0);
            ok(HANDLE_CLOSE, u(&handle, 0, 8), 0, 0, 0);
            trace(format_args!(
                "sdk-discard-{}-{}-rejected",
                config(0),
                config(1)
            ));
        } else {
            require(u(&meta, 8, 8) == 0);
        }
        let mut reply = [0; 64];
        put(&mut reply, 0, 4, 0x53444b);
        put(&mut reply, 4, 2, 1);
        put(&mut reply, 6, 2, u64::from(cfg!(feature = "minor-1")));
        put(&mut reply, 8, 4, 2);
        put(&mut reply, 16, 4, 16);
        put(&mut reply, 24, 8, n);
        put(&mut reply, 48, 8, status);
        let version = u64::from(cfg!(feature = "minor-1"));
        put(&mut reply, 56, 8, version | (version << 32));
        let d = encode([reply.as_ptr() as u64, 64, 0, 0]);
        loop {
            let (e, _) = invoke(CHANNEL_SEND, path, d.as_ptr() as u64, 32, 0);
            if e == WOULDBLOCK {
                yield_now();
                continue;
            }
            require(e == 0);
            break;
        }
    }
}
