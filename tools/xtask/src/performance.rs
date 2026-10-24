//! P2-12 selected-core host measurements. No guest or display-completion claims.
use crate::{Result, accounting as ac, custom, hash, reload::Resources, root, write_json};
use custom_ui::{
    Rect,
    input::Controller,
    paint,
    retained::Frame,
    scene::{Dirty, Scene},
};
use editor_model::{Action, Entry, Selection};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{OnceLock, atomic::Ordering::Relaxed},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn checked<T>(v: custom_ui::Result<T>) -> Result<T> {
    v.map_err(|e| format!("selected measurement: {e:?}"))
}
fn need(ok: bool, why: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(why.into()) }
}
fn protocol() -> Result<Value> {
    serde_json::from_slice(
        &fs::read(root().join("config/p2-12-performance.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
fn directory(name: &str) -> Result<PathBuf> {
    let p = root().join(format!(
        "build/m2/P2-12/{name}-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::process::id()
    ));
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}
fn clock() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_nanos() as u64
}
fn sleep_until(deadline: Instant) {
    if let Some(d) = deadline.checked_duration_since(Instant::now()) {
        thread::sleep(d);
    }
}
fn encoded(c: &Controller) -> Result<Vec<u8>> {
    c.provider
        .model()
        .encode()
        .map_err(|e| format!("checkpoint: {e:?}"))
}

// Independent nominal-100ms observer. Its costs are charged to this process;
// actual elapsed timestamps and skipped observer slots remain explicit.
struct Sampler {
    stop: std::sync::mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<Result<Vec<Value>>>>,
}
impl Sampler {
    fn start(phase: &'static str) -> Self {
        let (stop, rx) = std::sync::mpsc::channel();
        let worker = thread::spawn(move || {
            let clock = Instant::now();
            let mut slot = 1u64;
            let mut rows = Vec::with_capacity(2048);
            loop {
                let deadline = clock + Duration::from_millis(slot * 100);
                let delay = deadline.saturating_duration_since(Instant::now());
                if rx.recv_timeout(delay).is_ok() {
                    break;
                }
                let elapsed = clock.elapsed().as_nanos() as u64;
                let next = elapsed / 100_000_000 + 1;
                rows.push(json!({"phase":phase,"elapsed_ns":elapsed,"slot":slot,"missed_observations":next.saturating_sub(slot+1),"process":ac::process()?}));
                slot = next;
            }
            Ok(rows)
        });
        Self {
            stop,
            worker: Some(worker),
        }
    }
    fn finish(mut self) -> Result<Vec<Value>> {
        let _ = self.stop.send(());
        self.worker
            .take()
            .unwrap()
            .join()
            .map_err(|_| "Process sampler panicked".to_string())?
    }
}
impl Drop for Sampler {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct View {
    scene: Scene,
    owner: Controller,
    frame: Frame,
    rgb: Vec<u8>,
    scale: u16,
}
impl View {
    fn new(
        c: &std::collections::BTreeMap<String, Value>,
        w: u32,
        h: u32,
        scale: u16,
    ) -> Result<Self> {
        let source = custom::tree(&c["layout"])?;
        let mut scene = checked(Scene::new(
            source.clone(),
            custom::tokens(&c["tokens"], &c["variants"]["variants"][0])?,
        ))?;
        scene.selected = true;
        let viewport = Rect([
            0.0,
            0.0,
            w as f32 * 100.0 / scale as f32,
            h as f32 * 100.0 / scale as f32,
        ]);
        let plan = crate::selected::reference_plan(&source, viewport.0[2], viewport.0[3])?;
        checked(scene.arrange(&source, &plan, viewport))?;
        Ok(Self {
            scene,
            owner: custom::controller(&c["corpus"])?,
            frame: checked(Frame::new(w, h))?,
            rgb: vec![0; w as usize * h as usize * 3],
            scale,
        })
    }
    fn draw(&mut self, engine: &mut text_core::Engine<'_>, trace: bool) -> Result<Value> {
        let start = Instant::now();
        let a = ac::ALLOCS.load(Relaxed);
        let b = ac::BYTES.load(Relaxed);
        self.scene.dirty.layout |= self.owner.dirty.layout;
        self.scene.dirty.paint |= self.owner.dirty.paint;
        self.scene.dirty.semantics |= self.owner.dirty.semantics;
        self.owner.dirty = Dirty::default();
        self.scene.visual_caret = self.owner.visual_caret;
        self.scene.layout();
        let layout = start.elapsed().as_nanos() as u64;
        self.frame.clock = if trace { Some(clock) } else { None };
        let raster = Instant::now();
        let (damage, c) = checked(self.frame.draw(
            &mut self.scene,
            self.owner.provider.model(),
            self.owner.composition.as_ref().map(|x| x.preedit()),
            engine,
            self.scale,
        ))?;
        let raster_compose = raster.elapsed().as_nanos() as u64;
        let submit = Instant::now();
        // RAM consumer has independent storage. Copy only the validated damage.
        for [x, y, w, h] in &damage.regions {
            for row in *y..y + h {
                for col in *x..x + w {
                    let i = (row * self.frame.width + col) as usize;
                    self.rgb[i * 3..i * 3 + 3]
                        .copy_from_slice(&self.frame.pixels[i * 4..i * 4 + 3]);
                }
            }
        }
        let submission = submit.elapsed().as_nanos() as u64;
        let frame_ns = start.elapsed().as_nanos() as u64;
        let audit = Instant::now();
        let digest = if trace {
            Some(hash(&self.frame.pixels))
        } else {
            None
        };
        let (cache_entries, cache_bytes) = engine.cache_usage();
        let (layout_entries, layout_bytes, layout_hits, layout_misses) =
            engine.layout_cache_usage();
        Ok(
            json!({"layout_ns":layout,"raster_compose_ns":raster_compose,"raster_ns":if trace{Some(raster_compose.saturating_sub(c.composition_ns))}else{None},"compose_ns":if trace{Some(c.composition_ns)}else{None},"submission_ns":submission,"frame_ns":frame_ns,"audit_ns":audit.elapsed().as_nanos() as u64,"rgba_sha256":digest,"painted":!damage.regions.is_empty(),"damage_pixels":damage.pixels(),"regions":damage.regions,"fallback":damage.fallback,"allocations":if trace{Some(ac::ALLOCS.load(Relaxed)-a)}else{None},"allocated_bytes":if trace{Some(ac::BYTES.load(Relaxed)-b)}else{None},"rust_live":ac::LIVE.load(Relaxed),"rust_peak":ac::PEAK.load(Relaxed),"frame_bytes":self.frame.bytes(),"rgb_bytes":self.rgb.capacity(),"layout_cache_entries":layout_entries,"layout_cache_bytes":layout_bytes,"layout_cache_hits":layout_hits,"layout_cache_misses":layout_misses,"cache_entries":cache_entries,"cache_bytes":cache_bytes,"paint_scratch_peak":c.scratch_peak,"touched":c.touched,"shape_work":c.shape_work,"realized":c.realized,"missed_60hz":frame_ns>16_666_667}),
        )
    }
    fn script(&mut self, n: usize, trace: bool) -> Result<Value> {
        let input = Instant::now();
        let workload = (n / 6) % 6;
        self.owner.phase_clock = if trace { Some(clock) } else { None };
        self.owner.dispatch_ns = 0;
        self.owner.state_ns = 0;
        let action = match workload {
            0 => Action::Focus(
                if (n / 36) % 2 == 0 {
                    "editor"
                } else {
                    "search"
                }
                .into(),
            ),
            1 => Action::Search(
                if (n / 36) % 2 == 0 {
                    "Draft"
                } else {
                    "العربية"
                }
                .into(),
            ),
            2 => Action::FileScroll((n as u32 * 137) % 9900),
            3 => Action::Window([40 + (n % 31) as i32, 40, 900 + (n % 17) as i32, 600]),
            4 => Action::Effects((n / 36) % 2 == 1),
            _ => Action::Motion((n / 36) % 2 == 1),
        };
        let input_ns = input.elapsed().as_nanos() as u64;
        let dispatch = Instant::now();
        checked(self.owner.invoke(action, Entry::Automation))?;
        if workload == 1 {
            checked(self.owner.invoke(
                Action::Select(Selection::new(
                    0,
                    self.owner.provider.model().document().text().len() as u32,
                )),
                Entry::Shortcut,
            ))?;
            checked(self.owner.invoke(
                Action::Insert("Draft: العربية हिन्दी e\u{301} אבג\n".into()),
                Entry::Shortcut,
            ))?;
        }
        if workload == 3 || workload == 5 {
            let mut next = self.scene.tree.clone();
            next.revision += 1;
            if workload == 5 {
                next.generation += 1;
            }
            for node in &mut next.nodes {
                if node.id == "settings" {
                    if workload == 3 {
                        node.rect.0[2] = 240.0 + (n % 17) as f32;
                    }
                    node.rect.0[0] += if (n / 36) % 2 == 0 { 1.0 } else { -1.0 };
                }
            }
            checked(self.scene.publish(next, self.scene.tokens.clone()))?;
        }
        if workload == 4 {
            checked(self.scene.motion.transition(
                "launcher",
                0.9,
                if (n / 36) % 2 == 0 { 0.6 } else { 0.9 },
                n as u64 * 100_000,
                self.owner.provider.model().preferences().reduced_motion,
            ))?;
            checked(self.scene.tick(n as u64 * 100_000 + 60_000))?;
        }
        Ok(
            json!({"workload":workload,"input_ns":input_ns,"dispatch_state_total_ns":dispatch.elapsed().as_nanos() as u64,"dispatch_ns":if trace{Some(self.owner.dispatch_ns)}else{None},"state_ns":if trace{Some(self.owner.state_ns)}else{None}}),
        )
    }
    fn oracle(&mut self, engine: &mut text_core::Engine<'_>) -> Result<(String, u64)> {
        let mut pixels = vec![0; self.frame.pixels.len()];
        let start = Instant::now();
        let mut cv = checked(paint::Canvas::new(
            &mut pixels,
            self.frame.width,
            self.frame.height,
            self.frame.width * 4,
            self.scale,
        ))?;
        checked(paint::render(
            &mut self.scene,
            self.owner.provider.model(),
            self.owner.composition.as_ref().map(|x| x.preedit()),
            engine,
            &mut cv,
        ))?;
        let ns = start.elapsed().as_nanos() as u64;
        need(
            pixels == self.frame.pixels,
            "Incremental/full pixel mismatch",
        )?;
        Ok((hash(&pixels), ns))
    }
}

pub fn damage() -> Result<()> {
    let run = directory("damage")?;
    let p = protocol()?;
    write_json(&run.join("protocol.json"), &p)?;
    let corpus = crate::ui_fixtures::checked_inputs()?;
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("{e:?}"))?;
    // Exercise LRU eviction and every key field against uncached shaping.
    for i in 0..300 {
        let text = format!("Cache {}: العربية हिन्दी office e\u{301}", i / 6);
        let mut cfg = text_core::Config {
            mono: false,
            size: 14,
            scale: 100,
            width: 320,
            ligatures: true,
        };
        match i % 6 {
            1 => cfg.mono = true,
            2 => cfg.size = 18,
            3 => cfg.scale = 150,
            4 => cfg.width = 170,
            5 => cfg.ligatures = false,
            _ => {}
        }
        let direct = engine.layout(&text, cfg).map_err(|e| format!("{e:?}"))?;
        let cached = engine
            .layout_cached(&text, cfg)
            .map_err(|e| format!("{e:?}"))?;
        let hit = engine
            .layout_cached(&text, cfg)
            .map_err(|e| format!("{e:?}"))?;
        need(
            direct.lines == cached.lines
                && direct.graphemes == cached.graphemes
                && direct.missing == cached.missing
                && direct.lines == hit.lines
                && hit.shape_work == 0,
            "Layout cache differs from uncached oracle",
        )?;
        let (entries, bytes, _, _) = engine.layout_cache_usage();
        need(
            entries <= 128 && bytes <= 2 * 1024 * 1024,
            "Layout LRU exceeded budget",
        )?;
    }
    let mut rows = Vec::new();
    for g in p["geometry"].as_array().unwrap() {
        let mut v = View::new(
            &corpus,
            g[0].as_u64().unwrap() as u32,
            g[1].as_u64().unwrap() as u32,
            g[2].as_u64().unwrap() as u16,
        )?;
        for n in 0..18 {
            let mut label = "baseline";
            match n {
                1 => {
                    label = "reduced-effects";
                    checked(v.owner.invoke(Action::Effects(true), Entry::Automation))?;
                }
                2 => {
                    label = "scroll";
                    checked(v.owner.invoke(Action::FileScroll(537), Entry::Automation))?;
                }
                3 => {
                    label = "type";
                    checked(v.owner.invoke(
                        Action::Insert(" unsaved العربية e\u{301}".into()),
                        Entry::Automation,
                    ))?;
                }
                4 => {
                    label = "focus";
                    checked(
                        v.owner
                            .invoke(Action::Focus("search".into()), Entry::Automation),
                    )?;
                }
                5 => {
                    label = "old-new-translucent-bounds";
                    let mut t = v.scene.tree.clone();
                    t.revision += 1;
                    for q in &mut t.nodes {
                        if q.id == "settings" {
                            q.rect.0[0] -= 23.5;
                            q.rect.0[2] += 11.25;
                            q.opacity = 0.55;
                        }
                    }
                    checked(v.scene.publish(t, v.scene.tokens.clone()))?;
                }
                6 => {
                    label = "rounded-clipped-descendants";
                    let mut t = v.scene.tree.clone();
                    t.revision += 1;
                    for q in &mut t.nodes {
                        if q.id == "editor-window" {
                            q.clip = true;
                            q.rect.0[2] -= 31.5;
                        }
                    }
                    checked(v.scene.publish(t, v.scene.tokens.clone()))?;
                }
                7 => {
                    label = "reorder-overlap";
                    let mut t = v.scene.tree.clone();
                    t.revision += 1;
                    let root = t.root.clone();
                    t.nodes
                        .iter_mut()
                        .find(|q| q.id == root)
                        .unwrap()
                        .children
                        .reverse();
                    checked(v.scene.publish(t, v.scene.tokens.clone()))?;
                }
                8 => {
                    label = "exposed-removal";
                    if v.scene.tree.node("settings").is_ok() {
                        checked(v.scene.retire("settings"))?;
                    }
                }
                9 => {
                    label = "backdrop-full";
                    checked(v.owner.invoke(Action::Effects(false), Entry::Automation))?;
                }
                10 => {
                    label = "background-under-filter";
                    v.scene.tokens.colors[0] = [49, 25, 57, 255];
                    v.scene.dirty.paint = true;
                }
                11 => {
                    label = "scale-change";
                    v.scale = if v.scale == 150 { 200 } else { 150 };
                    v.scene.dirty.paint = true;
                }
                12 => {
                    label = "caret";
                    v.scene.caret_hidden = true;
                    v.scene.dirty.paint = true;
                }
                13 => {
                    label = "preedit";
                    checked(v.owner.event(
                        &v.scene.tree,
                        custom_ui::input::Event {
                            sequence: 1,
                            time_us: 1,
                            revision: v.scene.tree.revision,
                            target: "editor".into(),
                            kind: custom_ui::input::EventKind::Focus,
                        },
                    ))?;
                    checked(v.owner.event(
                        &v.scene.tree,
                        custom_ui::input::Event {
                            sequence: 2,
                            time_us: 2,
                            revision: v.scene.tree.revision,
                            target: "editor".into(),
                            kind: custom_ui::input::EventKind::Composition {
                                phase: "start".into(),
                                text: "preedit العربية".into(),
                            },
                        },
                    ))?;
                }
                14 => {
                    v.owner.composition = None;
                    label = "reduced-return";
                    checked(v.owner.invoke(Action::Effects(true), Entry::Automation))?;
                }
                15 => {
                    label = "compatible-motion";
                    checked(v.scene.motion.transition("launcher", 0.9, 0.5, 0, false))?;
                    checked(v.scene.tick(60_000))?;
                }
                16 => {
                    label = "paint-invalidation";
                    v.scene.dirty.paint = true;
                }
                17 => {
                    label = "static-no-paint";
                    v.scene.dirty.paint = false;
                    v.owner.dirty = Dirty::default();
                }
                _ => {}
            }
            let row = v.draw(&mut engine, true)?;
            let (digest, full_ns) = v
                .oracle(&mut engine)
                .map_err(|e| format!("{e}: geometry {g}, case {label}"))?;
            if n == 17 {
                need(row["painted"] == false, "Static frame repainted")?;
            }
            rows.push(json!({"geometry":g,"case":label,"incremental":row,"full_rgba_sha256":digest,"full_render_ns":full_ns,"max_channel_error":0}));
        }
    }
    let mut fragmented = custom_ui::retained::Damage::new(1920, 1080);
    for i in 0..33 {
        fragmented.add(Rect([i as f32 * 4.0, 0.0, 1.0, 1.0]), 100);
    }
    need(fragmented.fallback, "Fragmentation did not fall back")?;
    write_json(
        &run.join("damage.json"),
        &json!({"status":"passed","pixel_tolerance":0,"cases":rows,"fragmentation_fallback":true,"guest":false}),
    )?;
    println!("UI damage checks passed: {}", run.display());
    Ok(())
}

pub fn measure(trace: &str, repetition: &str, run: &Path) -> Result<()> {
    let trace = match trace {
        "on" => true,
        "off" => false,
        _ => return Err("trace must be on|off".into()),
    };
    let rep: usize = repetition.parse().map_err(|_| "repetition")?;
    need(rep < 5, "repetition must be 0..4")?;
    fs::create_dir_all(run).map_err(|e| e.to_string())?;
    let p = protocol()?;
    write_json(&run.join("protocol.json"), &p)?;
    let corpus = crate::ui_fixtures::checked_inputs()?;
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("{e:?}"))?;
    let mut views = Vec::new();
    for g in p["geometry"].as_array().unwrap() {
        views.push(View::new(
            &corpus,
            g[0].as_u64().unwrap() as u32,
            g[1].as_u64().unwrap() as u32,
            g[2].as_u64().unwrap() as u16,
        )?);
    }
    let warm = Instant::now();
    for n in 0..50 {
        sleep_until(warm + Duration::from_millis(n * 100));
        views[n as usize % 6].draw(&mut engine, false)?;
    }
    sleep_until(warm + Duration::from_secs(5));
    let warmup_ns = warm.elapsed().as_nanos() as u64;
    let mut rows = Vec::with_capacity(300);
    ac::COUNT.store(trace, Relaxed);
    let active = Instant::now();
    let active_before = ac::process()?;
    let sampler = Sampler::start("active");
    for n in 0..300 {
        let slot = active + Duration::from_millis(n as u64 * 100);
        sleep_until(slot);
        let late = Instant::now().saturating_duration_since(slot).as_nanos() as u64;
        let v = &mut views[n % 6];
        let input = v.script(n, trace)?;
        let mut row = v.draw(&mut engine, trace)?;
        row["input"] = input;
        row["n"] = json!(n);
        row["geometry_index"] = json!(n % 6);
        row["lateness_ns"] = json!(late);
        row["input_to_ram_submission_ns"] = json!(
            row["frame_ns"].as_u64().unwrap()
                + row["input"]["input_ns"].as_u64().unwrap()
                + row["input"]["dispatch_state_total_ns"].as_u64().unwrap()
        );
        rows.push(row);
    }
    sleep_until(active + Duration::from_secs(30));
    let active_after = ac::process()?;
    let active_ns = active.elapsed().as_nanos() as u64;
    let mut memory = sampler.finish()?;
    ac::COUNT.store(false, Relaxed);
    for v in &mut views {
        v.scene.motion.settle();
        v.scene.dirty.paint = true;
        v.draw(&mut engine, false)?;
    }
    let idle_base = views
        .iter()
        .map(|v| v.scene.motion.now())
        .max()
        .unwrap_or(0);
    let idle = Instant::now();
    let before = ac::process()?;
    let sampler = Sampler::start("idle");
    let mut idle_frames = Vec::with_capacity(240);
    let mut idle_phases = Vec::new();
    let mut timer_wakes = 0;
    let mut static_frames = 0;
    ac::COUNT.store(trace, Relaxed);
    for (phase, seconds) in [
        ("static", 10),
        ("clock-caret", 10),
        ("hidden", 5),
        ("minimized", 5),
    ] {
        let phase_start = Instant::now();
        let phase_before = ac::process()?;
        let frame_start = idle_frames.len();
        for v in &mut views {
            checked(v.scene.motion.timers(
                phase == "clock-caret",
                phase == "clock-caret",
                false,
                idle_base + idle.elapsed().as_micros() as u64,
                false,
            ))?;
        }
        if phase == "static" {
            for v in &mut views {
                let row = v.draw(&mut engine, false)?;
                static_frames += usize::from(row["painted"] == true);
            }
            need(static_frames == 0, "Static scene perpetually repainted")?;
        }
        let end = phase_start + Duration::from_secs(seconds);
        while Instant::now() < end {
            if phase != "clock-caret" {
                sleep_until(end);
                break;
            }
            let now = idle_base + idle.elapsed().as_micros() as u64;
            let due = views
                .iter()
                .filter_map(|v| v.scene.motion.next_deadline())
                .min()
                .unwrap_or(now + 1_000_000);
            sleep_until((Instant::now() + Duration::from_micros(due.saturating_sub(now))).min(end));
            if Instant::now() >= end {
                break;
            }
            // At most one tick per view, using actual elapsed time. Motion skips
            // elapsed periods in O(1); never replay overdue observer slots.
            for (i, v) in views.iter_mut().enumerate() {
                let now = idle_base + idle.elapsed().as_micros() as u64;
                let tick = checked(v.scene.tick(now))?;
                v.scene.caret_hidden = !v.scene.motion.caret_visible;
                if tick.paint || tick.clock {
                    timer_wakes += 1;
                    let frame = v.draw(&mut engine, trace)?;
                    idle_frames.push(json!({"elapsed_ns":idle.elapsed().as_nanos() as u64,"geometry_index":i,"phase":phase,"frame":frame}));
                }
            }
        }
        idle_phases.push(json!({"phase":phase,"elapsed_ns":phase_start.elapsed().as_nanos() as u64,"frames":idle_frames.len()-frame_start,"before":phase_before,"after":ac::process()?}));
    }
    let after = ac::process()?;
    ac::COUNT.store(false, Relaxed);
    let idle_ns = idle.elapsed().as_nanos() as u64;
    let idle_memory = sampler.finish()?;
    let observer_wakes = idle_memory.len();
    memory.extend(idle_memory);
    let missed_samples = memory
        .iter()
        .map(|r| r["missed_observations"].as_u64().unwrap())
        .sum::<u64>();
    write_json(&run.join("frames.json"), &json!(rows))?;
    write_json(&run.join("process.json"), &json!(memory))?;
    write_json(&run.join("idle-frames.json"), &json!(idle_frames))?;
    write_json(
        &run.join("result.json"),
        &json!({"status":"passed","trace":trace,"repetition":rep,"pid":std::process::id(),"warmup_ns":warmup_ns,"active_ns":active_ns,"active_before":active_before,"active_after":active_after,"idle_ns":idle_ns,"idle_before":before,"idle_after":after,"idle_phases":idle_phases,"observer_wakes":observer_wakes,"process_sample_misses":missed_samples,"clock_caret_surface_wakes":timer_wakes,"static_frames":static_frames,"hidden_frames":0,"minimized_frames":0,"os_scheduler_wakes":null,"accessibility_timer":"none; synchronous semantic snapshots","caret_period_us":500000,"clock_period_us":1000000,"executable_sha256":hash(&fs::read(std::env::current_exe().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?),"guest":false}),
    )?;
    Ok(())
}

pub fn stability() -> Result<()> {
    let run = directory("stability")?;
    for (w, h) in [(1280, 720), (1920, 1080)] {
        let child = run.join(format!("{w}x{h}"));
        fs::create_dir_all(&child).map_err(|e| e.to_string())?;
        stability_at(&child, w, h)?;
    }
    write_json(
        &run.join("result.json"),
        &json!({"status":"passed","measured_cycles_per_resolution":100,"resolutions":[[1280,720],[1920,1080]],"guest":false}),
    )?;
    println!("UI stability checks passed: {}", run.display());
    Ok(())
}
fn stability_at(run: &Path, width: u32, height: u32) -> Result<()> {
    let p = protocol()?;
    write_json(&run.join("protocol.json"), &p)?;
    let corpus = crate::ui_fixtures::checked_inputs()?;
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("{e:?}"))?;
    let mut resources = Resources::new()?;
    let payload = resources.payload();
    let mut v = View::new(&corpus, width, height, 100)?;
    v.scene = checked(Scene::new(
        custom::tree(&payload["layout"])?,
        custom::tokens(&payload["tokens"], &payload["variant"])?,
    ))?;
    v.scene.selected = true;
    let initial = v.scene.tree.clone();
    let plan = checked(custom_ui::layout::Plan::anchored(&initial))?;
    checked(v.scene.arrange(
        &initial,
        &plan,
        Rect([0.0, 0.0, width as f32, height as f32]),
    ))?;
    for action in [
        Action::Insert(" unsaved".into()),
        Action::Search("a".into()),
        Action::Scroll(3),
        Action::FileScroll(537),
        Action::Window([90, 70, width as i32, height as i32]),
        Action::Insert("history".into()),
        Action::Undo,
        Action::Select(Selection::new(2, 5)),
    ] {
        checked(v.owner.invoke(action, Entry::Automation))?;
    }
    let checkpoint = encoded(&v.owner)?;
    let provider_address = &v.owner.provider as *const _ as usize;
    v.draw(&mut engine, true)?;
    let source = run.join("source.json");
    let file = run.join("completed.json");
    // Preallocate observation storage; stream rows to disk so the observation
    // log itself cannot masquerade as retained UI generation growth.
    let mut observations = fs::File::create(run.join("cycles.jsonl")).map_err(|e| e.to_string())?;
    use std::io::Write;
    let mut trigger = crate::reload::Trigger::default();
    let mut baseline = None;
    let mut failures = Vec::new();
    for cycle in 0..105 {
        let mut next = resources.payload();
        let id = resources.id() + 1;
        next["generation"] = json!(id);
        next["base_generation"] = json!(id - 1);
        next["build_id"] = json!(format!("stability-{id}"));
        next["tokens"]["generation"] = json!(id);
        next["layout"]["generation"] = json!(id);
        next["tokens"]["colors"]["panel"] = if cycle % 2 == 0 {
            json!([45, 55, 76, 255])
        } else {
            json!([35, 45, 66, 255])
        };
        next["variant"]["colors"] = next["tokens"]["colors"].clone();
        for node in next["layout"]["nodes"].as_array_mut().unwrap() {
            if node["id"] == "settings" {
                node["rect"][0] = json!(1000 + cycle % 2 * 8);
            }
        }
        write_json(&source, &next)?;
        let build = Instant::now();
        crate::reload::build(&source, &file)?;
        let build_ns = build.elapsed().as_nanos() as u64;
        let old = resources.reader();
        let complete = Instant::now();
        need(
            !trigger.poll(&file, resources.now())?,
            "Reload bypassed first debounce",
        )?;
        thread::sleep(Duration::from_millis(100));
        need(
            trigger.poll(&file, resources.now())?,
            "Completed reload did not become eligible",
        )?;
        let before_live = ac::LIVE.load(Relaxed);
        ac::PEAK.store(before_live, Relaxed);
        resources.apply(
            &file,
            &mut v.scene,
            &mut v.owner,
            &mut engine,
            width,
            height,
            100,
        )?;
        need(
            encoded(&v.owner)? == checkpoint,
            "Reload checkpoint changed",
        )?;
        need(
            &v.owner.provider as *const _ as usize == provider_address,
            "Document owner replaced",
        )?;
        let frame = v.draw(&mut engine, true)?;
        resources.first_frame();
        let visible_ns = complete.elapsed().as_nanos() as u64;
        let peak_live = ac::PEAK.load(Relaxed);
        let peak_process = ac::process()?;
        need(resources.live() == 2, "Delayed old reader not retained")?;
        drop(old);
        thread::sleep(Duration::from_millis(100));
        need(
            resources.live() == 1,
            "Old generation retained after settle",
        )?;
        let resource = resources.query();
        let (entries, bytes) = engine.cache_usage();
        let (layout_entries, layout_bytes, layout_hits, layout_misses) =
            engine.layout_cache_usage();
        need(
            layout_entries <= 128 && layout_bytes <= 2 * 1024 * 1024,
            "Layout cache bound exceeded",
        )?;
        need(
            entries <= 128 && bytes <= 1024 * 1024,
            "Cache bound exceeded",
        )?;
        // Diagnostic trace has a separate bounded owner; flush it after capture.
        let trace = std::mem::take(&mut resources.trace);
        write_json(&run.join("last-trace.json"), &json!(trace))?;
        drop(trace);
        let process = ac::process()?;
        let live = ac::LIVE.load(Relaxed);
        if cycle == 4 {
            baseline = Some((live, process.clone()));
        }
        let mut growth = json!(null);
        if cycle >= 5 {
            let (base, bp) = baseline.as_ref().unwrap();
            let rust = live.saturating_sub(*base);
            let commit = process["private_commit"]
                .as_u64()
                .unwrap()
                .saturating_sub(bp["private_commit"].as_u64().unwrap());
            let ws = process["working_set"]
                .as_u64()
                .unwrap()
                .saturating_sub(bp["working_set"].as_u64().unwrap());
            growth = json!({"rust_live":rust,"private_commit":commit,"working_set":ws});
            if rust > 65536 || commit > 4194304 || ws > 8388608 {
                failures.push(cycle);
            }
        }
        let row = json!({"cycle":cycle,"measured":cycle>=5,"generation":id,"checkpoint_sha256":hash(&checkpoint),"owner_address_unchanged":true,"build_ns":build_ns,"completed_build_to_ram_visible_ns":visible_ns,"before_rust_live":before_live,"peak_rust_live":peak_live,"settled_rust_live":live,"peak_process":peak_process,"settled_process":process,"growth_from_baseline":growth,"resources":resource,"layout_cache_entries":layout_entries,"layout_cache_bytes":layout_bytes,"layout_cache_hits":layout_hits,"layout_cache_misses":layout_misses,"cache_entries":entries,"cache_bytes":bytes,"frame":frame});
        writeln!(observations, "{}", row).map_err(|e| e.to_string())?;
    }
    // Two held readers fill all three roots. Fourth staging must reject before
    // allocation, preserve active pixels/state, then recover when readers release.
    let mut held = Vec::new();
    for _ in 0..2 {
        held.push(resources.reader());
        let mut next = resources.payload();
        let id = resources.id() + 1;
        next["generation"] = json!(id);
        next["base_generation"] = json!(id - 1);
        next["tokens"]["generation"] = json!(id);
        next["layout"]["generation"] = json!(id);
        write_json(&file, &next)?;
        resources.apply(
            &file,
            &mut v.scene,
            &mut v.owner,
            &mut engine,
            width,
            height,
            100,
        )?;
    }
    let active = resources.id();
    let state = encoded(&v.owner)?;
    let mut next = resources.payload();
    next["generation"] = json!(active + 1);
    next["base_generation"] = json!(active);
    next["tokens"]["generation"] = json!(active + 1);
    next["layout"]["generation"] = json!(active + 1);
    write_json(&file, &next)?;
    let rejected = resources
        .apply(
            &file,
            &mut v.scene,
            &mut v.owner,
            &mut engine,
            width,
            height,
            100,
        )
        .is_err();
    need(
        rejected && resources.id() == active && encoded(&v.owner)? == state,
        "Headroom rejection changed active state",
    )?;
    drop(held);
    need(resources.live() == 1, "Headroom readers not retired")?;
    resources.apply(
        &file,
        &mut v.scene,
        &mut v.owner,
        &mut engine,
        width,
        height,
        100,
    )?;
    let active = resources.id();
    let scene_revision = v.scene.tree.revision;
    let before_state = encoded(&v.owner)?;
    let before_pixels = hash(&v.frame.pixels);
    let completed_before = fs::read(&file).map_err(|e| e.to_string())?;
    let mut invalid = resources.payload();
    invalid["generation"] = json!(active + 1);
    invalid["base_generation"] = json!(active);
    invalid["tokens"]["generation"] = json!(active + 1);
    invalid["layout"]["generation"] = json!(active + 1);
    invalid["tokens"]["colors"]["surface"][3] = json!(0);
    invalid["variant"]["colors"]["surface"][3] = json!(0);
    write_json(&source, &invalid)?;
    let builder_rejected = crate::reload::build(&source, &file).is_err();
    need(
        builder_rejected && fs::read(&file).map_err(|e| e.to_string())? == completed_before,
        "Nonopaque builder replaced completed output",
    )?;
    let api_rejected = resources
        .apply(
            &source,
            &mut v.scene,
            &mut v.owner,
            &mut engine,
            width,
            height,
            100,
        )
        .is_err();
    need(
        api_rejected
            && resources.id() == active
            && v.scene.tree.revision == scene_revision
            && encoded(&v.owner)? == before_state
            && hash(&v.frame.pixels) == before_pixels,
        "Nonopaque resource changed published state",
    )?;
    let opacity = json!({"builder_rejected":builder_rejected,"api_rejected":api_rejected,"active_before":active,"active_after":resources.id(),"scene_revision_before":scene_revision,"scene_revision_after":v.scene.tree.revision,"checkpoint_before":hash(&before_state),"checkpoint_after":hash(&encoded(&v.owner)?),"frame_before":before_pixels,"frame_after":hash(&v.frame.pixels),"rejection":resources.status});
    write_json(&run.join("opacity-negative.json"), &opacity)?;
    write_json(
        &run.join("result.json"),
        &json!({"status":if failures.is_empty(){"passed"}else{"failed"},"warmup_cycles":5,"measured_cycles":100,"growth_failures":failures,"checkpoint_sha256":hash(&checkpoint),"headroom_rejected":rejected,"reader_release_recovery":true,"opacity_rejected":builder_rejected&&api_rejected,"scope":"same-process RAM consumer; no native display latency","guest":false}),
    )?;
    need(
        failures.is_empty(),
        "Reload resource growth exceeded preregistered tolerance",
    )?;
    Ok(())
}
