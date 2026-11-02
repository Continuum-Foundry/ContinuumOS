use super::*;
fn heap_exhaustion() {
    use core::alloc::{GlobalAlloc, Layout};
    unsafe {
        sdk::allocator::initialize(Handle(word(3))).unwrap();
    }
    let allocator = sdk::allocator::Allocator;
    let full = Layout::from_size_align(65536, 4096).unwrap();
    let small = Layout::from_size_align(64, 64).unwrap();
    let backing = unsafe { allocator.alloc(full) };
    require(!backing.is_null());
    require(unsafe { allocator.alloc(small) }.is_null());
    let error = sdk::allocator::last_error();
    require(error == QUOTAEXCEEDED);
    require(sdk::allocator::shutdown() == Err(WOULDBLOCK));
    unsafe {
        allocator.dealloc(backing, full);
    }
    let reused = unsafe { allocator.alloc(full) };
    require(reused == backing);
    unsafe {
        allocator.dealloc(reused, full);
    }
    require(sdk::allocator::live() == 0);
    sdk::allocator::shutdown().unwrap();
    trace(format_args!("oom-{}-{error}-65536-reused", config(0)));
}
fn pair() -> [u64; 2] {
    let mut b = [0; 16];
    ok(CHANNEL_CREATE, word(3), b.as_mut_ptr() as u64, 16, 0);
    sdk::wire::read_words(&b).unwrap()
}
fn close(h: u64) {
    sdk::guest::close(Handle(h)).unwrap();
}
fn exhaustion() {
    heap_exhaustion();
    require(invoke(MEMORY_CREATE, word(3), 65537, 32, 0).0 == INVALIDARGUMENT);
    let mut handles = [0; 32];
    let mut n = 0;
    loop {
        let (e, h) = invoke(EVENT_CREATE, word(3), 0, 0, 0);
        if e != 0 {
            require(e == QUOTAEXCEEDED);
            break;
        }
        require(n < 32);
        handles[n] = h;
        n += 1;
    }
    require(n > 16);
    for h in &handles[..n] {
        close(*h);
        require(invoke(OBJECT_INFO, *h, 0, 0, 0).0 == BADHANDLE);
    }
    let fresh = ok(EVENT_CREATE, word(3), 0, 0, 0);
    require(!handles[..n].contains(&fresh));
    close(fresh);
}
fn transfer(pressure: bool) {
    let [a, b] = pair();
    let m = ok(MEMORY_CREATE, word(3), 4096, 32, 0);
    let attachment = sdk::wire::attachment(Handle(m), 1 | 4 | 16 | 32, 0).unwrap();
    let bytes = [7u8; 256];
    let desc =
        sdk::wire::words::<4, 32>([bytes.as_ptr() as u64, 256, attachment.as_ptr() as u64, 1]);
    ok(CHANNEL_SEND, a, desc.as_ptr() as u64, 32, 0);
    require(invoke(OBJECT_INFO, m, 0, 0, 0).0 == BADHANDLE);
    if pressure {
        let mut fill = [0; 32];
        let mut n = 0;
        loop {
            let (e, h) = invoke(EVENT_CREATE, word(3), 0, 0, 0);
            if e != 0 {
                require(e == QUOTAEXCEEDED);
                break;
            }
            fill[n] = h;
            n += 1;
            require(n < 32);
        }
        let mut out = [0; 256];
        let mut h = [0; 8];
        let mut meta = [0; 64];
        let recv = sdk::wire::words::<6, 48>([
            out.as_mut_ptr() as u64,
            256,
            h.as_mut_ptr() as u64,
            1,
            meta.as_mut_ptr() as u64,
            64,
        ]);
        require(invoke(CHANNEL_RECEIVE, b, recv.as_ptr() as u64, 48, 0).0 == QUOTAEXCEEDED);
        require(ok(CHANNEL_SIZE, b, 0, 0, 0) == (1u64 << 32) | 256);
        close(fill[n - 1]);
        n -= 1;
        ok(CHANNEL_RECEIVE, b, recv.as_ptr() as u64, 48, 0);
        close(u64::from_le_bytes(h));
        for handle in &fill[..n] {
            close(*handle);
        }
        let desc = sdk::wire::words::<4, 32>([bytes.as_ptr() as u64, 256, 0, 0]);
        for _ in 0..8 {
            ok(CHANNEL_SEND, a, desc.as_ptr() as u64, 32, 0);
        }
        require(invoke(CHANNEL_SEND, a, desc.as_ptr() as u64, 32, 0).0 == WOULDBLOCK);
    }
    // Queued authority/bytes are reclaimed even if no receiver accepts them.
    close(b);
    close(a);
}
fn cancellation() {
    let event = ok(EVENT_CREATE, word(3), 0, 0, 0);
    let item = sdk::wire::wait_item(Handle(event), 0, 0, 0).unwrap();
    if config(0) % 20 == 2 {
        let request =
            sdk::wire::words::<4, 32>([item.as_ptr() as u64, 1, now() + 320_000_000, event]);
        let status = invoke(WAIT_MANY, request.as_ptr() as u64, 32, 0, 0).0;
        require(status == CANCELLED);
        trace(format_args!("cancelrace-{}-{status}", config(0)));
    }
    let cancel = ok(EVENT_CREATE, word(3), 1, 0, 0);
    for _ in 0..32 {
        let req = sdk::wire::words::<4, 32>([item.as_ptr() as u64, 1, now() + 320_000_000, cancel]);
        require(invoke(WAIT_MANY, req.as_ptr() as u64, 32, 0, 0).0 == CANCELLED);
    }
    let timer = ok(TIMER_CREATE, word(3), 0, 0, 0);
    let deadline = now() + 1_000_000;
    let arm = sdk::wire::timer(MonotonicNs(deadline), 0).unwrap();
    ok(TIMER_ARM, timer, arm.as_ptr() as u64, 24, 0);
    sdk::guest::wait(
        Handle(timer),
        0,
        0,
        0,
        MonotonicNs(now() + 320_000_000),
        Handle(0),
    )
    .unwrap();
    trace(format_args!("timerlat-{}-{deadline}-{}", config(0), now()));
    close(timer);
    close(event);
    close(cancel);
}
fn copies() {
    require(invoke(ABI_QUERY, 0x80000000, 32, 0, 0).0 == INVALIDARGUMENT);
    require(invoke(DIAGNOSTIC_LOG, u64::MAX - 3, 32, 0, 0).0 == INVALIDARGUMENT);
    let m = ok(MEMORY_CREATE, word(3), 4096, 32 | 64, 0);
    let desc = sdk::wire::map(0x62000000, 0, 4096, 1).unwrap();
    ok(MEMORY_MAP, word(3), m, desc.as_ptr() as u64, 32);
    require(invoke(MEMORY_MAP, word(3), m, desc.as_ptr() as u64, 32).0 == INVALIDARGUMENT);
    ok(MEMORY_PROTECT, word(3), 0x62000000, 0, 0);
    close(m);
    require(invoke(ABI_QUERY, 0x62000000, 32, 0, 0).0 == INVALIDARGUMENT);
    ok(MEMORY_UNMAP, word(3), 0x62000000, 0, 0);
}
pub fn run(wave: u64) {
    // Invalid authority remains BadHandle. Only the registered stress observer
    // uses this deterministic interleaving marker; it grants no public power.
    require(invoke(OBJECT_INFO, 0, 0, 0, 0).0 == BADHANDLE);
    if wave % 20 == 12 {
        let event = ok(EVENT_CREATE, word(3), 0, 0, 0);
        let _ = sdk::guest::wait(Handle(event), 0, 0, 0, MonotonicNs(u64::MAX), Handle(0));
        require(false);
    }
    match wave % 10 {
        0 => exhaustion(),
        1 => transfer(true),
        2 => cancellation(),
        3 => copies(),
        4 => {
            let until = now() + 10_000_000;
            while now() < until {
                core::hint::spin_loop();
            }
        }
        5 => transfer(false),
        6 => {
            copies();
            cancellation();
        }
        7 => {
            exhaustion();
            transfer(true);
        }
        8 => copies(),
        9 => {
            transfer(false);
            cancellation();
        }
        _ => unreachable!(),
    }
    trace(format_args!("fault-{wave}-{}-ok", wave % 10));
}
