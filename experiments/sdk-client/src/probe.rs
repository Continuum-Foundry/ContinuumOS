//! Every exposed syscall is exercised through the public register interface.
use super::*;
pub fn run() {
    for (version, features, expected) in [(1u64, 0u64, INCOMPATIBLEPROTOCOL), (0, 1, UNSUPPORTED)] {
        let e: u64;
        let v: u64;
        unsafe {
            asm!("syscall",inlateout("rax") ABI_QUERY=>e,in("rdi")0u64,in("rsi")0u64,inlateout("rdx")0u64=>v,in("r10")0u64,in("r8")features,in("r9")version,lateout("rcx")_,lateout("r11")_);
        }
        require((e, v) == (expected, 0));
    }

    let mut b = [0u8; 32];
    require(invoke(ABI_QUERY, b.as_mut_ptr() as u64, 31, 0, 0) == (BUFFERTOOSMALL, 32));
    require(ok(ABI_QUERY, b.as_mut_ptr() as u64, 32, 0, 0) == 32);
    sdk::wire::validate("discovery", &b).unwrap();
    require(invoke(999, 0, 0, 0, 0) == (UNSUPPORTED, 0));
    require(invoke(2, 4096, 16, 0, 0) == (UNSUPPORTED, 0));
    let p = word(3);
    let m = ok(MEMORY_CREATE, p, 4096, 32 | 64, 0);
    let rights = ok(OBJECT_INFO, m, 0, 0, 0) & 0xffffffff;
    require(rights & 32 != 0 && invoke(HANDLE_DUPLICATE, m, 4096, 0, 0).0 == ACCESSDENIED);
    let copy = ok(HANDLE_DUPLICATE, m, 1 | 4 | 16 | 32, 0, 0);
    let d = sdk::wire::map(0x62000000, 0, 4096, 1).unwrap();
    ok(MEMORY_MAP, p, m, d.as_ptr() as u64, 32);
    unsafe {
        (0x62000000 as *mut u8).write_volatile(7);
    }
    ok(MEMORY_PROTECT, p, 0x62000000, 0, 0);
    ok(HANDLE_CLOSE, m, 0, 0, 0);
    require(unsafe { (0x62000000 as *const u8).read_volatile() } == 7);
    ok(MEMORY_UNMAP, p, 0x62000000, 0, 0);
    ok(HANDLE_CLOSE, copy, 0, 0, 0);
    let e = ok(EVENT_CREATE, p, 0, 0, 0);
    require(invoke(EVENT_SET, e, 2, 0, 0).0 == INVALIDARGUMENT);
    ok(EVENT_SET, e, 1, 0, 0);
    let item = sdk::wire::wait_item(sdk::Handle(e), 0, 0, 0).unwrap();
    let wait = sdk::wire::words::<4, 32>([item.as_ptr() as u64, 1, now() + 1_000_000_000, 0]);
    require(ok(WAIT_MANY, wait.as_ptr() as u64, 32, 0, 0) == 0);
    let timer = ok(TIMER_CREATE, p, 0, 0, 0);
    let arm = sdk::wire::timer(sdk::MonotonicNs(now() + 1_000_000), 0).unwrap();
    ok(TIMER_ARM, timer, arm.as_ptr() as u64, 24, 0);
    wait_handle(timer, 0);
    ok(TIMER_CANCEL, timer, 0, 0, 0);
    ok(HANDLE_CLOSE, timer, 0, 0, 0);
    require(ok(CLOCK_RESOLUTION, 0, 0, 0, 0) > 0);
    let mut pair = [0u8; 16];
    ok(CHANNEL_CREATE, p, pair.as_mut_ptr() as u64, 16, 0);
    let pair = sdk::wire::read_words::<2>(&pair).unwrap();
    require(invoke(CHANNEL_SIZE, pair[0], 0, 0, 0).0 == WOULDBLOCK);
    let payload = b"public-sdk";
    let send = sdk::wire::words::<4, 32>([payload.as_ptr() as u64, payload.len() as u64, 0, 0]);
    let attempt = sdk::wire::words::<3, 24>([send.as_ptr() as u64, now() + 1_000_000_000, 0]);
    ok(CHANNEL_SEND_UNTIL, pair[0], attempt.as_ptr() as u64, 24, 0);
    require(ok(CHANNEL_SIZE, pair[1], 0, 0, 0) == payload.len() as u64);
    let mut out = [0u8; 32];
    let mut meta = [0u8; 64];
    let recv = sdk::wire::words::<6, 48>([
        out.as_mut_ptr() as u64,
        32,
        0,
        0,
        meta.as_mut_ptr() as u64,
        64,
    ]);
    let attempt = sdk::wire::words::<3, 24>([recv.as_ptr() as u64, now() + 1_000_000_000, 0]);
    ok(
        CHANNEL_RECEIVE_UNTIL,
        pair[1],
        attempt.as_ptr() as u64,
        24,
        0,
    );
    require(&out[..payload.len()] == payload);
    let attempt = sdk::wire::words::<3, 24>([recv.as_ptr() as u64, now(), 0]);
    require(
        invoke(
            CHANNEL_RECEIVE_UNTIL,
            pair[1],
            attempt.as_ptr() as u64,
            24,
            0,
        )
        .0 == TIMEDOUT,
    );
    let attempt = sdk::wire::words::<3, 24>([send.as_ptr() as u64, u64::MAX, e]);
    require(invoke(CHANNEL_SEND_UNTIL, pair[0], attempt.as_ptr() as u64, 24, 0).0 == CANCELLED);
    sdk::guest::send_until(
        sdk::Handle(pair[0]),
        payload,
        &[],
        sdk::wire::NEVER,
        sdk::Handle(0),
    )
    .unwrap();
    let received = sdk::guest::receive_until(
        sdk::Handle(pair[1]),
        &mut out,
        &mut [],
        sdk::wire::NEVER,
        sdk::Handle(0),
    )
    .unwrap();
    require(received.bytes == payload.len());
    for h in [pair[0], pair[1], e] {
        ok(HANDLE_CLOSE, h, 0, 0, 0);
        require(invoke(OBJECT_INFO, h, 0, 0, 0).0 == BADHANDLE);
    }
    require(live(identity(0)));
    yield_now();
    log(b"sdk-syscalls-all-supported");
}
