//! Trusted boot/test adapter only. All grant decisions execute in the CPL3 broker.
use super::*;
use continuum_kernel::objects::channels::ENDPOINT_RIGHTS;
const COUNT: usize = 6;
fn sender(t: Task) -> [u64; 5] {
    [
        t.current.id.slot as u64,
        t.current.id.generation,
        t.current.context.principal,
        t.current.context.session,
        u64::from(t.current.context.service),
    ]
}
#[inline(never)]
fn configure(wave: u64) -> Result<()> {
    let mut endpoints = [[0u64; 8]; COUNT];
    let tasks: [Option<Task>; COUNT] = core::array::from_fn(|i| unsafe { (&*TASKS.0.get())[i] });
    let broker = tasks[0].unwrap();
    for i in 1..COUNT {
        let other = tasks[i].unwrap();
        let pair = capabilities::registry(|c| {
            let mut pair = [0; 2];
            // The self-process handle occupies the first table entry.
            // prepare_task writes it to the control page; use trusted lookup below.
            c.channel_pair(
                broker.current.space,
                c.self_process_handle(broker.current.space)?,
                |v| {
                    pair = v;
                    Ok(())
                },
            )?;
            let remote = c.grant_endpoint(
                broker.current.space,
                pair[1],
                other.current.space,
                ENDPOINT_RIGHTS,
            )?;
            Ok::<_, continuum_kernel::objects::Error>((pair[0], pair[1], remote))
        })
        .map_err(|_| error("routes", "pair"))?;
        runtime_memory::with(|r, b| {
            capabilities::registry(|c| c.close(r, b, broker.current.space, pair.1))
        })
        .map_err(|_| error("routes", "close"))?;
        endpoints[0][i - 1] = pair.0;
        endpoints[i][0] = pair.2;
    }
    // Two proxy/service paths plus a retained client/old-service endpoint.
    for (left, right, li, ri) in [(2, 4, 1, 1), (2, 5, 2, 1), (3, 4, 1, 2)] {
        let a = tasks[left].unwrap();
        let b = tasks[right].unwrap();
        let pair = capabilities::registry(|c| {
            let mut pair = [0; 2];
            c.channel_pair(
                a.current.space,
                c.self_process_handle(a.current.space)?,
                |v| {
                    pair = v;
                    Ok(())
                },
            )?;
            let h = c.grant_endpoint(a.current.space, pair[1], b.current.space, ENDPOINT_RIGHTS)?;
            Ok::<_, continuum_kernel::objects::Error>((pair[0], pair[1], h))
        })
        .map_err(|_| error("routes", "pair"))?;
        runtime_memory::with(|r, b| {
            capabilities::registry(|c| c.close(r, b, a.current.space, pair.1))
        })
        .map_err(|_| error("routes", "close"))?;
        endpoints[left][li] = pair.0;
        endpoints[right][ri] = pair.2;
    }
    for i in 0..COUNT {
        let t = tasks[i].unwrap();
        runtime_memory::with(|r, b| {
            let allocation = r.allocate(b, t.current.space, PAGE, PAGE)?.unwrap();
            let mut words = [0u64; 64];
            words[0] = wave;
            words[1] = i as u64;
            words[2..10].copy_from_slice(&endpoints[i]);
            for j in 0..COUNT {
                words[16 + j * 5..21 + j * 5].copy_from_slice(&sender(tasks[j].unwrap()));
            }
            for (j, w) in words.iter().enumerate() {
                r.write_staging(
                    b,
                    t.current.space,
                    allocation,
                    j as u64 * 8,
                    &w.to_le_bytes(),
                )?;
            }
            r.map(
                b,
                t.current.space,
                allocation,
                0x71000000,
                0,
                PAGE,
                Access::Read,
            )?;
            Ok::<_, continuum_kernel::Error>(())
        })?;
        let root = runtime_memory::with(|r, _| r.root(t.current.space))?.0;
        unsafe {
            (&mut *TASKS.0.get())[i].as_mut().unwrap().root = root;
        }
        record(
            "m1.route_admit",
            "routes",
            "ok",
            format_args!(
                "wave={wave} actor={i} process={} generation={} principal={} authority_session={} service={} private_root={} roster_readonly=true",
                t.current.id.slot,
                t.current.id.generation,
                t.current.context.principal,
                t.current.context.session,
                t.current.context.service,
                t.root
            ),
        );
    }
    Ok(())
}
#[inline(never)]
pub(crate) fn run() -> Result<()> {
    fixture::install_routes();
    unsafe {
        asm!("cli");
        let root: u64;
        asm!("mov {}, cr3",out(reg)root);
        *KERNEL_ROOT.0.get() = root;
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
    capabilities::enable();
    capabilities::enable_channels();
    waiting::enable();
    let memory = runtime_memory::with(|r, _| r.inventory());
    let objects = capabilities::inventory();
    let frames = crate::paging::runtime_frames();
    let fixed = memory.metadata_bytes
        + memory.table_storage_bytes
        + 16 * 73728
        + core::mem::size_of::<[Option<Task>; N]>()
        + core::mem::size_of::<continuum_kernel::objects::Registry>()
        + core::mem::size_of::<Launcher<16>>()
        + waiting::fixed_bytes()
        + 8192;
    need(fixed <= 16 * 1024 * 1024)?;
    for wave in 0..9 {
        for i in 0..COUNT {
            let (principal, session, service) = match i {
                0 => (303, 3003 + wave, true),
                1 => (404, 4004 + wave, true),
                3 => (101, 1001 + wave, false),
                _ => (505, 5005 + wave, true),
            };
            let context = Context {
                principal,
                session,
                service,
                grant_ceiling: if service { 3 } else { 1 },
                commitment_ceiling: 1048576,
            };
            let slot = supervision::prepare_task(
                if i == 3 { 0 } else { 2 },
                700 + i as u64,
                context,
                false,
            )?;
            need(slot == i)?;
        }
        configure(wave)?;
        let mut turns = 0;
        while unsafe { (&*TASKS.0.get()).iter().any(Option::is_some) } {
            let mut wakes = Queue::EMPTY;
            waiting::poll(&mut wakes)?;
            while wakes.pop().is_some() {}
            let mut dispatched = false;
            for i in 0..COUNT {
                let Some(t) = (unsafe { (&*TASKS.0.get())[i] }) else {
                    continue;
                };
                let thread = ThreadId {
                    slot: i,
                    generation: t.current.id.generation,
                };
                if waiting::blocked(thread) {
                    continue;
                }
                dispatched = true;
                turns += 1;
                need(turns < 8192)?;
                unsafe {
                    *CURRENT.0.get() = Some(t.current);
                    *INDEX.0.get() = i;
                    *KERNEL_SP.0.get() = t.stack;
                    *HEAP.0.get() = t.heap;
                    *PENDING.0.get() = false;
                    *BEGIN.0.get() = 0;
                    for msr in [0xc0000100, 0xc0000101, 0xc0000102] {
                        wrmsr(msr, 0);
                    }
                }
                crate::exceptions::user_stack(t.stack);
                need(valid_return(&t.frame))?;
                waiting::resume(thread)?;
                unsafe {
                    let task = (&mut *TASKS.0.get())[i].as_mut().unwrap();
                    task.dispatches += 1;
                    *ENABLED.0.get() = true;
                    task_resume(task.root, &task.frame, &task.fp);
                    *ENABLED.0.get() = false;
                }
                let t = unsafe { (&*TASKS.0.get())[i].unwrap() };
                if let Some(code) = t.result {
                    record(
                        "m1.route_end",
                        "routes",
                        "ok",
                        format_args!(
                            "wave={wave} actor={i} process={} generation={} code={code} vector={} dispatches={}",
                            t.current.id.slot, t.current.id.generation, t.vector, t.dispatches
                        ),
                    );
                    let victim = match wave {
                        1 | 2 | 3 | 4 | 7 => 0,
                        5 => 4,
                        6 | 8 => 2,
                        _ => 99,
                    };
                    need(code == if i == victim { 262 } else { 0 })?;
                    waiting::terminate(ThreadId {
                        slot: i,
                        generation: t.current.id.generation,
                    });
                    unsafe {
                        *CURRENT.0.get() = None;
                    }
                    crate::exceptions::user_stack(crate::exceptions::stacks()[0] + 69632);
                    capabilities::remove(t.current.space)?;
                    runtime_memory::with(|r, b| unsafe {
                        (&mut *LAUNCHER.0.get()).destroy(r, b, t.current.id)
                    })?;
                    unsafe {
                        (&mut *TASKS.0.get())[i] = None;
                    }
                }
            }
            if !dispatched {
                waiting::idle()?;
            }
        }
        let m = runtime_memory::with(|r, _| r.inventory());
        let o = capabilities::inventory();
        need(m == memory && o == objects && crate::paging::runtime_frames() == frames)?;
        waiting::settled()?;
        record(
            "m1.route_settled",
            "routes",
            "ok",
            format_args!(
                "wave={wave} objects={} handles={} mappings={} spaces={} allocations={} committed={} resident={} tables={} stacks={} pins={} frames={frames}",
                o.objects,
                o.handles,
                o.mappings,
                m.spaces,
                m.allocations,
                m.committed,
                m.resident,
                m.page_tables,
                m.kernel_stack_bytes,
                m.active_pins
            ),
        );
    }
    record(
        "m1.route_ready",
        "routes",
        "ok",
        format_args!("waves=9 tasks=54 settled=true user_broker=true fixed_bytes={fixed}"),
    );
    unsafe {
        asm!("sti");
    }
    Ok(())
}
