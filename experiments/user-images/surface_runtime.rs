//! User-mode broker endpoint loop. All policy and pixel composition lives here.
use super::{ALLOCATOR, check, pause, surface as s, syscall, word};
use core::alloc::{GlobalAlloc, Layout};
fn call(op: u64, a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    unsafe { syscall(op, a, b, c, d) }
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
fn reply(ticket: u64, status: u64, value: u64, request: u64) {
    let mut body = [0u8; 24];
    s::put64(&mut body, 0, status);
    s::put64(&mut body, 8, value);
    s::put64(&mut body, 16, request);
    let result = call(0x4202, ticket, body.as_ptr() as u64, 24, 0);
    // A client can die after receive. Its slot is cancelled and a late reply is dropped.
    check(result == (0, 24) || result.0 == super::ACCESSDENIED);
}
fn live(p: s::Peer) -> bool {
    call(0x4204, p.process, p.incarnation, p.principal, p.session) == (0, 1)
}
fn present(broker: &mut s::Broker, width: usize, height: usize, frame: &mut [u8], generation: u64) {
    let n = broker.begin_frame().unwrap_or_else(|_| {
        check(false);
        0
    });
    check(broker.render(width, height, frame).is_ok());
    for y in 0..height {
        check(
            call(
                0x4102,
                frame[y * width * 4..].as_ptr() as u64,
                (y as u64) << 32,
                width as u64,
                generation,
            ) == (0, width as u64),
        );
        // Each copy remains synchronous. Cooperate once per 16 rows rather than
        // introducing a scheduling opportunity on every scanline.
        if y % 16 == 15 || y + 1 == height {
            #[cfg(input_runtime)]
            super::input_runtime::tick(broker);
            check(call(super::THREAD_YIELD, word(3), 0, 0, 0).0 == 0);
        }
    }
    check(broker.end_frame(n).is_ok());
}
/// A dead producer cannot request a frame. Retire its private snapshots and
/// schedule the exposure repair ourselves; ordinary service always enables repair.
fn retire_dead(
    broker: &mut s::Broker,
    width: usize,
    height: usize,
    frame: &mut [u8],
    generation: u64,
    repair: bool,
) -> bool {
    let before = broker.inventory();
    broker.retain_live(live);
    let changed = before != broker.inventory();
    if changed && repair {
        present(broker, width, height, frame, generation);
    }
    changed
}
#[cfg(surface_probe)]
fn scenario() -> u64 {
    match env!("CONTINUUM_SURFACE_SCENARIO").as_bytes() {
        [b'1'] => 1,
        [b'2'] => 2,
        [b'3'] => 3,
        [b'4'] => 4,
        [b'5'] => 5,
        [b'6'] => 6,
        [b'7'] => 7,
        [b'8'] => 8,
        [b'9'] => 9,
        _ => 0,
    }
}
pub fn run(role: u64, mode: u64) {
    #[cfg(surface_probe)]
    {
        if role != 1 {
            if role == 4 && mode == 2 && (scenario() < 7 || scenario() == 9) {
                super::surface_client::wrong_session();
            } else if (role == 3 || role == 4) && mode == 1 {
                super::surface_client::run(role, scenario());
            } else if role != 3 && role != 4 {
                for op in 0x4200..=0x4204 {
                    check(call(op, 0, 0, 0, 0).0 == super::ACCESSDENIED);
                }
                log(b"surface-transport-denied");
            }
            return;
        }
    }
    #[cfg(not(surface_probe))]
    if role != 1 {
        return;
    }
    let (status, g) = call(0x4100, 0, 0, 0, 0);
    check(status == 0);
    let (status, size) = call(0x4101, 0, 0, 0, g);
    check(status == 0);
    let (w, h) = (size as u32 as usize, (size >> 32) as usize);
    let layout = Layout::from_size_align(w * h * 4, 4096).unwrap();
    let pointer = unsafe { ALLOCATOR.alloc(layout) };
    check(!pointer.is_null());
    // This private scratch is admitted once before processing any surface commit.
    let frame = unsafe { core::slice::from_raw_parts_mut(pointer, w * h * 4) };
    let mut broker = s::Broker::with_epoch(g).unwrap_or_else(|_| {
        check(false);
        s::Broker::new()
    });
    #[cfg(input_runtime)]
    super::input_runtime::initialize(w as u32, h as u32);
    #[cfg(input_runtime)]
    let mut copy_probe_done = false;
    let mut peers = [None; 2];
    let mut sequences = [0; 2];
    #[cfg(surface_probe)]
    let mut done = [None; 2];
    #[cfg(surface_probe)]
    let mut ready = [None; 2];
    #[cfg(surface_probe)]
    let mut finished = false;
    #[cfg(all(input_runtime, surface_probe))]
    let (mut stale_announced, mut owner_dead, mut reset_done, mut held_announced) =
        (false, false, false, false);
    loop {
        #[cfg(surface_probe)]
        let repair = !finished; // The terminal capture is fixed while fixture teardown finishes.
        #[cfg(not(surface_probe))]
        let repair = true;
        retire_dead(&mut broker, w, h, frame, g, repair);
        #[cfg(input_runtime)]
        super::input_runtime::tick(&broker);
        #[cfg(all(input_runtime, surface_probe))]
        {
            if super::input_runtime::pressure() {
                present(&mut broker, w, h, frame, g);
                log(b"input-render-pressure-passed");
            }
            let seen = super::input_runtime::seen(1);
            if seen >= 16 && !stale_announced {
                stale_announced = true;
                log(b"input-stale-release-passed");
                log(b"input-inject-11");
            }
            if !owner_dead && peers[1].is_some_and(|p| !live(p)) {
                owner_dead = true;
                log(b"input-owner-death-passed");
                log(b"input-inject-12");
            }
            if owner_dead && seen >= 22 && !held_announced {
                let target = broker
                    .input_windows()
                    .into_iter()
                    .flatten()
                    .find(|w| w.peer.principal == 403)
                    .unwrap();
                super::input_runtime::bind(target.peer, target.id, target.scene);
                held_announced = true;
                log(b"input-inject-13");
            }
        }

        #[cfg(surface_probe)]
        {
            #[cfg(input_runtime)]
            let probe = !copy_probe_done;
            #[cfg(not(input_runtime))]
            let probe = true;
            if probe {
                let result = call(0x4201, 0, 4144, 0, 0);
                check(result == (0, 0) || result.0 == super::ACCESSDENIED);
                #[cfg(input_runtime)]
                if result.0 == super::ACCESSDENIED {
                    copy_probe_done = true;
                }
            }
        }
        let mut packet = [0u8; 4144];
        let (status, n) = call(
            0x4201,
            packet.as_mut_ptr() as u64,
            packet.len() as u64,
            0,
            0,
        );
        check(status == 0);
        if n == 0 {
            pause();
            continue;
        }
        let ticket = s::u64_at(&packet, 0);
        let peer = s::Peer {
            process: s::u64_at(&packet, 8),
            incarnation: s::u64_at(&packet, 16),
            principal: s::u64_at(&packet, 24),
            session: s::u64_at(&packet, 32),
        };
        let length = s::u64_at(&packet, 40) as usize;
        check(n as usize == length + 48 && length <= 4096);
        let client = match peer.principal {
            403 => 0,
            404 => 1,
            _ => {
                reply(ticket, s::AUTHORITY, 0, 0);
                continue;
            }
        };
        if peers[client] != Some(peer) {
            peers[client] = Some(peer);
            sequences[client] = 0;
        }
        let m = match s::decode(&packet[48..48 + length]) {
            Ok(m) => m,
            Err(e) => {
                reply(ticket, e, 0, 0);
                continue;
            }
        };
        if !live(peer) || m.sequence <= sequences[client] {
            reply(ticket, s::STALE, 0, m.request);
            continue;
        }
        sequences[client] = m.sequence;
        if m.kind == 8 {
            if peer.session != 1001 {
                reply(ticket, s::AUTHORITY, 0, m.request);
                continue;
            }
            if m.resource != 1 || m.output != 1 || m.surface != 0 || m.body != [0; 64] {
                reply(ticket, s::INVALID, 0, m.request);
                continue;
            }
            #[cfg(surface_probe)]
            {
                check(ready[client].is_none());
                ready[client] = Some((ticket, m.request));
                if ready.iter().any(Option::is_none) {
                    continue;
                }
                present(&mut broker, w, h, frame, g);
                log(b"surface-initial-presented");
                for (t, r) in ready.into_iter().flatten() {
                    reply(t, 0, 0, r);
                }
            }
            #[cfg(not(surface_probe))]
            {
                present(&mut broker, w, h, frame, g);
                reply(ticket, 0, 0, m.request);
            }
            continue;
        }
        #[cfg(surface_probe)]
        if m.kind == 9 {
            #[cfg(input_runtime)]
            if !reset_done {
                check(owner_dead && held_announced);
                check(call(0x4303, 0, 0, 0, 0).0 == 0);
                super::input_runtime::reset();
                super::input_runtime::tick(&broker);
                reset_done = true;
                log(b"input-device-reset-passed");
                reply(ticket, 0, 0, m.request);
                continue;
            }
            check(done[client].is_none());
            done[client] = Some((ticket, m.request));
            #[cfg(input_runtime)]
            if done[0].is_none() || !owner_dead {
                continue;
            }
            #[cfg(not(input_runtime))]
            if done.iter().any(Option::is_none) {
                continue;
            }
            if scenario() == 6 {
                let (t, r) = done[1].take().unwrap();
                reply(t, 0, 0, r);
                while live(peers[1].unwrap()) {
                    pause();
                }
                check(retire_dead(&mut broker, w, h, frame, g, true));
                log(b"surface-owner-death-reclaimed");
            }
            if scenario() != 6 {
                present(&mut broker, w, h, frame, g);
            }
            log(b"surface-final-presented");
            if scenario() == 8 {
                loop {
                    core::hint::spin_loop();
                }
            }
            if scenario() == 7 {
                // One live client's reply remains queued at actual compositor panic.
                let (t, r) = done[1].take().unwrap();
                reply(t, 0, 0, r);
                panic!("synthetic compositor failure");
            }
            for (t, r) in done.into_iter().flatten() {
                reply(t, 0, 0, r);
            }
            #[cfg(input_runtime)]
            {
                broker.retain_live(|_| false);
                unsafe {
                    ALLOCATOR.dealloc(pointer, layout);
                }
                super::input_runtime::shutdown();
                log(b"input-inventory-zero");
                break;
            }
            finished = true;
            continue;
        }
        #[cfg(input_runtime)]
        let result = if m.kind >= 10 {
            super::input_runtime::request(peer, m)
        } else {
            broker.request(peer, m)
        };
        #[cfg(not(input_runtime))]
        let result = broker.request(peer, m);
        #[cfg(surface_probe)]
        let last = finished && peer.session == 2002;
        #[cfg(surface_probe)]
        if last {
            check(result == Err(s::AUTHORITY));
        }

        match result {
            Ok(value) => reply(ticket, 0, value, m.request),
            Err(e) => reply(ticket, e, 0, m.request),
        }
        #[cfg(surface_probe)]
        if last {
            broker.retain_live(|_| false);
            check(broker.inventory() == (0, 0, 0));
            unsafe {
                ALLOCATOR.dealloc(pointer, layout);
            }
            log(b"surface-inventory-zero");
            break;
        }
    }
}
