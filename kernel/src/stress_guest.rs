//! Bounded SDK stress coordinator. All mutation/reap points hold IF clear and
//! have no outstanding user copy pin, registry reference or kernel continuation.
use super::*;
use continuum_kernel::objects::channels::ENDPOINT_RIGHTS;
const WAVES: u64 = 100;
const BOUND: u64 = 320_000_000;
type Verified = continuum_kernel::user_image::VerifiedImage<'static>;
const _: () = assert!(core::mem::size_of::<[Option<Verified>; 3]>() <= 4096);
static IMAGES_VERIFIED: Local<[Option<Verified>; 3]> = Local(UnsafeCell::new([const { None }; 3]));
pub(crate) fn launch(
    launcher: &mut Launcher<16>,
    memory: &mut runtime_memory::Memory,
    store: &mut runtime_memory::GuestStore,
    module: usize,
    context: Context,
) -> Result<ProcessId> {
    let image = unsafe { (&*IMAGES_VERIFIED.0.get())[module].as_ref().unwrap() };
    launcher.launch_verified_context(memory, store, image, context, EXIT_TRAMPOLINE, true)
}
static ARMED: Local<bool> = Local(UnsafeCell::new(false));
static KILL: Local<bool> = Local(UnsafeCell::new(false));
static OPERATION: Local<u64> = Local(UnsafeCell::new(0));
static WAVE: Local<u64> = Local(UnsafeCell::new(0));
static DIAGNOSTICS: Local<bool> = Local(UnsafeCell::new(false));
static TRACE_CALLS: Local<u64> = Local(UnsafeCell::new(0));
static PEAK_FRAMES: Local<usize> = Local(UnsafeCell::new(0));
static PEAK_OBJECTS: Local<usize> = Local(UnsafeCell::new(0));
static PEAK_HANDLES: Local<usize> = Local(UnsafeCell::new(0));
static PEAK_MESSAGES: Local<usize> = Local(UnsafeCell::new(0));
static PEAK_QUEUED: Local<usize> = Local(UnsafeCell::new(0));
static PEAK_SHARED: Local<u64> = Local(UnsafeCell::new(0));
static PEAK_WAITS: Local<usize> = Local(UnsafeCell::new(0));
static PEAK_TIMERS: Local<usize> = Local(UnsafeCell::new(0));
static QUEUED_HANDLES: Local<usize> = Local(UnsafeCell::new(0));
static PHASE_NS: Local<[u64; 3]> = Local(UnsafeCell::new([0; 3]));
static MASKED_BOUNDARY: Local<(u64, u64, u64, u64)> = Local(UnsafeCell::new((0, 0, 0, 0)));
static PEAK_COMMITTED: Local<u64> = Local(UnsafeCell::new(0));
// Every adjacent pair of observed gaps contains a real user IRET/STI boundary.
// Their sum conservatively covers any IF-clear interval crossing an observation;
// it deliberately includes user/HLT time rather than understating masked time.
fn masked_boundary(at: u64) {
    unsafe {
        let state = &mut *MASKED_BOUNDARY.0.get();
        if state.0 != 0 {
            let gap = at.checked_sub(state.0).expect("monotonic boundary clock");
            state.2 = state
                .2
                .max(state.1 + gap + crate::timer::elapsed_resolution());
            state.1 = gap;
            state.3 += 1;
        }
        state.0 = at;
    }
}
#[derive(Clone, Copy)]
struct Samples {
    values: [u32; 4096],
    count: usize,
    observed: u64,
    maximum: u64,
}
impl Samples {
    const EMPTY: Self = Self {
        values: [0; 4096],
        count: 0,
        observed: 0,
        maximum: 0,
    };
    fn push(&mut self, ns: u64, kind: &str, actor: usize) {
        if ns > BOUND {
            record(
                "m1.stress_bound_violation",
                "routes",
                "ok",
                format_args!(
                    "kind={kind} actor={actor} wave={} value_ns={ns} bound_ns={BOUND} observed={}",
                    unsafe { *WAVE.0.get() },
                    self.observed
                ),
            );
        }
        assert!(ns <= BOUND);
        self.observed += 1;
        self.maximum = self.maximum.max(ns);
        if self.observed % 32 == 0 {
            assert!(self.count < 4096);
            self.values[self.count] = ns as u32;
            self.count += 1;
        }
    }
    fn emit(&self, kind: &str) {
        for (batch, values) in self.values[..self.count].chunks(16).enumerate() {
            let mut line = crate::Record {
                bytes: [0; 512],
                len: 0,
            };
            for value in values {
                core::fmt::Write::write_fmt(&mut line, format_args!("{value:08x}")).unwrap();
            }
            let raw = core::str::from_utf8(&line.bytes[..line.len]).unwrap();
            record(
                "m1.stress_samples",
                "routes",
                "ok",
                format_args!("kind={kind} batch={batch} raw={raw}"),
            );
        }
        record(
            "m1.stress_metric",
            "routes",
            "ok",
            format_args!(
                "kind={kind} samples={} observed={} maximum_ns={} bound_ns={BOUND} stride=32 limit=4096",
                self.count, self.observed, self.maximum
            ),
        );
    }
}
static DISPATCH: Local<Samples> = Local(UnsafeCell::new(Samples::EMPTY));
static WAKE: Local<Samples> = Local(UnsafeCell::new(Samples::EMPTY));
pub(crate) fn wake_sample(ns: u64, actor: usize) {
    unsafe { (&mut *WAKE.0.get()).push(ns, "wake", actor) }
}
pub(crate) fn entry(op: u64, argument: u64, entered: u64) {
    begin_syscall(entered);
    if current().case == 900 && op == OBJECT_INFO {
        if argument == u64::MAX {
            unsafe {
                *DIAGNOSTICS.0.get() = true;
            }
        }
        if argument == u64::MAX - 1 {
            unsafe {
                *DIAGNOSTICS.0.get() = false;
            }
        }
    }
    unsafe {
        *OPERATION.0.get() = op;
    }
    if current().case >= 900 && current().case < 910 && op == OBJECT_INFO && argument == 0 {
        unsafe {
            *ARMED.0.get() = true;
        }
    }
    unsafe {
        *TRACE_CALLS.0.get() += 1;
        if cfg!(feature = "stress-trace") && *TRACE_CALLS.0.get() % 16 == 0 {
            sample_peak();
        }
    }
}
pub(crate) fn diagnostic_benchmark() -> bool {
    unsafe { *DIAGNOSTICS.0.get() }
}
pub(crate) fn race_pending() -> bool {
    unsafe { *ARMED.0.get() && *WAVE.0.get() % 20 == 2 }
}
pub(crate) fn race_complete() {
    unsafe {
        *ARMED.0.get() = false;
    }
}
pub(crate) fn copy_observer(pinned: bool) {
    if pinned
        && unsafe { *ARMED.0.get() }
        && ((current().case == 908 && unsafe { *OPERATION.0.get() } == MEMORY_MAP)
            || (current().case == 909 && unsafe { *OPERATION.0.get() } == CHANNEL_SEND))
    {
        unsafe {
            *KILL.0.get() = true;
            *ARMED.0.get() = false;
        }
        // The real nested IRQ can only mark pending while a borrow/pin is live.
        copy_irq(true);
    }
}
pub(crate) fn return_observer(f: &Frame, op: u64) {
    if (CHANNEL_CREATE..=CHANNEL_RECEIVE).contains(&op) || op == 25 || op == 26 {
        // Inline-only traffic changes queue occupancy, not backing ownership.
        // A transfer entering or leaving a queue also samples the full registry.
        let q = capabilities::registry(|c| c.channel_usage());
        let held = unsafe { *QUEUED_HANDLES.0.get() };
        sample_queue_peak(q.messages, q.bytes, q.handles);
        if op == CHANNEL_CREATE || held != 0 || q.handles != 0 {
            sample_objects_peak();
        }
    } else if (HANDLE_CLOSE..CHANNEL_CREATE).contains(&op)
        || (EVENT_CREATE..=TIMER_CANCEL).contains(&op)
    {
        sample_objects_peak();
    }
    if matches!(op, MEMORY_ALLOCATE | MEMORY_CREATE | MEMORY_MAP) {
        sample_peak();
    }
    if op == MEMORY_MAP && f.rax == 0 {
        let backing = capabilities::registry(|c| c.backing_usage());
        if backing.shared_bytes > 0 {
            record(
                "m1.stress_share",
                "routes",
                "ok",
                format_args!(
                    "wave={} shared={} object_backing={} counted_once=true",
                    unsafe { *WAVE.0.get() },
                    backing.shared_bytes,
                    capabilities::inventory().backing_bytes
                ),
            );
        }
    }
    if op == WAIT_MANY {
        unsafe {
            *PEAK_WAITS.0.get() = (*PEAK_WAITS.0.get()).max(waiting::stress_usage().0);
        }
    }
    if unsafe { *KILL.0.get() } {
        assert_eq!(f.rax, 0, "interrupted operation must commit successfully");
        let memory = runtime_memory::with(|r, _| r.inventory());
        let objects = capabilities::inventory();
        let backing = capabilities::registry(|c| c.backing_usage());
        assert_eq!(memory.active_pins, 0);
        assert!(if current().case == 908 {
            objects.mappings == 2
        } else {
            backing.queued_only_bytes == 4096
        });
        record(
            "m1.stress_kill",
            "routes",
            "ok",
            format_args!(
                "wave={} cause=copy operation={op} status={} pins={} mappings={} queued_only={} code=768",
                unsafe { *WAVE.0.get() },
                f.rax,
                memory.active_pins,
                objects.mappings,
                backing.queued_only_bytes
            ),
        );
        unsafe {
            *KILL.0.get() = false;
        }
        sample_peak();
        finish("deferred-stress-kill", 768, 0);
    }
    waiting::notify().unwrap();
    syscall_return(
        f,
        op == THREAD_YIELD && f.rax == 0 || waiting::pending_current(),
    );
}
fn sample_peak() {
    let m = runtime_memory::with(|r, _| r.inventory());
    need(m.resident == crate::paging::runtime_frames() as u64 * PAGE).unwrap();
    unsafe {
        *PEAK_COMMITTED.0.get() = (*PEAK_COMMITTED.0.get()).max(m.committed);
    }
    sample_objects_peak();
}
fn sample_objects_peak() {
    let o = capabilities::inventory();
    let q = capabilities::registry(|c| c.channel_usage());
    let backing = capabilities::registry(|c| c.backing_usage());
    sample_queue_peak(q.messages, q.bytes, q.handles);
    unsafe {
        *PEAK_FRAMES.0.get() = (*PEAK_FRAMES.0.get()).max(crate::paging::runtime_frames());
        *PEAK_OBJECTS.0.get() = (*PEAK_OBJECTS.0.get()).max(o.objects);
        *PEAK_HANDLES.0.get() = (*PEAK_HANDLES.0.get()).max(o.handles);
        *PEAK_SHARED.0.get() = (*PEAK_SHARED.0.get()).max(backing.shared_bytes);
        *PEAK_TIMERS.0.get() = (*PEAK_TIMERS.0.get()).max(backing.timers);
    }
}
fn sample_queue_peak(messages: usize, bytes: usize, handles: usize) {
    unsafe {
        *PEAK_MESSAGES.0.get() = (*PEAK_MESSAGES.0.get()).max(messages);
        *PEAK_QUEUED.0.get() = (*PEAK_QUEUED.0.get()).max(bytes);
        *QUEUED_HANDLES.0.get() = handles;
    }
}
fn snapshot(wave: u64, phase: &str) {
    let started = now();
    sample_peak();
    let m = runtime_memory::with(|r, _| r.inventory());
    let o = capabilities::inventory();
    let q = capabilities::registry(|c| c.channel_usage());
    let pool = crate::paging::runtime_pool();
    let image = crate::paging::image_bounds();
    let backing = capabilities::registry(|c| c.backing_usage());
    let mut owners = [[0u64; 7]; 3];
    let mut count = 0usize;
    let mut owned = 0;
    for actor in 0..3 {
        if let Some(t) = unsafe { (&*TASKS.0.get())[actor] } {
            let (requested, resident) =
                runtime_memory::with(|r, _| r.owner_backing(t.current.space)).unwrap();
            let referenced =
                capabilities::registry(|c| c.referenced_bytes(t.current.space)).unwrap();
            owners[count] = [
                actor as u64,
                t.current.id.slot as u64,
                t.current.id.generation,
                requested,
                resident,
                referenced,
                runtime_memory::with(|r, _| r.owner_commitment(t.current.space)).unwrap(),
            ];
            count += 1;
            owned += resident;
        }
    }
    need(
        backing.private_bytes + backing.shared_bytes + backing.queued_only_bytes == o.backing_bytes,
    )
    .unwrap();
    // Explicit wire order is documented independently in p3-13-inventory.json.
    // Every original numeric observation is retained; this only removes headers.
    let fields = [
        m.spaces as u64,
        m.allocations as u64,
        m.committed,
        m.resident,
        pool.1 as u64,
        (pool.0 - pool.1) as u64 * PAGE,
        pool.0 as u64 * PAGE,
        image.1 - image.0,
        runtime_memory::with(|r, _| r.foreign_commitment()),
        o.objects as u64,
        o.handles as u64,
        o.mappings as u64,
        o.backing_bytes,
        o.mapped_bytes,
        q.messages as u64,
        q.bytes as u64,
        q.handles as u64,
        m.page_tables as u64,
        m.kernel_stack_bytes,
        m.active_pins as u64,
        backing.private_bytes,
        backing.shared_bytes,
        backing.queued_only_bytes,
        m.resident - o.backing_bytes,
        waiting::stress_usage().0 as u64,
        waiting::stress_usage().1 as u64,
        backing.timers as u64,
        backing.events as u64,
        m.resident - owned,
        waiting::stress_usage().2 as u64,
    ];
    let mut encoded = [0u8; 512];
    let mut length = 0;
    for mut value in core::iter::once(count as u64)
        .chain(fields)
        .chain(owners[..count].iter().flatten().copied())
    {
        loop {
            let mut byte = (value & 127) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 128;
            }
            assert!(length < encoded.len());
            encoded[length] = byte;
            length += 1;
            if value == 0 {
                break;
            }
        }
    }
    let parts = length.div_ceil(64);
    for (batch, bytes) in encoded[..length].chunks(64).enumerate() {
        let mut raw = crate::Record {
            bytes: [0; 512],
            len: 0,
        };
        for byte in bytes {
            core::fmt::Write::write_fmt(&mut raw, format_args!("{byte:02x}")).unwrap();
        }
        let raw = core::str::from_utf8(&raw.bytes[..raw.len]).unwrap();
        record(
            "m1.stress_inventory",
            "routes",
            "ok",
            format_args!(
                "wave={wave} phase={phase} batch={batch} parts={parts} schema=2 raw={raw}"
            ),
        );
    }
    unsafe {
        (*PHASE_NS.0.get())[2] += now() - started;
    }
}
fn pair(left: usize, right: usize) -> Result<(u64, u64)> {
    let a = unsafe { (&*TASKS.0.get())[left].unwrap() };
    let b = unsafe { (&*TASKS.0.get())[right].unwrap() };
    let (local, temporary, remote) = capabilities::registry(|c| {
        let mut pair = [0; 2];
        c.channel_pair(
            a.current.space,
            c.self_process_handle(a.current.space)?,
            |p| {
                pair = p;
                Ok(())
            },
        )?;
        let h = if left == right {
            pair[1]
        } else {
            c.grant_endpoint(a.current.space, pair[1], b.current.space, ENDPOINT_RIGHTS)?
        };
        Ok::<_, continuum_kernel::objects::Error>((pair[0], pair[1], h))
    })
    .map_err(|_| error("stress", "pair"))?;
    if left != right {
        runtime_memory::with(|r, b| {
            capabilities::registry(|c| c.close(r, b, a.current.space, temporary))
        })
        .map_err(|_| error("stress", "close"))?;
    }
    Ok((local, if left == right { temporary } else { remote }))
}
fn configure(slot: usize, wave: u64, role: u64, endpoint: u64) -> Result<()> {
    let task = unsafe { (&*TASKS.0.get())[slot].unwrap() };
    let mut words = [0u64; 32];
    words[0] = wave;
    words[1] = role;
    words[2] = endpoint;
    for (at, other) in [(16, 2), (21, 1)] {
        if let Some(t) = unsafe { (&*TASKS.0.get())[other] } {
            words[at..at + 5].copy_from_slice(&[
                t.current.id.slot as u64,
                t.current.id.generation,
                t.current.context.principal,
                t.current.context.session,
                u64::from(t.current.context.service),
            ]);
        }
    }
    runtime_memory::with(|r, b| {
        let a = r.allocate(b, task.current.space, PAGE, PAGE)?.unwrap();
        let mut bytes = [0u8; 256];
        for (i, w) in words.iter().enumerate() {
            bytes[i * 8..i * 8 + 8].copy_from_slice(&w.to_le_bytes());
        }
        r.write_staging(b, task.current.space, a, 0, &bytes)?;
        r.map(b, task.current.space, a, 0x71000000, 0, PAGE, Access::Read)?;
        Ok::<_, continuum_kernel::Error>(())
    })?;
    let root = runtime_memory::with(|r, _| r.root(task.current.space))?.0;
    unsafe {
        let task = (&mut *TASKS.0.get())[slot].as_mut().unwrap();
        task.root = root;
        task.ready_ns = now();
    }
    record(
        "m1.stress_admit",
        "routes",
        "ok",
        format_args!(
            "wave={wave} actor={slot} process={} generation={} principal={} authority_session={} service={} private_root={root} bootstrap=true",
            task.current.id.slot,
            task.current.id.generation,
            task.current.context.principal,
            task.current.context.session,
            task.current.context.service
        ),
    );
    Ok(())
}
fn step(slot: usize) -> Result<bool> {
    let Some(t) = (unsafe { (&*TASKS.0.get())[slot] }) else {
        return Ok(false);
    };
    let thread = ThreadId {
        slot,
        generation: t.current.id.generation,
    };
    if waiting::blocked(thread) {
        return Ok(false);
    }
    let started = now();
    masked_boundary(started);
    measure_masked();
    let at = now();
    if t.ready_ns != 0 {
        unsafe {
            let delay = continuum_kernel::wait::runnable_delay(
                t.ready_ns,
                waiting::ready_time(thread),
                at,
            )?;
            (&mut *DISPATCH.0.get()).push(delay, "dispatch", slot);
        }
    }
    unsafe {
        *CURRENT.0.get() = Some(t.current);
        *INDEX.0.get() = slot;
        *KERNEL_SP.0.get() = t.stack;
        *HEAP.0.get() = t.heap;
        *PENDING.0.get() = false;
        *BEGIN.0.get() = 0;
        // install() established zero bases. FSGSBASE is disabled before any
        // user entry, all admitted GDT bases are zero, no TLS/MSR operation is
        // exposed, and task_resume restores null selectors. Under this declared
        // single-CPU profile there is no writer between resumes.
    }
    crate::exceptions::user_stack(t.stack);
    need(valid_return(&t.frame))?;
    waiting::resume(thread)?;
    unsafe {
        let task = (&mut *TASKS.0.get())[slot].as_mut().unwrap();
        task.dispatches += 1;
        *ENABLED.0.get() = true;
        task_resume(task.root, &task.frame, &task.fp);
        *ENABLED.0.get() = false;
        (&mut *TASKS.0.get())[slot].as_mut().unwrap().ready_ns = now();
        (*PHASE_NS.0.get())[0] += now() - started;
    }
    Ok(true)
}
fn reap(slot: usize, wave: u64, expected: u64) -> Result<()> {
    let started = now();
    let t = unsafe { (&*TASKS.0.get())[slot].unwrap() };
    need(t.result == Some(expected))?;
    record(
        "m1.stress_end",
        "routes",
        "ok",
        format_args!(
            "wave={wave} actor={slot} process={} generation={} code={expected} vector={} dispatches={} preemptions={} progress={}",
            t.current.id.slot,
            t.current.id.generation,
            t.vector,
            t.dispatches,
            t.preemptions,
            t.progress
        ),
    );
    waiting::terminate(ThreadId {
        slot,
        generation: t.current.id.generation,
    });
    unsafe {
        *CURRENT.0.get() = None;
    }
    crate::exceptions::user_stack(crate::exceptions::stacks()[0] + 69632);
    capabilities::remove(t.current.space)?;
    runtime_memory::with(|r, b| unsafe { (&mut *LAUNCHER.0.get()).destroy(r, b, t.current.id) })?;
    unsafe {
        (&mut *TASKS.0.get())[slot] = None;
        (*PHASE_NS.0.get())[1] += now() - started;
    }
    Ok(())
}
#[inline(never)]
pub(crate) fn run() -> Result<()> {
    masked_boundary(now());
    fixture::install_stress();
    unsafe {
        asm!("cli");
        let root: u64;
        asm!("mov {}, cr3",out(reg)root);
        *KERNEL_ROOT.0.get() = root;
        *BEGIN.0.get() = now();
    }
    need(cpu_supported(core::arch::x86_64::__cpuid(1).edx, 0, 7))?;
    install()?;
    unsafe {
        let cr0: u64;
        let cr4: u64;
        asm!("mov {}, cr0","mov {}, cr4",out(reg)cr0,out(reg)cr4);
        asm!("mov cr0, {}",in(reg)cr0|(1<<5));
        asm!("mov cr4, {}",in(reg)cr4&!((1<<18)|(1<<16)|(1<<22)|(1<<23)));
    }
    let bases = [0xc0000100u32, 0xc0000101, 0xc0000102].map(|id| {
        let low: u32;
        let high: u32;
        unsafe {
            asm!("rdmsr", in("ecx") id, out("eax") low, out("edx") high);
        }
        (u64::from(high) << 32) | u64::from(low)
    });
    let cr4: u64;
    unsafe {
        asm!("mov {}, cr4", out(reg) cr4);
    }
    need(bases == [0; 3] && cr4 & (1 << 16) == 0)?;
    record(
        "m1.stress_cpu",
        "routes",
        "ok",
        format_args!(
            "fs_base={} gs_base={} kernel_gs_base={} cr4={cr4:#x} tls=false segment_policy=constant_zero",
            bases[0], bases[1], bases[2]
        ),
    );
    capabilities::enable();
    capabilities::enable_channels();
    waiting::enable();
    let verification_start = now();
    for module in 0..3 {
        let image = Verified::verify(TrustedImage {
            bytes: crate::paging::module(module)?,
            digest: IMAGES[module].0,
            size: IMAGES[module].1,
        })?;
        unsafe {
            (&mut *IMAGES_VERIFIED.0.get())[module] = Some(image);
        }
    }
    record(
        "m1.stress_cache",
        "routes",
        "ok",
        format_args!(
            "entries=3 bytes={} verification_ns={} immutable=true writable_aliases=0 retained=true charged=kernel_image",
            core::mem::size_of::<[Option<Verified>; 3]>(),
            now() - verification_start
        ),
    );
    let baseline = runtime_memory::with(|r, _| r.inventory());
    let active_start = now();
    let objects = capabilities::inventory();
    let frames = crate::paging::runtime_frames();
    let image = crate::paging::image_bounds();
    let pool = crate::paging::runtime_pool();
    let info = unsafe { &*crate::BOOT_INFO.0.get() };
    let modules = info.modules[..info.module_count]
        .iter()
        .map(|m| (m.range.base % PAGE + m.range.len).div_ceil(PAGE) * PAGE)
        .sum::<u64>();
    let video = info.framebuffers[..info.framebuffer_count]
        .iter()
        .map(|f| f.range.len)
        .sum::<u64>();
    let reserved = 2147483648u64 - pool.0 as u64 * PAGE;
    need(image.1 - image.0 <= reserved)?;
    record(
        "m1.stress_capacity",
        "routes",
        "ok",
        format_args!(
            "ram_capacity=2147483648 eligible={} reserved_capacity={reserved} kernel_image={} module_bytes={modules} framebuffer_bytes={video} reclamation=0 clock=hpet resolution_ns={} quantum_ns=5000000",
            pool.0 as u64 * PAGE,
            image.1 - image.0,
            crate::timer::elapsed_resolution()
        ),
    );
    let sample_bytes = 2 * core::mem::size_of::<Samples>();
    let fixed = baseline.metadata_bytes
        + baseline.table_storage_bytes
        + 16 * 73728
        + core::mem::size_of::<[Option<Task>; N]>()
        + core::mem::size_of::<continuum_kernel::objects::Registry>()
        + core::mem::size_of::<Launcher<16>>()
        + waiting::fixed_bytes()
        + sample_bytes
        + core::mem::size_of::<[Option<Verified>; 3]>()
        + 8192;
    need(fixed <= 16777216)?;
    record(
        "m1.stress_storage",
        "routes",
        "ok",
        format_args!(
            "kind=runtime metadata={} tables={} stacks=1179648 tasks={} registry={} launcher={} waits={} samples={sample_bytes} fixed_budget={fixed} fixed_ceiling=16777216",
            baseline.metadata_bytes,
            baseline.table_storage_bytes,
            core::mem::size_of::<[Option<Task>; N]>(),
            core::mem::size_of::<continuum_kernel::objects::Registry>(),
            core::mem::size_of::<Launcher<16>>(),
            waiting::fixed_bytes()
        ),
    );
    snapshot(0, "empty");
    let control = Context {
        principal: 404,
        session: 4004,
        service: true,
        grant_ceiling: 3,
        commitment_ceiling: 1048576,
    };
    need(supervision::prepare_task(0, 920, control, false)? == 0)?;
    let (control_read, control_close) = pair(0, 0)?;
    configure(0, 0, 0, control_read)?;
    let steady = runtime_memory::with(|r, _| r.inventory());
    let steady_objects = capabilities::inventory();
    let steady_frames = crate::paging::runtime_frames();
    for wave in 0..WAVES {
        let admission_start = now();
        unsafe {
            *WAVE.0.get() = wave;
            *PHASE_NS.0.get() = [0; 3];
            *ARMED.0.get() = false;
            *KILL.0.get() = false;
        }
        let server = Context {
            principal: 303,
            session: 5005 + wave,
            service: true,
            grant_ceiling: 3,
            commitment_ceiling: 1048576,
        };
        let client = Context {
            session: 3003 + wave,
            service: false,
            grant_ceiling: 1,
            ..server
        };
        // A real rejected launch must leave the live control and inventories intact.
        let before = runtime_memory::with(|r, _| r.inventory());
        let rejected = runtime_memory::with(|r, b| unsafe {
            launch(
                &mut *LAUNCHER.0.get(),
                r,
                b,
                0,
                Context {
                    commitment_ceiling: 1,
                    ..client
                },
            )
        });
        need(
            matches!(rejected,Err(e) if e==error("launch","ceiling"))
                && runtime_memory::with(|r, _| r.inventory()) == before,
        )?;
        need(supervision::prepare_task(0, 900 + wave % 10, client, false)? == 1)?;
        need(supervision::prepare_task(2, 930, server, false)? == 2)?;
        let (send, receive) = pair(1, 2)?;
        configure(1, wave, 1, send)?;
        configure(2, wave, 2, receive)?;
        let admission_end = now();
        snapshot(wave, "staged");
        let progress_before = unsafe { (&*TASKS.0.get())[0].unwrap().progress };
        let mut turns = 0;
        let mut broker_dead = false;
        while unsafe { (&*TASKS.0.get())[1].is_some() || (&*TASKS.0.get())[2].is_some() } {
            let mut wakes = Queue::EMPTY;
            waiting::poll(&mut wakes)?;
            while wakes.pop().is_some() {}
            for slot in 0..3 {
                if slot == 1 && broker_dead {
                    if let Some(t) = unsafe { (&*TASKS.0.get())[1] } {
                        if t.result.is_none() {
                            record(
                                "m1.stress_kill",
                                "routes",
                                "ok",
                                format_args!("wave={wave} cause=broker code=768"),
                            );
                            waiting::terminate(ThreadId {
                                slot: 1,
                                generation: t.current.id.generation,
                            });
                            unsafe {
                                (&mut *TASKS.0.get())[1].as_mut().unwrap().result = Some(768);
                            }
                        }
                    }
                }
                if slot == 1 && wave % 20 == 12 && unsafe { *ARMED.0.get() } {
                    if let Some(t) = unsafe { (&*TASKS.0.get())[1] } {
                        let id = ThreadId {
                            slot: 1,
                            generation: t.current.id.generation,
                        };
                        if waiting::blocked(id) {
                            record(
                                "m1.stress_kill",
                                "routes",
                                "ok",
                                format_args!("wave={wave} cause=wait blocked=true code=768"),
                            );
                            waiting::terminate(id);
                            unsafe {
                                (&mut *TASKS.0.get())[1].as_mut().unwrap().result = Some(768);
                            }
                        }
                    }
                }
                if let Some(t) = unsafe { (&*TASKS.0.get())[slot] } {
                    if t.result.is_none() {
                        step(slot)?;
                        turns += 1;
                        need(turns < 100000)?;
                    }
                    if slot > 0 {
                        if let Some(t) = unsafe { (&*TASKS.0.get())[slot] } {
                            if let Some(code) = t.result {
                                sample_peak();
                                let expected = if slot == 1
                                    && (matches!(wave % 10, 6 | 8 | 9) || wave % 20 == 12)
                                {
                                    768
                                } else if slot == 1 && wave % 10 == 3 {
                                    270
                                } else if slot == 2 && wave % 10 == 6 {
                                    262
                                } else {
                                    0
                                };
                                need(code == expected)?;
                                if slot == 1 && wave % 10 == 4 {
                                    need(t.preemptions > 0)?;
                                }
                                reap(slot, wave, expected)?;
                                if matches!(wave % 10, 3 | 6 | 8 | 9) {
                                    snapshot(
                                        wave,
                                        if slot == 1 {
                                            "client-dead"
                                        } else {
                                            "server-dead"
                                        },
                                    );
                                }
                                if slot == 2 && code == 262 {
                                    broker_dead = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        let progress = unsafe { (&*TASKS.0.get())[0].unwrap().progress };
        need(progress > progress_before)?;
        need(
            runtime_memory::with(|r, _| r.inventory()) == steady
                && capabilities::inventory() == steady_objects
                && crate::paging::runtime_frames() == steady_frames,
        )?;
        let work_end = now();
        let phases = unsafe { *PHASE_NS.0.get() };
        snapshot(wave, "settled");
        record(
            "m1.stress_cycle",
            "routes",
            "ok",
            format_args!(
                "wave={wave} seed=20261025 scenario={} control_before={progress_before} control_after={progress} launched=2 reaped=2 supervision=separate launch_denials=1 admission_ns={} work_ns={} dispatch_ns={} reap_ns={} inventory_ns={}",
                wave % 10,
                admission_end - admission_start,
                work_end - admission_end,
                phases[0],
                phases[1],
                phases[2]
            ),
        );
    }
    let space = unsafe { (&*TASKS.0.get())[0].unwrap().current.space };
    runtime_memory::with(|r, b| capabilities::registry(|c| c.close(r, b, space, control_close)))
        .map_err(|_| error("stress", "control_close"))?;
    let control_deadline = now() + BOUND;
    while unsafe { (&*TASKS.0.get())[0].unwrap().result.is_none() } {
        need(now() < control_deadline)?;
        step(0)?;
    }
    reap(0, WAVES, 0)?;
    need(
        runtime_memory::with(|r, _| r.inventory()) == baseline
            && capabilities::inventory() == objects
            && crate::paging::runtime_frames() == frames,
    )?;
    waiting::settled()?;
    record(
        "m1.stress_interval",
        "routes",
        "ok",
        format_args!(
            "start_ns={active_start} end_ns={} warmup=1000 measured=10000",
            now()
        ),
    );
    let idle_start = now();
    let ticks = crate::timer::uptime()?.0;
    let mut halts = 0;
    while halts < 64 || now() - idle_start < 64 * 5_000_000 {
        masked_boundary(now());
        waiting::idle()?;
        halts += 1;
    }
    unsafe {
        *BEGIN.0.get() = now();
    }
    record(
        "m1.stress_idle",
        "routes",
        "ok",
        format_args!(
            "elapsed_ns={} ticks={} requested_halts=64 halts={halts} processes=0",
            now() - idle_start,
            crate::timer::uptime()?.0 - ticks
        ),
    );
    snapshot(WAVES, "final");
    measure_masked();
    masked_boundary(now());
    // All owners/waits/borrows are gone. Export is interruptible; timer IRQs
    // cannot mutate the frozen samples because scheduling is inactive.
    unsafe {
        *BEGIN.0.get() = 0;
        asm!("sti", "nop");
    }
    masked_boundary(now());
    let export_started = now();
    unsafe {
        (&*DISPATCH.0.get()).emit("dispatch");
        (&*WAKE.0.get()).emit("wake");
    }
    record(
        "m1.stress_export",
        "routes",
        "ok",
        format_args!("elapsed_ns={} interruptible=true", now() - export_started),
    );
    let body = unsafe { *MAX_MASKED.0.get() };
    let boundary = unsafe { *MASKED_BOUNDARY.0.get() };
    let masked = body.max(boundary.2);
    need(masked <= BOUND)?;
    record(
        "m1.stress_peaks",
        "routes",
        "ok",
        format_args!(
            "frames={} objects={} handles={} messages={} queued_bytes={} masked_ns={masked} masked_body_ns={body} segments={} committed={} method=adjacent_resume_sti calls={} traced={}",
            unsafe { *PEAK_FRAMES.0.get() },
            unsafe { *PEAK_OBJECTS.0.get() },
            unsafe { *PEAK_HANDLES.0.get() },
            unsafe { *PEAK_MESSAGES.0.get() },
            unsafe { *PEAK_QUEUED.0.get() },
            boundary.3,
            unsafe { *PEAK_COMMITTED.0.get() },
            unsafe { *TRACE_CALLS.0.get() },
            cfg!(feature = "stress-trace")
        ),
    );
    record(
        "m1.stress_bounds",
        "routes",
        "ok",
        format_args!(
            "peak_shared={} peak_waits={} peak_timers={} pins_deferred={} shared_counted_once=true",
            unsafe { *PEAK_SHARED.0.get() },
            unsafe { *PEAK_WAITS.0.get() },
            unsafe { *PEAK_TIMERS.0.get() },
            unsafe { *PIN_IRQS.0.get() }
        ),
    );
    record(
        "m1.route_ready",
        "routes",
        "ok",
        format_args!("waves={WAVES} tasks=201 settled=true stress=true user_broker=true"),
    );
    unsafe {
        asm!("sti");
    }
    Ok(())
}
