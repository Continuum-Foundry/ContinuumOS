//! Public SDK adversarial fixture. Bootstrap addresses belong only to the harness.
#![no_std]
#![no_main]
use continuum_rt::{self as sdk, *};
use core::{
    arch::{asm, global_asm},
    fmt::{self, Write},
    panic::PanicInfo,
};
mod faults;
#[allow(dead_code)]
#[path = "../../sdk-service/src/policy.rs"]
mod policy;
mod rate_window;
#[allow(dead_code)]
#[path = "../../sdk-service/src/routes.rs"]
mod routes;
mod supervision;
global_asm!(r#"
.section .text.fixture_entry
.global fixture_entry
.type fixture_entry,@function
fixture_entry:
    sub rsp,8
    call {main}
    ud2
.size fixture_entry,.-fixture_entry
"#, main=sym main);
fn word(i: usize) -> u64 {
    unsafe { (0x70000000 as *const u64).add(i).read_volatile() }
}
fn config(i: usize) -> u64 {
    unsafe { (0x71000000 as *const u64).add(i).read_volatile() }
}
fn invoke(op: u64, a: u64, b: u64, c: u64, d: u64) -> (u64, u64) {
    unsafe { sdk::syscall(op, a, b, c, d) }
}
fn require(v: bool) {
    if !v {
        let _ = sdk::log(b"stress-failed");
        sdk::exit(99, Handle(word(3)));
    }
}
fn ok(op: u64, a: u64, b: u64, c: u64, d: u64) -> u64 {
    let (e, v) = invoke(op, a, b, c, d);
    if e != 0 {
        trace(format_args!("syscall-error-{op}-{e}"));
    }
    require(e == 0);
    v
}
fn now() -> u64 {
    sdk::guest::now().unwrap().0
}
fn yield_now() {
    sdk::guest::yield_now(Handle(word(4))).unwrap();
}
fn progress() {
    unsafe {
        let p = 0x70000008 as *mut u64;
        p.write_volatile(p.read_volatile() + 1);
    }
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
fn trace(args: fmt::Arguments<'_>) {
    let mut l = Line { b: [0; 96], n: 0 };
    l.write_fmt(args).unwrap();
    sdk::log(&l.b[..l.n]).unwrap();
}
fn base62(line: &mut Line, values: &[u32]) {
    const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    for value in values {
        let mut value = *value;
        let mut digits = [0u8; 5];
        for at in (0..5).rev() {
            digits[at] = ALPHABET[(value % 62) as usize];
            value /= 62;
        }
        require(value == 0);
        line.write_str(core::str::from_utf8(&digits).unwrap())
            .unwrap();
    }
}
fn identity(at: usize) -> policy::Identity {
    policy::Identity {
        process: config(at),
        generation: config(at + 1),
        principal: config(at + 2),
        session: config(at + 3),
        service: config(at + 4) == 1,
    }
}
fn send(h: u64, words: [u64; 4]) {
    let b = sdk::wire::words::<4, 32>(words);
    sdk::guest::send_until(
        Handle(h),
        &b,
        &[],
        MonotonicNs(now() + 320_000_000),
        Handle(0),
    )
    .unwrap();
}
fn receive(h: u64) -> ([u64; 4], sdk::wire::Identity) {
    let mut b = [0; 32];
    let m = sdk::guest::receive_until(
        Handle(h),
        &mut b,
        &mut [],
        MonotonicNs(now() + 320_000_000),
        Handle(0),
    )
    .unwrap();
    require(m.bytes == 32 && m.handles == 0);
    (sdk::wire::read_words(&b).unwrap(), m.sender)
}
fn raw_send(h: u64, words: [u64; 4]) {
    let bytes = sdk::wire::words::<4, 32>(words);
    let descriptor = sdk::wire::words::<4, 32>([bytes.as_ptr() as u64, 32, 0, 0]);
    let mut deadline = None;
    loop {
        let (status, _) = invoke(CHANNEL_SEND, h, descriptor.as_ptr() as u64, 32, 0);
        if status == WOULDBLOCK {
            let at = now();
            let until = *deadline.get_or_insert(at + 320_000_000);
            require(at < until);
            yield_now();
            continue;
        }
        require(status == 0);
        return;
    }
}
fn raw_receive(
    h: u64,
    bytes: &mut [u8; 32],
    handles: &mut [[u8; 8]],
) -> Result<sdk::guest::Received, u64> {
    let mut metadata = [0u8; 64];
    let descriptor = sdk::wire::words::<6, 48>([
        bytes.as_mut_ptr() as u64,
        32,
        handles.as_mut_ptr() as u64,
        handles.len() as u64,
        metadata.as_mut_ptr() as u64,
        64,
    ]);
    let mut deadline = None;
    loop {
        let (status, _) = invoke(CHANNEL_RECEIVE, h, descriptor.as_ptr() as u64, 48, 0);
        if status == WOULDBLOCK {
            let at = now();
            let until = *deadline.get_or_insert(at + 320_000_000);
            require(at < until);
            yield_now();
            continue;
        }
        if status != 0 {
            return Err(status);
        }
        sdk::wire::validate("metadata", &metadata).unwrap();
        let w = sdk::wire::read_words::<8>(&metadata).unwrap();
        return Ok(sdk::guest::Received {
            bytes: w[0] as usize,
            handles: w[1] as usize,
            sender: sdk::wire::Identity {
                process: w[2],
                generation: w[3],
                principal: w[4],
                session: w[5],
                service: w[6] != 0,
            },
        });
    }
}
fn raw_reply(h: u64) -> ([u64; 4], sdk::wire::Identity) {
    let mut bytes = [0u8; 32];
    let received = raw_receive(h, &mut bytes, &mut []).unwrap();
    require(received.bytes == 32 && received.handles == 0);
    (sdk::wire::read_words(&bytes).unwrap(), received.sender)
}

fn live(i: policy::Identity) -> bool {
    let b = sdk::wire::words::<5, 40>([
        i.process,
        i.generation,
        i.principal,
        i.session,
        u64::from(i.service),
    ]);
    ok(CONTEXT_LIVE, b.as_ptr() as u64, 40, 0, 0) == 1
}
fn route_state(out: &mut [[u8; 4]; 7], a: &routes::Authority, step: usize, status: u64) {
    for (field, value) in [status, a.revision, a.value, a.queued() as u64]
        .into_iter()
        .enumerate()
    {
        out[step][field] = u8::try_from(value).unwrap();
    }
}
fn route_cycle(caller: policy::Identity) -> u64 {
    let mut observations = [[0u8; 4]; 7];
    require(live(caller));
    let admin = identity(16);
    let writer = admin;
    require(live(writer));
    let epoch = 100 + config(0) * 2;
    let until = now() + 320_000_000;
    let mut a = routes::Authority::new(admin, epoch);
    a.policy.restore(admin).unwrap();
    a.policy.lease_session(admin, caller, until, now()).unwrap();
    a.policy.lease_session(admin, writer, until, now()).unwrap();
    let grant = a
        .policy
        .assign(admin, caller, routes::SCOPE, policy::WRITE, until, now())
        .unwrap();
    let service = a
        .policy
        .assign(admin, writer, routes::SCOPE, policy::WRITE, until, now())
        .unwrap();
    a.grant_writer(admin, writer, service, now()).unwrap();
    a.publish(admin, writer, epoch).unwrap();
    let request = routes::Request {
        caller,
        id: 1,
        service: routes::SERVICE,
        protocol: routes::PROTOCOL,
        major: 1,
        features: 0,
        epoch,
        policy_epoch: epoch,
        grant,
        deadline: until,
        delta: 7,
    };
    require(a.admit(caller, request, now()).unwrap().is_none());
    require(a.forward(request, now()).unwrap() == writer);
    let committed = a.commit(writer, service, request, now()).unwrap();
    require(committed.status == 0 && committed.revision == 1 && committed.value == 7);
    route_state(&mut observations, &a, 0, committed.status);
    let cancelled = a.cancel(caller, request, now()).unwrap();
    require(cancelled == committed);
    route_state(&mut observations, &a, 1, cancelled.status);
    let queued = routes::Request { id: 2, ..request };
    a.admit(caller, queued, now()).unwrap();
    a.forward(queued, now()).unwrap();
    a.fence(admin).unwrap();
    let stale = a.commit(writer, service, queued, now()).unwrap();
    require(stale.status == policy::STALE && a.value == 7);
    route_state(&mut observations, &a, 2, stale.status);
    let denied = a
        .admit(caller, routes::Request { id: 3, ..request }, now())
        .unwrap_err();
    require(denied == policy::STALE);
    route_state(&mut observations, &a, 3, denied);
    a.grant_writer(admin, writer, service, now()).unwrap();
    a.publish(admin, writer, a.epoch).unwrap();
    let queued = routes::Request {
        id: 3,
        epoch: a.epoch,
        ..request
    };
    require(a.admit(caller, queued, now()).unwrap().is_none());
    a.forward(queued, now()).unwrap();
    // Attenuate an existing finite session only after work is queued. The
    // fixed one-millisecond elapsed fence cannot race initial admission.
    let sampled = now();
    let expiry = sampled + 1_000_000;
    a.policy
        .lease_session(admin, caller, expiry, sampled)
        .unwrap();
    while now() < expiry {
        yield_now();
    }
    trace(format_args!("expiry-{}-{expiry}-{}", config(0), now()));
    let expired = a.commit(writer, service, queued, now()).unwrap();
    require(expired.status == policy::EXPIRED && expired.value == 7);
    route_state(&mut observations, &a, 4, expired.status);
    a.policy.invalidate(admin).unwrap();
    let denied = a
        .admit(
            writer,
            routes::Request {
                caller: writer,
                grant: service,
                id: 4,
                epoch: a.epoch,
                ..request
            },
            now(),
        )
        .unwrap_err();
    require(denied == policy::DENIED);
    route_state(&mut observations, &a, 5, denied);
    a.policy.expire_session(admin, caller.session).unwrap();
    let denied = a
        .admit(
            caller,
            routes::Request {
                id: 5,
                epoch: a.epoch,
                ..request
            },
            now(),
        )
        .unwrap_err();
    require(denied == policy::EXPIRED);
    route_state(&mut observations, &a, 6, denied);
    require(a.queued() == 0);
    let mut line = Line { b: [0; 96], n: 0 };
    write!(&mut line, "routepack-{}-", config(0)).unwrap();
    for value in observations.into_iter().flatten() {
        write!(&mut line, "{value:02x}").unwrap();
    }
    sdk::log(&line.b[..line.n]).unwrap();
    7
}
fn share(h: u64, wave: u64) {
    let memory = ok(MEMORY_CREATE, word(3), 4096, 32, 0);
    let map = sdk::wire::map(0x61000000, 0, 4096, 0).unwrap();
    ok(MEMORY_MAP, word(3), memory, map.as_ptr() as u64, 32);
    let attachment = sdk::wire::attachment(Handle(memory), 1 | 4 | 16 | 32, 1).unwrap();
    let bytes = sdk::wire::words::<4, 32>([0, 3, wave, 0]);
    sdk::guest::send_until(
        Handle(h),
        &bytes,
        &[attachment],
        MonotonicNs(now() + 320_000_000),
        Handle(0),
    )
    .unwrap();
    let (ack, _) = receive(h);
    require(ack == [0, 3, wave, 4096]);
    ok(MEMORY_UNMAP, word(3), 0x61000000, 0, 0);
    ok(HANDLE_CLOSE, memory, 0, 0, 0);
}
fn client() {
    let wave = config(0);
    let h = config(2);
    let warm = if wave == 0 { 1000 } else { 0 };
    if wave == 0 {
        let start = now();
        for _ in 0..1024 {
            core::hint::black_box(now());
        }
        trace(format_args!("clockcost-1024-{}", now() - start));
        require(invoke(OBJECT_INFO, u64::MAX, 0, 0, 0).0 == BADHANDLE);
        let start = now();
        for i in 0..32 {
            trace(format_args!("diagbench-{i}"));
        }
        let end = now();
        require(invoke(OBJECT_INFO, u64::MAX - 1, 0, 0, 0).0 == BADHANDLE);
        trace(format_args!("diagcost-32-{}", end - start));
    }
    let mut samples = [0u32; 100];
    let echo_started = now();
    for i in 0..warm + 100 {
        let start = now();
        raw_send(h, [i, 0, wave, 20261025]);
        let (reply, from) = raw_reply(h);
        require(
            reply == [i, 0, wave, 20261025]
                && from.process == config(16)
                && from.generation == config(17),
        );
        let elapsed = now() - start;
        require(elapsed <= 320_000_000);
        if i >= warm {
            samples[(i - warm) as usize] = elapsed as u32;
        }
        progress();
    }
    let echo_finished = now();
    send(h, [0, 1, wave, 0]);
    let (reply, _) = receive(h);
    require(reply == [0, 1, wave, 7]);
    let route_finished = now();
    share(h, wave);
    trace(format_args!(
        "workphase-{wave}-{}-{}-{}",
        echo_finished - echo_started,
        route_finished - echo_finished,
        now() - route_finished
    ));
    for (batch, chunk) in samples.chunks(16).enumerate() {
        let mut line = Line { b: [0; 96], n: 0 };
        write!(&mut line, "ipc62-{wave}-{batch}-").unwrap();
        base62(&mut line, chunk);
        sdk::log(&line.b[..line.n]).unwrap();
    }
    faults::run(wave);
    send(h, [0, 2, wave, 0]);
    if wave % 10 == 6 {
        let _ = receive(h);
        require(false);
    }
    if wave % 10 == 3 {
        unsafe {
            (0x60000000 as *mut u64).write_volatile(1);
        }
    }
}
fn server() {
    let h = config(2);
    let caller = identity(21);
    let mut roundtrips = 0;
    let mut retained = 0;
    loop {
        let mut b = [0; 32];
        let mut handles = [[0u8; 8]; 1];
        let received = raw_receive(h, &mut b, &mut handles);
        if received == Err(PEERCLOSED) {
            require(roundtrips == if config(0) == 0 { 1100 } else { 100 });
            break;
        }
        let received = received.unwrap();
        require(received.bytes == 32);
        let request = sdk::wire::read_words::<4>(&b).unwrap();
        let from = received.sender;
        require(
            from.process == caller.process
                && from.generation == caller.generation
                && from.principal == caller.principal
                && from.session == caller.session
                && from.service == caller.service,
        );
        require(received.handles == usize::from(request[1] == 3));
        match request[1] {
            0 => {
                raw_send(h, request);
                roundtrips += 1;
                progress();
            }
            1 => {
                require(roundtrips == if config(0) == 0 { 1100 } else { 100 });
                let checks = route_cycle(caller);
                send(h, [0, 1, config(0), checks]);
                trace(format_args!("routecycle-{}-{checks}", config(0)));
            }
            2 => {
                if config(0) % 10 == 6 {
                    unsafe {
                        asm!("ud2", options(noreturn));
                    }
                }
                while live(caller) {
                    yield_now();
                }
                break;
            }
            3 => {
                require(retained == 0);
                retained = u64::from_le_bytes(handles[0]);
                let map = sdk::wire::map(0x61000000, 0, 4096, 0).unwrap();
                ok(MEMORY_MAP, word(3), retained, map.as_ptr() as u64, 32);
                require(unsafe { (0x61000000 as *const u8).read_volatile() } == 0);
                send(h, [0, 3, config(0), 4096]);
            }
            _ => require(false),
        }
    }
    if retained != 0 {
        require(unsafe { (0x61000000 as *const u8).read_volatile() } == 0);
        ok(MEMORY_UNMAP, word(3), 0x61000000, 0, 0);
        ok(HANDLE_CLOSE, retained, 0, 0, 0);
    }
}
fn control() {
    let mut checksum = 20261025u64;
    loop {
        let (e, _) = invoke(CHANNEL_SIZE, config(2), 0, 0, 0);
        if e == PEERCLOSED {
            break;
        }
        require(e == WOULDBLOCK);
        checksum = checksum.wrapping_mul(1664525).wrapping_add(1013904223);
        progress();
        yield_now();
    }
    trace(format_args!("control-{}-{checksum}", word(1)));
}
extern "C" fn main(startup: *const u8, token: u64, parent: u64) {
    require(
        sdk::wire::Startup::decode(unsafe { core::slice::from_raw_parts(startup, 64) }).is_ok(),
    );
    if (500..504).contains(&word(0)) || word(0) >= 1000 {
        supervision::run(token, parent);
        sdk::exit(99, Handle(word(3)));
    }
    match config(1) {
        0 => control(),
        1 => client(),
        2 => server(),
        _ => require(false),
    }
    sdk::exit(0, Handle(word(3)))
}
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        trace(format_args!("panic-line-{}", location.line()));
    }
    let _ = sdk::log(if word(0) == 1002 {
        b"panic"
    } else {
        b"stress-panic"
    });
    sdk::exit(if word(0) == 1002 { 1 } else { 98 }, Handle(word(3)))
}
