//! P2-10 selected-core integration and evidence. No file reload or guest claims.
use crate::{Result, custom, hash, root, write_json};
use custom_ui::{
    input::{Controller, Event, EventKind},
    layout::{Placement, Plan, Size},
    scene::{Dirty, Scene, Tree},
    semantics::{Command, Snapshot},
    surface::Surface,
    *,
};
use serde_json::{Value, json};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
fn check<T>(v: custom_ui::Result<T>) -> Result<T> {
    v.map_err(|e| format!("selected: {e:?}"))
}
fn require(v: bool, reason: &str) -> Result<()> {
    if v { Ok(()) } else { Err(reason.into()) }
}

pub(crate) fn reference_plan(source: &Tree, width: f32, height: f32) -> Result<Plan> {
    require(
        width >= 640.0 && height >= 360.0 && width <= 4096.0 && height <= 2160.0,
        "Selected shell viewport bounds",
    )?;
    let mut plan = check(Plan::anchored(source))?;
    for (id, w, h) in [
        ("editor-window", width - 220.0, height - 32.0),
        ("files-window", 192.0, (height - 32.0).min(1488.0)),
    ] {
        check(plan.rule_mut(id))?.size = [Size::Fixed(w), Size::Fixed(h)];
    }
    for id in ["search", "editor", "status"] {
        if let Placement::Anchored { stretch, .. } = &mut check(plan.rule_mut(id))?.placement {
            *stretch = [true, id == "editor"];
        }
    }
    for (id, x, y) in [
        ("files-window", width - 200.0, 8.0),
        ("settings", width - 260.0, 92.0),
        ("launcher", width / 2.0, height / 2.0),
        ("status", 96.0, height - 48.0),
    ] {
        if source.node(id).is_err() {
            continue;
        }
        // status coordinates are local to editor-window.
        check(plan.rule_mut(id))?.placement = Placement::Anchored {
            x,
            y,
            stretch: [id == "status", false],
        };
    }
    Ok(plan)
}
pub(crate) fn snapshot_json(s: &Snapshot) -> Value {
    json!({"scene_revision":s.scene_revision,"model_revision":s.model_revision,"items":s.items.iter().map(|n|json!({"id":n.id,"parent":n.parent,"role":n.role,"name":n.name,"value":n.value,"actions":n.actions,"bounds":n.bounds.0,"focus":n.focused,"checked":n.checked,"selected":n.selected,"position":n.position,"count":n.count})).collect::<Vec<_>>()})
}
fn send(
    c: &mut Controller,
    t: &Tree,
    seq: &mut u64,
    kind: EventKind,
    target: &str,
    engine: &mut text_core::Engine<'_>,
    tokens: &custom_ui::scene::Tokens,
    scale: u16,
) -> Result<()> {
    *seq += 1;
    check(c.event_shaped(
        t,
        Event {
            sequence: *seq,
            time_us: *seq * 1000,
            revision: t.revision,
            target: target.into(),
            kind,
        },
        engine,
        tokens,
        scale,
    ))
}
fn tab(
    c: &mut Controller,
    t: &Tree,
    seq: &mut u64,
    backwards: bool,
    count: usize,
    engine: &mut text_core::Engine<'_>,
    tokens: &custom_ui::scene::Tokens,
    scale: u16,
) -> Result<()> {
    if backwards {
        let focus = c.provider.model().session().focus.clone();
        send(
            c,
            t,
            seq,
            EventKind::Key {
                usage: 229,
                down: true,
            },
            &focus,
            engine,
            tokens,
            scale,
        )?;
    }
    for _ in 0..count {
        let focus = c.provider.model().session().focus.clone();
        send(
            c,
            t,
            seq,
            EventKind::Key {
                usage: 43,
                down: true,
            },
            &focus,
            engine,
            tokens,
            scale,
        )?;
        let focus = c.provider.model().session().focus.clone();
        send(
            c,
            t,
            seq,
            EventKind::Key {
                usage: 43,
                down: false,
            },
            &focus,
            engine,
            tokens,
            scale,
        )?;
    }
    if backwards {
        let focus = c.provider.model().session().focus.clone();
        send(
            c,
            t,
            seq,
            EventKind::Key {
                usage: 229,
                down: false,
            },
            &focus,
            engine,
            tokens,
            scale,
        )?;
    }
    Ok(())
}
pub fn execute() -> Result<()> {
    let run = root().join(format!(
        "build/m2/P2-10/run-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id()
    ));
    fs::create_dir_all(&run).map_err(|e| e.to_string())?;
    let outcome = (|| -> Result<Value> {
        let corpus = crate::ui_fixtures::checked_inputs()?;
        let fonts = crate::text::inputs()?;
        let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
        let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("font {e:?}"))?;
        let source = custom::tree(&corpus["layout"])?;
        let mut matrix = Vec::new();
        let mut workflow = Vec::new();
        let mut texts = Vec::new();
        let mut quality = Vec::new();
        for size in corpus["corpus"]["physical_sizes"]
            .as_array()
            .ok_or("sizes")?
        {
            let width = size[0].as_u64().ok_or("width")? as u32;
            let height = size[1].as_u64().ok_or("height")? as u32;
            for scale in [100, 150, 200] {
                for variant in corpus["variants"]["variants"]
                    .as_array()
                    .ok_or("variants")?
                {
                    let id = format!(
                        "{width}x{height}-{scale}-{}",
                        variant["id"].as_str().ok_or("variant")?
                    );
                    let tokens = custom::tokens(&corpus["tokens"], variant)?;
                    let mut scene = check(Scene::new(source.clone(), tokens.clone()))?;
                    scene.selected = true;
                    let mut c = custom::controller(&corpus["corpus"])?;
                    let mut seq = 0;
                    let plan = reference_plan(
                        &source,
                        width as f32 * 100.0 / scale as f32,
                        height as f32 * 100.0 / scale as f32,
                    )?;
                    let view = Rect([
                        0.0,
                        0.0,
                        width as f32 * 100.0 / scale as f32,
                        height as f32 * 100.0 / scale as f32,
                    ]);
                    let work = check(scene.arrange(&source, &plan, view))?;
                    check(c.reconcile(&scene.tree))?;
                    let before = check(Snapshot::build(&scene.tree, c.provider.model()))?;
                    // Keyboard-only editor, search, list and settings; committed text is separate.
                    send(
                        &mut c,
                        &scene.tree,
                        &mut seq,
                        EventKind::Text("notes".into()),
                        "editor",
                        &mut engine,
                        &tokens,
                        scale,
                    )?;
                    send(
                        &mut c,
                        &scene.tree,
                        &mut seq,
                        EventKind::Key {
                            usage: 43,
                            down: true,
                        },
                        "editor",
                        &mut engine,
                        &tokens,
                        scale,
                    )?;
                    send(
                        &mut c,
                        &scene.tree,
                        &mut seq,
                        EventKind::Key {
                            usage: 43,
                            down: false,
                        },
                        "files-window",
                        &mut engine,
                        &tokens,
                        scale,
                    )?;
                    for _ in 0..20 {
                        send(
                            &mut c,
                            &scene.tree,
                            &mut seq,
                            EventKind::Key {
                                usage: 81,
                                down: true,
                            },
                            "files-window",
                            &mut engine,
                            &tokens,
                            scale,
                        )?;
                        send(
                            &mut c,
                            &scene.tree,
                            &mut seq,
                            EventKind::Key {
                                usage: 81,
                                down: false,
                            },
                            "files-window",
                            &mut engine,
                            &tokens,
                            scale,
                        )?;
                    }
                    tab(
                        &mut c,
                        &scene.tree,
                        &mut seq,
                        true,
                        2,
                        &mut engine,
                        &tokens,
                        scale,
                    )?;
                    send(
                        &mut c,
                        &scene.tree,
                        &mut seq,
                        EventKind::Text("notes".into()),
                        "search",
                        &mut engine,
                        &tokens,
                        scale,
                    )?;
                    tab(
                        &mut c,
                        &scene.tree,
                        &mut seq,
                        false,
                        5,
                        &mut engine,
                        &tokens,
                        scale,
                    )?;
                    send(
                        &mut c,
                        &scene.tree,
                        &mut seq,
                        EventKind::Key {
                            usage: 44,
                            down: true,
                        },
                        "reduced-motion",
                        &mut engine,
                        &tokens,
                        scale,
                    )?;
                    send(
                        &mut c,
                        &scene.tree,
                        &mut seq,
                        EventKind::Key {
                            usage: 44,
                            down: false,
                        },
                        "reduced-motion",
                        &mut engine,
                        &tokens,
                        scale,
                    )?;
                    let after = check(Snapshot::build(&scene.tree, c.provider.model()))?;
                    require(
                        c.provider.model().document().text() == "Draft: notes\n"
                            && c.provider.model().document().query() == "notes"
                            && c.provider.model().session().selected_file == 20
                            && c.provider.model().preferences().reduced_motion,
                        "Selected keyboard workflows",
                    )?;
                    let mut parity = custom::controller(&corpus["corpus"])?;
                    check(parity.semantic(
                        &scene.tree,
                        scene.tree.revision,
                        "sem.editor",
                        Command::SetText("notes".into()),
                    ))?;
                    check(parity.semantic(
                        &scene.tree,
                        scene.tree.revision,
                        "sem.files-window",
                        Command::Focus,
                    ))?;
                    for _ in 0..20 {
                        check(parity.semantic(
                            &scene.tree,
                            scene.tree.revision,
                            "sem.files-window",
                            Command::Navigate(1),
                        ))?;
                    }
                    for id in ["sem.editor", "sem.search"] {
                        check(parity.semantic(
                            &scene.tree,
                            scene.tree.revision,
                            id,
                            Command::Focus,
                        ))?;
                    }
                    check(parity.semantic(
                        &scene.tree,
                        scene.tree.revision,
                        "sem.search",
                        Command::SetText("notes".into()),
                    ))?;
                    for id in [
                        "sem.editor",
                        "sem.files-window",
                        "sem.launch-editor",
                        "sem.reduced-effects",
                        "sem.reduced-motion",
                    ] {
                        check(parity.semantic(
                            &scene.tree,
                            scene.tree.revision,
                            id,
                            Command::Focus,
                        ))?;
                    }
                    check(parity.semantic(
                        &scene.tree,
                        scene.tree.revision,
                        "sem.reduced-motion",
                        Command::Activate,
                    ))?;
                    require(
                        parity.provider.model().encode() == c.provider.model().encode(),
                        "Complete semantic/keyboard model checkpoint parity",
                    )?;
                    workflow.push(json!({"id":id,"state":custom::state(&c,&scene.tree),"before":snapshot_json(&before),"after":snapshot_json(&after),"notifications":format!("{:?}",after.diff(&before))}));
                    let mut surface = check(Surface::new(width, height, width * 4, 1))?;
                    let (image, _) = custom::capture_at(
                        custom::Candidate::Custom,
                        &mut scene,
                        &c,
                        &mut engine,
                        &mut surface,
                        scale,
                        &run.join(format!("{id}.ppm")),
                    )?;
                    matrix.push(json!({"id":id,"viewport":view.0,"work":{"measured":work.measured,"arranged":work.arranged,"depth":work.depth},"image":image,"semantics":snapshot_json(&after)}));
                }
            }
        }
        // The authored complex/long strings remain the corpus, now exercised with
        // visual navigation over real font-derived stops at every declared scale.
        let strings = corpus["corpus"]["text_cases"].as_array().ok_or("strings")?;
        for scale in [100, 150, 200] {
            for (index, string) in strings.iter().enumerate() {
                let text = string.as_str().ok_or("string text")?;
                let layout = engine
                    .layout(
                        text,
                        text_core::Config {
                            mono: true,
                            size: 14,
                            scale,
                            width: 300,
                            ligatures: true,
                        },
                    )
                    .map_err(|e| format!("shape: {e:?}"))?;
                let mut stops = Vec::new();
                for line in 0..layout.lines.len() {
                    let visual = layout
                        .visual_stops(line)
                        .map_err(|e| format!("stops {e:?}"))?;
                    for (index, stop) in visual.iter().enumerate() {
                        require(
                            text_core::is_boundary(text, stop.byte),
                            "Non-grapheme visual stop",
                        )?;
                        if index + 1 < visual.len() {
                            let at = text_core::VisualCaret {
                                line,
                                stop: index,
                                byte: stop.byte,
                                x: stop.x,
                            };
                            let next = layout
                                .visual_move(stop.byte, Some(at), 79)
                                .map_err(|e| format!("move {e:?}"))?;
                            require(
                                next.byte == visual[index + 1].byte
                                    && next.x == visual[index + 1].x,
                                "Visual right adjacency",
                            )?;
                            let back = layout
                                .visual_move(next.byte, Some(next), 80)
                                .map_err(|e| format!("back {e:?}"))?;
                            require(back == at, "Visual affinity roundtrip")?;
                        }
                        stops.push(json!({"line":line,"stop":index,"byte":stop.byte,"x":stop.x}));
                    }
                }
                // Exercise the actual selected controller, including Shift selection.
                let mut editor = check(Controller::new(
                    editor_model::Model::new(1, 1, text, "", editor_model::Selection::new(0, 0))
                        .map_err(|e| format!("model {e:?}"))?,
                ))?;
                let mut seq = 0;
                let tokens = custom::tokens(&corpus["tokens"], &corpus["variants"]["variants"][0])?;
                send(
                    &mut editor,
                    &source,
                    &mut seq,
                    EventKind::Key {
                        usage: 229,
                        down: true,
                    },
                    "editor",
                    &mut engine,
                    &tokens,
                    scale,
                )?;
                send(
                    &mut editor,
                    &source,
                    &mut seq,
                    EventKind::Key {
                        usage: 79,
                        down: true,
                    },
                    "editor",
                    &mut engine,
                    &tokens,
                    scale,
                )?;
                require(
                    editor.provider.model().document().selection().anchor == 0
                        && text_core::is_boundary(
                            text,
                            editor.provider.model().document().selection().caret as usize,
                        ),
                    "Shift visual selection",
                )?;
                send(
                    &mut editor,
                    &source,
                    &mut seq,
                    EventKind::Key {
                        usage: 79,
                        down: false,
                    },
                    "editor",
                    &mut engine,
                    &tokens,
                    scale,
                )?;
                send(
                    &mut editor,
                    &source,
                    &mut seq,
                    EventKind::Key {
                        usage: 229,
                        down: false,
                    },
                    "editor",
                    &mut engine,
                    &tokens,
                    scale,
                )?;
                texts.push(json!({"id":index,"text":text,"scale":scale,"stops":stops,"missing":layout.missing.iter().map(|r|[r.start,r.end]).collect::<Vec<_>>()}));
            }
        }
        // Standalone rounded coverage and coincident ancestor clips. The external
        // reviewer samples a much denser mathematical oracle, never these masks.
        for scale in [100, 150, 200] {
            let width = 64 * scale as u32 / 100;
            let height = width;
            let mut pixels = vec![0u8; width as usize * height as usize * 4];
            let mut canvas = check(paint::Canvas::new(
                &mut pixels,
                width,
                height,
                width * 4,
                scale,
            ))?;
            canvas.smooth_edges = true;
            canvas.fill(
                Rect([0.0, 0.0, 64.0, 64.0]),
                Rect([0.0, 0.0, 64.0, 64.0]),
                0.0,
                [0, 0, 0, 255],
                255,
            );
            let shape = Rect([8.25, 8.5, 40.0, 36.0]);
            canvas.fill(
                shape,
                Rect([0.0, 0.0, 64.0, 64.0]),
                8.0,
                [255, 255, 255, 255],
                255,
            );
            let name = format!("rounded-{scale}");
            let mut ppm = format!("P6\n{width} {height}\n255\n").into_bytes();
            for p in pixels.chunks_exact(4) {
                ppm.extend(&p[..3]);
            }
            fs::write(run.join(format!("{name}.ppm")), ppm).map_err(|e| e.to_string())?;
            quality
                .push(json!({"id":name,"scale":scale,"rect":shape.0,"radius":8.0,"kind":"round"}));
            let mut t = source.clone();
            t.nodes
                .retain(|n| ["desktop", "editor-window", "sidebar"].contains(&n.id.as_str()));
            t.nodes[0].children = vec!["editor-window".into()];
            t.nodes[0].rect = Rect([0.0, 0.0, 64.0, 64.0]);
            for n in &mut t.nodes[1..] {
                n.rect = shape;
                n.text.clear();
                n.clip = true;
                n.kind = custom_ui::scene::Kind::Box;
                n.opacity = 1.0;
            }
            t.nodes[1].children = vec!["sidebar".into()];
            t.nodes[2].children.clear();
            let mut tokens = custom::tokens(&corpus["tokens"], &corpus["variants"]["variants"][0])?;
            tokens.colors[0] = [0, 0, 0, 255];
            tokens.colors[1] = [255, 255, 255, 255];
            tokens.radius = 8.0;
            let mut scene = check(Scene::new(t, tokens))?;
            scene.selected = true;
            let c = custom::controller(&corpus["corpus"])?;
            let mut surface = check(Surface::new(width, height, width * 4, 1))?;
            let name = format!("ancestor-{scale}");
            custom::capture_at(
                custom::Candidate::Custom,
                &mut scene,
                &c,
                &mut engine,
                &mut surface,
                scale,
                &run.join(format!("{name}.ppm")),
            )?;
            quality.push(json!({"id":name,"scale":scale,"rect":shape.0,"radius":8.0,"kind":"coincident-ancestor"}));
        }
        // Actual publication counters: color and semantics do not cause layout.
        let tokens = custom::tokens(&corpus["tokens"], &corpus["variants"]["variants"][0])?;
        let mut scene = check(Scene::new(source.clone(), tokens))?;
        scene.dirty = Dirty::default();
        let mut next = scene.tree.clone();
        next.revision += 1;
        let mut colors = scene.tokens.clone();
        colors.colors[0] = [12, 16, 24, 255];
        check(scene.publish(next, colors))?;
        require(!scene.layout(), "Paint-only publication laid out")?;
        let paint_dirty = scene.dirty;
        scene.dirty = Dirty::default();
        let mut next = scene.tree.clone();
        next.revision += 1;
        next.nodes[0].name = "Desktop label".into();
        check(scene.publish(next, scene.tokens.clone()))?;
        require(
            !scene.layout() && scene.dirty.semantics,
            "Semantic-only publication laid out",
        )?;
        write_json(&run.join("matrix.json"), &json!(matrix))?;
        let mut deep = source.clone();
        deep.nodes.clear();
        deep.root = "desktop".into();
        for i in 0..12 {
            let mut n = source.nodes[0].clone();
            n.id = if i == 0 {
                "desktop".into()
            } else {
                format!("group{i}")
            };
            n.semantic_id = format!("sem.group{i}");
            n.rect = Rect([0.0, 0.0, 1280.0, 720.0]);
            n.clip = true;
            n.children = if i < 11 {
                vec![format!("group{}", i + 1)]
            } else {
                vec![]
            };
            deep.nodes.push(n);
        }
        let mut pressure = check(Scene::new(deep, scene.tokens.clone()))?;
        pressure.selected = true;
        let c = custom::controller(&corpus["corpus"])?;
        let before = c.provider.model().encode().map_err(|_| "model snapshot")?;
        let mut pixels = vec![0xA5; 1280 * 720 * 4];
        let mut canvas = check(paint::Canvas::new(&mut pixels, 1280, 720, 5120, 100))?;
        require(
            matches!(
                paint::render(
                    &mut pressure,
                    c.provider.model(),
                    None,
                    &mut engine,
                    &mut canvas
                ),
                Err(Error::Exhausted)
            ),
            "Selected scratch pressure must reject",
        )?;
        require(
            pixels.iter().all(|v| *v == 0xA5)
                && c.provider.model().encode().map_err(|_| "model snapshot")? == before,
            "Pressure changed unpublished pixels or document",
        )?;
        write_json(&run.join("workflows.json"), &json!(workflow))?;
        write_json(&run.join("visual-text.json"), &json!(texts))?;
        write_json(&run.join("quality.json"), &json!(quality))?;
        let selected_text = custom::selected_text(&corpus, &mut engine, &run)?;
        Ok(
            json!({"status":"passed","configurations":matrix.len(),"captures":matrix.len(),"visual_text_cases":texts.len(),"selected_text":selected_text,"semantic_keyboard_checkpoint_parity":true,"pressure":{"rejected_before_pixels":true,"document_preserved":true,"temporary_limit_bytes":64*1024*1024},"paint_only":{"layout":paint_dirty.layout,"paint":paint_dirty.paint,"semantics":paint_dirty.semantics},"native_assistive":false,"native_ime":false,"guest":false,"reload":false,"matrix_sha256":hash(&fs::read(run.join("matrix.json")).map_err(|e|e.to_string())?)}),
        )
    })();
    match outcome {
        Ok(report) => {
            write_json(&run.join("result.json"), &report)?;
            println!("Selected UI checks passed: {}", run.display());
            Ok(())
        }
        Err(reason) => {
            write_json(
                &run.join("result.json"),
                &json!({"status":"failed","reason":reason}),
            )?;
            Err(format!("{reason}; retained at {}", run.display()))
        }
    }
}
