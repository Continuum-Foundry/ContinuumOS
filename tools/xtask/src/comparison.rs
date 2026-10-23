//! P2-08 host-only experiment. RAM submission is not display completion.
use crate::{Result, custom, hash, root, write_json};
use custom::{Candidate, controller, tokens, tree};
use custom_ui::{
    input::{Controller, Event, EventKind},
    paint::Canvas,
    scene::{Dirty, Scene},
    surface::Surface,
};
use editor_model::{Action, Entry, Model};
use serde_json::{Value, json};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    fs,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    thread,
    time::{Duration, Instant},
};

// Rust allocations only: foreign/runtime/OS allocation remains in process counters.
struct Counted;
static COUNT: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
#[global_allocator]
static ALLOCATOR: Counted = Counted;
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() && COUNT.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() && COUNT.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(l.size() as u64, Relaxed);
        }
        p
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let p = unsafe { System.realloc(p, l, n) };
        if !p.is_null() && COUNT.load(Relaxed) {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(n as u64, Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
}
#[repr(C)]
#[derive(Default)]
struct Memory {
    cb: u32,
    faults: u32,
    peak_ws: usize,
    ws: usize,
    peak_paged: usize,
    paged: usize,
    peak_nonpaged: usize,
    nonpaged: usize,
    pagefile: usize,
    peak_pagefile: usize,
    private: usize,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn GetProcessTimes(
        p: *mut std::ffi::c_void,
        created: *mut u64,
        exited: *mut u64,
        kernel: *mut u64,
        user: *mut u64,
    ) -> i32;
}
#[link(name = "psapi")]
unsafe extern "system" {
    fn GetProcessMemoryInfo(p: *mut std::ffi::c_void, m: *mut Memory, cb: u32) -> i32;
}
fn process() -> Result<Value> {
    let mut m = Memory {
        cb: std::mem::size_of::<Memory>() as u32,
        ..Default::default()
    };
    let (mut c, mut e, mut k, mut u) = (0, 0, 0, 0);
    // Current-process pseudo-handle, initialized buffers with exact Windows ABI sizes.
    if unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut m, m.cb) } == 0
        || unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) } == 0
    {
        return Err("Windows process accounting failed".into());
    }
    Ok(
        json!({"cpu_100ns":k+u,"working_set":m.ws,"private_commit":m.private,"peak_working_set":m.peak_ws,"peak_commit":m.peak_pagefile,"page_faults":m.faults}),
    )
}
fn checked<T>(v: custom_ui::Result<T>) -> Result<T> {
    v.map_err(|e| format!("UI: {e:?}"))
}
fn candidate(s: &str) -> Result<Candidate> {
    match s {
        "custom" => Ok(Candidate::Custom),
        "slint" => Ok(Candidate::Slint),
        _ => Err("candidate must be custom|slint".into()),
    }
}
fn ensure(v: bool, s: &str) -> Result<()> {
    if v { Ok(()) } else { Err(s.into()) }
}
struct View {
    scene: Scene,
    owner: Controller,
    surface: Surface,
    rgb: Vec<u8>,
    previous: Vec<u8>,
    scale: u16,
}
impl View {
    fn new(
        corpus: &std::collections::BTreeMap<String, Value>,
        width: u32,
        height: u32,
        scale: u16,
    ) -> Result<Self> {
        Ok(Self {
            scene: checked(Scene::new(
                tree(&corpus["layout"])?,
                tokens(&corpus["tokens"], &corpus["variants"]["variants"][0])?,
            ))?,
            owner: controller(&corpus["corpus"])?,
            surface: checked(Surface::new(width, height, width * 4, 1))?,
            rgb: vec![0; width as usize * height as usize * 3],
            previous: vec![0; width as usize * height as usize * 4],
            scale,
        })
    }
    fn frame(
        &mut self,
        candidate: Candidate,
        engine: &mut text_core::Engine<'_>,
        trace: bool,
    ) -> Result<Value> {
        let start = Instant::now();
        let a = ALLOCS.load(Relaxed);
        let b = BYTES.load(Relaxed);
        self.scene.dirty.layout |= self.owner.dirty.layout;
        self.scene.layout();
        self.owner.dirty = Dirty::default();
        let layout = start.elapsed().as_nanos() as u64;
        let raster = Instant::now();
        let s = &mut self.surface;
        let (w, h, stride) = (s.width, s.height, s.stride);
        let lease = checked(s.acquire(self.scene.tree.generation))?;
        let mut canvas = checked(Canvas::new(
            checked(s.write(lease))?,
            w,
            h,
            stride,
            self.scale,
        ))?;
        let result = if candidate == Candidate::Slint {
            crate::slint_candidate::paint(&mut self.scene, &self.owner, engine, &mut canvas)
        } else {
            custom_ui::paint::render(
                &mut self.scene,
                self.owner.provider.model(),
                self.owner.composition.as_ref().map(|p| p.preedit()),
                engine,
                &mut canvas,
            )
        };
        let counters = match result {
            Ok(v) => v,
            Err(e) => {
                checked(s.abandon(lease))?;
                return Err(format!("paint {e:?}"));
            }
        };
        let raster_ns = raster.elapsed().as_nanos() as u64;
        let submit = Instant::now();
        checked(s.commit(lease, &[[0, 0, w, h]]))?;
        let pixels = checked(s.read(lease))?;
        for (dest, src) in self.rgb.chunks_exact_mut(3).zip(pixels.chunks_exact(4)) {
            dest.copy_from_slice(&src[..3]);
        }
        std::hint::black_box(&self.rgb);
        let submission_ns = submit.elapsed().as_nanos() as u64;
        let frame_ns = start.elapsed().as_nanos() as u64;
        let audit = Instant::now();
        // Paired tracing overhead includes changed-pixel scan, hash, and counters.
        let changed = if trace {
            Some(
                pixels
                    .chunks_exact(4)
                    .zip(self.previous.chunks_exact(4))
                    .filter(|(a, b)| a != b)
                    .count(),
            )
        } else {
            None
        };
        let digest = if trace { Some(hash(pixels)) } else { None };
        self.previous.copy_from_slice(pixels);
        checked(s.release(lease))?;
        let trace_ns = audit.elapsed().as_nanos() as u64;
        Ok(
            json!({"layout_ns":layout,"raster_ns":raster_ns,"submission_ns":submission_ns,"frame_ns":frame_ns,"trace_ns":trace_ns,"total_ns":start.elapsed().as_nanos() as u64,"missed_60hz_deadline":frame_ns>16_666_667,"changed_pixels":changed,"submitted_damage_pixels":w as u64*h as u64,"rgba_sha256":digest,"allocations":if trace{Some(ALLOCS.load(Relaxed)-a)}else{None},"allocated_bytes":if trace{Some(BYTES.load(Relaxed)-b)}else{None},"surface_bytes":s.allocated_bytes(),"rgb_bytes":self.rgb.capacity(),"audit_previous_bytes":self.previous.capacity(),"glyph_cache_bytes":counters.glyph_cache_bytes,"glyph_cache_entries":counters.glyph_cache_entries,"scratch_peak_bytes":counters.scratch_peak,"shape_work":counters.shape_work,"realized":counters.realized,"touched":counters.touched}),
        )
    }
}

pub fn measure(name: &str, trace: &str, repetition: &str, directory: &Path) -> Result<()> {
    let candidate = candidate(name)?;
    let trace = match trace {
        "on" => true,
        "off" => false,
        _ => return Err("trace must be on|off".into()),
    };
    let repetition: usize = repetition.parse().map_err(|_| "repetition")?;
    ensure(repetition < 5, "repetition must be 0..4")?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let corpus = crate::ui_fixtures::checked_inputs()?;
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("{e:?}"))?;
    let mut views = Vec::new();
    for (w, h) in [(1280, 720), (1920, 1080)] {
        for scale in [100, 150, 200] {
            views.push(View::new(&corpus, w, h, scale)?);
        }
    }
    let warm_clock = Instant::now();
    for warm in 0..50usize {
        let slot = warm_clock + Duration::from_millis(warm as u64 * 100);
        if let Some(delay) = slot.checked_duration_since(Instant::now()) {
            thread::sleep(delay);
        }
        views[warm % 6].frame(candidate, &mut engine, false)?;
    }
    if let Some(delay) =
        (warm_clock + Duration::from_secs(5)).checked_duration_since(Instant::now())
    {
        thread::sleep(delay);
    }
    let warm = 50;
    let warmup_ms = warm_clock.elapsed().as_millis() as u64;
    let mut rows = Vec::with_capacity(300);
    let mut memory = Vec::with_capacity(602);
    COUNT.store(trace, Relaxed);
    let active = Instant::now();
    for n in 0..300usize {
        let slot = active + Duration::from_millis(n as u64 * 100);
        if let Some(delay) = slot.checked_duration_since(Instant::now()) {
            thread::sleep(delay);
        }
        let input = Instant::now();
        let config = n % 6;
        let workload = (n / 6) % 6;
        let variant = (n / 36) % 5;
        let view = &mut views[config];
        // One fixed round-robin schedule. All candidates receive identical requests.
        let t = tokens(&corpus["tokens"], &corpus["variants"]["variants"][variant])?;
        view.scene.tokens = t;
        let reduced = view.scene.tokens.reduced_effects;
        checked(
            view.owner
                .invoke(Action::Effects(reduced), Entry::Automation),
        )?;
        match workload {
            0 => {
                checked(view.owner.invoke(
                    Action::Focus(if n % 2 == 0 { "editor" } else { "search" }.into()),
                    Entry::Shortcut,
                ))?;
            }
            1 => {
                checked(view.owner.invoke(
                    Action::Select(editor_model::Selection::new(
                        0,
                        view.owner.provider.model().document().text().len() as u32,
                    )),
                    Entry::Shortcut,
                ))?;
                checked(view.owner.invoke(
                    Action::Insert("Draft: العربية हिन्दी e\u{301} אבג\n".into()),
                    Entry::Shortcut,
                ))?;
                checked(
                    view.owner
                        .invoke(Action::Search("Draft".into()), Entry::Shortcut),
                )?;
            }
            2 => checked(view.owner.invoke(
                Action::FileScroll(((n * 137) % 9900) as u32),
                Entry::Shortcut,
            ))?,
            3 => {
                checked(view.owner.invoke(
                    Action::Window([40 + (n % 31) as i32, 40, 900 + (n % 17) as i32, 600]),
                    Entry::Automation,
                ))?;
                let mut next = view.scene.tree.clone();
                next.revision += 1;
                for node in &mut next.nodes {
                    if node.id == "editor" {
                        node.rect.0[2] = 560.0 + (n % 17) as f32;
                    }
                }
                checked(view.scene.publish(next, view.scene.tokens.clone()))?;
            }
            4 => {
                checked(view.owner.invoke(Action::LauncherOpen, Entry::Shortcut))?;
                checked(view.owner.invoke(
                    Action::Motion(variant == 2 || variant == 4),
                    Entry::Shortcut,
                ))?;
            }
            _ => {
                let mut next = view.scene.tree.clone();
                next.revision += 1;
                next.generation += 1;
                checked(view.scene.publish(next, view.scene.tokens.clone()))?;
            }
        }
        let dispatch_ns = input.elapsed().as_nanos() as u64;
        let mut row = view.frame(candidate, &mut engine, trace)?;
        row["input_to_ram_submission_ns"] = json!(dispatch_ns + row["frame_ns"].as_u64().unwrap());
        row["n"] = json!(n);
        row["configuration"] = json!(config);
        row["workload"] = json!(workload);
        row["variant"] = json!(variant);
        row["scheduled_lateness_ns"] =
            json!(input.saturating_duration_since(slot).as_nanos() as u64);
        rows.push(row);
        memory.push(
            json!({"phase":"active","ms":active.elapsed().as_millis() as u64,"process":process()?}),
        );
    }
    if let Some(delay) = (active + Duration::from_secs(30)).checked_duration_since(Instant::now()) {
        thread::sleep(delay);
    }
    COUNT.store(false, Relaxed);
    let active_elapsed = active.elapsed().as_nanos() as u64;
    // Settled, eventless RAM adapter. No simulated clock or caret wakeups are hidden.
    let idle = Instant::now();
    let before = process()?;
    let mut render_requests = 0;
    for n in 0..300u64 {
        let deadline = idle + Duration::from_millis((n + 1) * 100);
        if let Some(delay) = deadline.checked_duration_since(Instant::now()) {
            thread::sleep(delay);
        }
        render_requests += views
            .iter()
            .filter(|v| v.scene.dirty.layout || v.scene.dirty.paint)
            .count();
        memory.push(
            json!({"phase":"idle","ms":idle.elapsed().as_millis() as u64,"process":process()?}),
        );
    }
    let after = process()?;
    write_json(&directory.join("frames.json"), &json!(rows))?;
    write_json(&directory.join("process.json"), &json!(memory))?;
    write_json(
        &directory.join("result.json"),
        &json!({"status":"passed","candidate":candidate.name(),"trace":trace,"repetition":repetition,"pid":std::process::id(),"executable_sha256":hash(&fs::read(std::env::current_exe().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?),"protocol_sha256":hash(&fs::read(root().join("config/p2-08-comparison.json")).map_err(|e|e.to_string())?),"warmup_ms":warmup_ms,"warmup_frames":warm,"active_ns":active_elapsed,"idle_ns":idle.elapsed().as_nanos() as u64,"idle_before":before,"idle_after":after,"idle_render_requests":render_requests,"observer_timer_wakes":300,"os_scheduler_wakeups":null,"cold_scope":"fresh process; OS file cache not flushed; warm-up precedes measured frames","guest_tested":false}),
    )?;
    Ok(())
}

pub fn behavior_child(name: &str, snapshot: &Path, mode: &str, directory: &Path) -> Result<()> {
    if mode == "hang" {
        thread::sleep(Duration::from_secs(60));
        return Ok(());
    }
    ensure(mode != "fail-start", "injected candidate startup failure")?;
    let candidate = candidate(name)?;
    let bytes = fs::read(snapshot).map_err(|e| e.to_string())?;
    ensure(bytes.len() <= 65536, "snapshot bound")?;
    let input = if mode == "fail-restore" {
        &bytes[..bytes.len() / 2]
    } else {
        &bytes
    };
    let model = Model::decode(input).map_err(|e| format!("restore: {e:?}"))?;
    let corpus = crate::ui_fixtures::checked_inputs()?;
    let mut view = View::new(&corpus, 1280, 720, 100)?;
    view.owner = checked(Controller::new(model))?;
    let restored = custom::state(&view.owner, &view.scene.tree);
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("{e:?}"))?;
    let first = view.frame(candidate, &mut engine, true)?;
    checked(view.owner.event(
        &view.scene.tree,
        Event {
            sequence: 1,
            time_us: 1,
            revision: view.scene.tree.revision,
            target: "editor".into(),
            kind: EventKind::Text("Continuum".into()),
        },
    ))?;
    let frame = view.frame(candidate, &mut engine, true)?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    fs::write(
        directory.join("after.bin"),
        view.owner
            .provider
            .model()
            .encode()
            .map_err(|e| format!("{e:?}"))?,
    )
    .map_err(|e| e.to_string())?;
    write_json(
        &directory.join("result.json"),
        &json!({"pid":std::process::id(),"build_sha256":hash(&fs::read(std::env::current_exe().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?),"variant":if cfg!(feature="behavior-v2"){2}else{1},"snapshot_sha256":hash(&bytes),"restored":restored,"after":custom::state(&view.owner,&view.scene.tree),"first_frame":first,"changed_frame":frame}),
    )
}

fn child(
    owner: &Controller,
    exe: &Path,
    name: &str,
    snapshot: &Path,
    mode: &str,
    directory: &Path,
    timeout: Duration,
) -> Result<Value> {
    ensure(
        owner.composition.is_none()
            && owner.capture.is_empty()
            && owner.keys.is_empty()
            && owner.buttons.is_empty()
            && owner.pending.is_empty(),
        "helper replacement blocked by active input",
    )?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let mut cmd = Command::new(exe);
    cmd.args(["ui-behavior-child", name])
        .arg(snapshot)
        .arg(mode)
        .arg(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(fs::File::create(directory.join("stderr.log")).map_err(|e| e.to_string())?);
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x08000000);
    let start = Instant::now();
    let mut process = cmd.spawn().map_err(|e| format!("spawn: {e}"))?;
    let pid = process.id();
    let mut killed = false;
    let status = loop {
        match process.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => {
                let _ = process.kill();
                let _ = process.wait();
                return Err(e.to_string());
            }
        }
        if start.elapsed() > timeout {
            killed = true;
            let _ = process.kill();
            break process.wait().map_err(|e| e.to_string())?;
        }
        thread::sleep(Duration::from_millis(5));
    };
    let result = if status.success() {
        Some(
            serde_json::from_slice::<Value>(
                &fs::read(directory.join("result.json")).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };
    Ok(
        json!({"pid":pid,"elapsed_ns":start.elapsed().as_nanos() as u64,"timed_out":killed,"exit":status.code(),"success":status.success(),"cleanup":"reaped","result":result}),
    )
}
pub fn reload(name: &str, replacement: &Path, directory: &Path) -> Result<()> {
    let candidate = candidate(name)?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let corpus = crate::ui_fixtures::checked_inputs()?;
    let mut view = View::new(&corpus, 1280, 720, 100)?;
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("{e:?}"))?;
    let owner_pid = std::process::id();
    let clock = Instant::now();
    let before = custom::state(&view.owner, &view.scene.tree);
    let snapshot = view
        .owner
        .provider
        .model()
        .encode()
        .map_err(|e| format!("{e:?}"))?;
    let mut cycles = Vec::new();
    let mut previous = view.frame(candidate, &mut engine, true)?;
    for n in 0..100 {
        let start = Instant::now();
        let mut next = view.scene.tree.clone();
        next.revision += 1;
        let mut t = view.scene.tokens.clone();
        if n % 2 == 0 {
            t.colors[0][0] = if t.colors[0][0] == 24 { 60 } else { 24 };
        } else {
            for node in &mut next.nodes {
                if node.id == "editor" {
                    node.rect.0[2] = if node.rect.0[2] == 560.0 {
                        520.0
                    } else {
                        560.0
                    };
                }
            }
        }
        checked(view.scene.publish(next, t))?;
        checked(view.owner.reconcile(&view.scene.tree))?;
        let frame = view.frame(candidate, &mut engine, true)?;
        ensure(
            frame["rgba_sha256"] != previous["rgba_sha256"],
            "resource edit must change output",
        )?;
        ensure(
            view.owner
                .provider
                .model()
                .encode()
                .map_err(|e| format!("{e:?}"))?
                == snapshot,
            "resource change mutated model",
        )?;
        cycles.push(json!({"n":n,"kind":if n%2==0{"token"}else{"layout"},"pause_ns":start.elapsed().as_nanos() as u64,"frame":frame,"state":custom::state(&view.owner,&view.scene.tree),"owner_pid":owner_pid,"monotonic_ns":clock.elapsed().as_nanos() as u64}));
        previous = frame;
    }
    // Invalid staged resources and removed focus during composition/capture.
    let mut bad = view.scene.tokens.clone();
    bad.radius = f32::NAN;
    let mut next = view.scene.tree.clone();
    next.revision += 1;
    ensure(
        view.scene.publish(next, bad).is_err(),
        "invalid tokens accepted",
    )?;
    let rev = view.scene.tree.revision;
    let mut next = view.scene.tree.clone();
    next.revision += 1;
    next.nodes
        .iter_mut()
        .find(|n| n.id == "editor")
        .unwrap()
        .resource = "missing.asset".into();
    ensure(
        view.scene.publish(next, view.scene.tokens.clone()).is_err()
            && view.scene.tree.revision == rev,
        "invalid resource published",
    )?;
    let mut transient = View::new(&corpus, 1280, 720, 100)?;
    checked(transient.owner.event(
        &transient.scene.tree,
        Event {
            sequence: 1,
            time_us: 1,
            revision: transient.scene.tree.revision,
            target: "editor".into(),
            kind: EventKind::Composition {
                phase: "start".into(),
                text: "preedit".into(),
            },
        },
    ))?;
    transient.owner.capture = "editor".into();
    let replacement_blocked = child(
        &transient.owner,
        replacement,
        name,
        &directory.join("not-created.bin"),
        "success",
        &directory.join("must-not-launch"),
        Duration::from_secs(10),
    )
    .is_err();
    ensure(replacement_blocked, "active input replacement gate")?;
    let mut next = transient.scene.tree.clone();
    next.revision += 1;
    next.nodes.retain(|n| n.id != "editor");
    for n in &mut next.nodes {
        n.children.retain(|id| id != "editor");
    }
    checked(
        transient
            .scene
            .publish(next, transient.scene.tokens.clone()),
    )?;
    checked(transient.owner.reconcile(&transient.scene.tree))?;
    ensure(
        transient.owner.composition.is_none()
            && transient.owner.capture.is_empty()
            && transient.owner.provider.model().session().focus == "search",
        "removed focus must cancel transient input and focus search",
    )?;
    // Failed candidates never receive publication rights over the stable owner.
    let input = directory.join("input.bin");
    fs::write(&input, &snapshot).map_err(|e| e.to_string())?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let invalid_source = directory.join("failed-build.rs");
    fs::write(
        &invalid_source,
        "fn main() { compile_error!(\"p2-08 owner-alive build failure\"); }",
    )
    .map_err(|e| e.to_string())?;
    let error_file = directory.join("failed-build.stderr.log");
    let build_start = Instant::now();
    let mut command = Command::new(crate::tool("rustc")?);
    command
        .arg("--crate-name")
        .arg("p2_08_failed_behavior")
        .arg(&invalid_source)
        .arg("--out-dir")
        .arg(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(fs::File::create(&error_file).map_err(|e| e.to_string())?);
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x08000000);
    let mut build = command.spawn().map_err(|e| e.to_string())?;
    let build_status = loop {
        match build.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(e) => {
                let _ = build.kill();
                let _ = build.wait();
                return Err(e.to_string());
            }
        }
        if build_start.elapsed() > Duration::from_secs(10) {
            let _ = build.kill();
            let _ = build.wait();
            return Err("failed-build probe timed out".into());
        }
        thread::sleep(Duration::from_millis(5));
    };
    ensure(
        !build_status.success()
            && fs::read_to_string(&error_file)
                .map_err(|e| e.to_string())?
                .contains("p2-08 owner-alive build failure"),
        "wrong failed-build result",
    )?;
    let failed_build = json!({"exit":build_status.code(),"cleanup":"reaped","elapsed_ns":build_start.elapsed().as_nanos() as u64,"owner_pid":owner_pid,"owner_checkpoint_sha256":hash(&view.owner.provider.model().encode().map_err(|e|format!("{e:?}"))?),"diagnostic_sha256":hash(&fs::read(&error_file).map_err(|e|e.to_string())?)});
    let mut helpers = Vec::new();
    for (label, executable, mode) in [
        ("v1", exe.as_path(), "success"),
        ("v2", replacement, "success"),
        ("start-failure", replacement, "fail-start"),
        ("restore-failure", replacement, "fail-restore"),
        ("timeout", replacement, "hang"),
    ] {
        let launch = child(
            &view.owner,
            executable,
            name,
            &input,
            mode,
            &directory.join(label),
            if mode == "hang" {
                Duration::from_millis(100)
            } else {
                Duration::from_secs(10)
            },
        )?;
        if mode == "success" {
            ensure(launch["success"] == true, "behavior child failed")?;
            ensure(
                launch["result"]["snapshot_sha256"] == hash(&snapshot)
                    && launch["result"]["restored"]["checkpoint_sha256"]
                        == before["checkpoint_sha256"],
                "restoration changed checkpoint",
            )?;
            let text = launch["result"]["after"]["document"]
                .as_str()
                .ok_or("child text")?;
            ensure(
                text.contains(if label == "v2" {
                    "CONTINUUM"
                } else {
                    "Continuum"
                }),
                "compiled behavior did not change",
            )?;
            ensure(
                launch["result"]["first_frame"]["rgba_sha256"]
                    != launch["result"]["changed_frame"]["rgba_sha256"],
                "behavior output unchanged",
            )?;
        } else {
            ensure(launch["success"] == false, "injected failure accepted")?;
        }
        ensure(
            view.owner
                .provider
                .model()
                .encode()
                .map_err(|e| format!("{e:?}"))?
                == snapshot,
            "helper mutated authoritative owner",
        )?;
        helpers.push(json!({"label":label,"launch":launch,"owner_checkpoint_sha256":hash(&snapshot),"owner_pid":owner_pid,"owner_monotonic_ns":clock.elapsed().as_nanos() as u64}));
    }
    // Independently reject truncated returned checkpoint before any owner mutation.
    ensure(
        view.owner
            .provider
            .restore(&snapshot[..snapshot.len() / 2])
            .is_err(),
        "truncated restore accepted",
    )?;
    ensure(
        view.owner
            .provider
            .model()
            .encode()
            .map_err(|e| format!("{e:?}"))?
            == snapshot,
        "failed restore mutated owner",
    )?;
    write_json(
        &directory.join("result.json"),
        &json!({"status":"passed","candidate":candidate.name(),"owner_pid":owner_pid,"owner_before":before,"owner_after":custom::state(&view.owner,&view.scene.tree),"owner_elapsed_ns":clock.elapsed().as_nanos() as u64,"cycles":cycles,"helpers":helpers,"failed_build":failed_build,"invalid_tokens_rejected":true,"invalid_resources_rejected":true,"active_composition_gesture_blocks_replacement":replacement_blocked,"removed_focus_cancelled":true,"failed_returned_checkpoint_rejected":true,"helper_results_are_proposals":true,"native_unload":false,"guest_reload":false}),
    )
}
