//! Native-host orchestration. Portable code owns pixels, routing and semantics.
use crate::{Result, hash, root, write_json};
use custom_ui::{
    input::{Controller, Event, EventKind},
    scene::{Kind, Node, Scene, Tokens, Tree},
    surface::Surface,
    *,
};
use editor_model::{Action, Entry, Model, Selection};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

fn checked<T>(r: custom_ui::Result<T>) -> Result<T> {
    r.map_err(|e| format!("custom UI: {e:?}"))
}
fn require(ok: bool, message: impl Into<String>) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn load(path: &str) -> Result<Value> {
    serde_json::from_slice(&fs::read(root().join(path)).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn strv(v: &Value) -> Result<&str> {
    v.as_str().ok_or("Expected string".into())
}
fn num(v: &Value) -> Result<f32> {
    v.as_f64()
        .filter(|n| n.is_finite())
        .map(|n| n as f32)
        .ok_or("Expected finite number".into())
}
fn uint(v: &Value) -> Result<u64> {
    v.as_u64().ok_or("Expected unsigned integer".into())
}
fn boolean(v: &Value) -> Result<bool> {
    v.as_bool().ok_or("Expected bool".into())
}
fn array(v: &Value) -> Result<&Vec<Value>> {
    v.as_array().ok_or("Expected array".into())
}
fn color(v: &Value) -> Result<[u8; 4]> {
    let a = array(v)?;
    require(a.len() == 4, "Color length")?;
    let mut c = [0; 4];
    for (i, v) in a.iter().enumerate() {
        c[i] = u8::try_from(uint(v)?).map_err(|_| "Color bound")?
    }
    Ok(c)
}
fn rect(v: &Value) -> Result<Rect> {
    let a = array(v)?;
    require(a.len() == 4, "Rect length")?;
    let r = Rect([num(&a[0])?, num(&a[1])?, num(&a[2])?, num(&a[3])?]);
    checked(r.validate())?;
    Ok(r)
}
fn tree(layout: &Value) -> Result<Tree> {
    let mut nodes = Vec::new();
    for n in array(&layout["nodes"])? {
        let kind = match strv(&n["kind"])? {
            "row" => Kind::Row,
            "column" => Kind::Column,
            "overlay" => Kind::Overlay,
            "box" => Kind::Box,
            "scroll" => Kind::Scroll,
            "virtual-list" => Kind::List,
            "label" => Kind::Label,
            "button" => Kind::Button,
            "text-field" => Kind::Field,
            "editor" => Kind::Editor,
            "toggle" => Kind::Toggle,
            _ => return Err("Unknown node".into()),
        };
        nodes.push(Node {
            id: strv(&n["id"])?.into(),
            kind,
            children: array(&n["children"])?
                .iter()
                .map(|v| strv(v).map(str::to_owned))
                .collect::<Result<_>>()?,
            rect: rect(&n["rect"])?,
            clip: boolean(&n["clip"])?,
            opacity: num(&n["opacity"])?,
            text: strv(&n["text"])?.into(),
            action: strv(&n["action"])?.into(),
            resource: strv(&n["resource"])?.into(),
            semantic_id: strv(&n["semantic"]["id"])?.into(),
            role: strv(&n["semantic"]["role"])?.into(),
            name: strv(&n["semantic"]["name"])?.into(),
            focusable: boolean(&n["focusable"])?,
        });
    }
    let tree = Tree {
        nodes,
        root: strv(&layout["root"])?.into(),
        revision: uint(&layout["new_revision"])?,
        generation: uint(&layout["generation"])?,
    };
    checked(tree.validate())?;
    Ok(tree)
}
fn tokens(value: &Value, variant: &Value) -> Result<Tokens> {
    let mut colors = [[0; 4]; 7];
    for (i, name) in [
        "surface", "panel", "text", "muted", "accent", "focus", "danger",
    ]
    .iter()
    .enumerate()
    {
        colors[i] = color(&variant["colors"][name])?;
    }
    let t = Tokens {
        colors,
        spacing: num(&value["spacing"]["small"])?,
        radius: num(&value["radii"]["panel"])?,
        control_radius: num(&value["radii"]["control"])?,
        body_size: uint(&value["font_sizes"]["body"])? as u16,
        editor_size: uint(&value["font_sizes"]["editor"])? as u16,
        title_size: uint(&value["font_sizes"]["title"])? as u16,
        tint: color(&value["effects"]["tint"])?,
        fallback: color(&value["effects"]["fallback"])?,
        blur_radius: uint(&value["effects"]["radius"])? as u32,
        passes: uint(&value["effects"]["passes"])? as u32,
        reduced_effects: boolean(&variant["reduced_effects"])?,
        reduced_motion: boolean(&variant["reduced_motion"])?,
    };
    checked(t.validate())?;
    Ok(t)
}
fn controller(corpus: &Value) -> Result<Controller> {
    let i = &corpus["initial_state"];
    let selection = &i["selection"];
    checked(Controller::new(
        Model::new(
            1,
            uint(&i["revision"])?,
            strv(&i["document"])?,
            "",
            Selection::new(uint(&selection[0])? as u32, uint(&selection[1])? as u32),
        )
        .map_err(|e| format!("model: {e:?}"))?,
    ))
}
fn event(v: &Value) -> Result<Event> {
    let kind = match strv(&v["kind"])? {
        "key" => EventKind::Key {
            usage: uint(&v["usage"])? as u16,
            down: boolean(&v["pressed"])?,
        },
        "text" => EventKind::Text(strv(&v["text"])?.into()),
        "composition" => EventKind::Composition {
            phase: strv(&v["phase"])?.into(),
            text: strv(&v["text"])?.into(),
        },
        "pointer" => EventKind::Pointer {
            x: num(&v["x"])?,
            y: num(&v["y"])?,
            button: uint(&v["button"])? as u8,
            down: strv(&v["phase"])? == "down",
        },
        "scroll" => EventKind::Scroll(num(&v["y"])?),
        "focus" => EventKind::Focus,
        "capture" => EventKind::Capture,
        "reset" => EventKind::Reset,
        _ => return Err("Unknown input".into()),
    };
    Ok(Event {
        sequence: uint(&v["sequence"])?,
        time_us: uint(&v["time_us"])?,
        revision: uint(&v["scene_revision"])?,
        target: strv(&v["target"])?.into(),
        kind,
    })
}
fn state(c: &Controller, tree: &Tree) -> Value {
    let m = c.provider.model();
    let d = m.document();
    let s = m.session();
    let selection = d.selection();
    json!({"document":d.text(),"document_sha256":hash(d.text().as_bytes()),"revision":d.revision(),"document_revision":d.revision(),"model_revision":m.revision(),"checkpoint_sha256":hash(&m.encode().unwrap()),"selection":[selection.anchor,selection.caret],"dirty":d.dirty(),"focus":s.focus,"window":s.window,"search":d.query(),"match_count":d.matches().len(),"scroll_row":s.file_scroll,"selected_file":s.selected_file,"capture":c.capture,"pressed_keys":c.keys.len(),"pressed_buttons":c.buttons.len(),"composition_active":c.composition.is_some(),"cancelled":c.cancelled,"resets":c.resets,"dropped":c.dropped,"action":c.last_action,"semantic_logical_count":10000,"realized_max":scene::realize(s.file_scroll,10000,tree.node("files-window").unwrap().rect.0[3],24.0).unwrap().len(),"effects":if m.preferences().reduced_effects{"opaque-fallback"}else{"backdrop-box-blur"},"motion_ms":if m.preferences().reduced_motion{0}else{120}})
}
fn assert_state(c: &Controller, tree: &Tree, expected: &Value) -> Result<Value> {
    let mut actual = state(c, tree);
    actual["same_document"] = json!(c.provider.model().document().id() == 1);
    actual["reduced_effects"] = json!(c.provider.model().preferences().reduced_effects);
    let actions: Vec<_> = tree
        .preorder()
        .into_iter()
        .filter(|n| !n.action.is_empty())
        .map(|n| n.action.as_str())
        .collect();
    actual["same_semantic_actions"] = json!(
        actions
            == [
                "search.set@1",
                "edit.insert@1",
                "files.select@1",
                "launcher.open@1",
                "settings.effects@1",
                "settings.motion@1"
            ]
    );
    for (k, v) in expected.as_object().ok_or("Expected assertions")? {
        if k == "realized_max" {
            require(actual[k].as_u64() <= v.as_u64(), "Realization cap")?;
        } else {
            require(
                actual[k] == *v,
                format!("State {k}: expected {v}, actual {}", actual[k]),
            )?;
        }
    }
    Ok(actual)
}
fn semantic_json(nodes: &[Semantic]) -> Value {
    json!(nodes.iter().map(|n|json!({"id":n.id,"widget":n.widget,"role":n.role,"name":n.name,"value":n.value,"action":n.action,"bounds":n.bounds.0,"focus":n.focused,"checked":n.checked,"logical_count":n.logical_count,"realized":n.realized})).collect::<Vec<_>>())
}
fn assert_semantics(tree: &Tree, nodes: &[Semantic], layout: &Value) -> Result<()> {
    for n in array(&layout["nodes"])? {
        let id = strv(&n["semantic"]["id"])?;
        let actual = nodes
            .iter()
            .find(|s| s.id == id)
            .ok_or("Lost semantic ID")?;
        require(
            actual.widget == n["id"]
                && actual.name == n["semantic"]["name"]
                && actual.role == n["semantic"]["role"]
                && actual.action == n["action"]
                && actual.bounds == rect(&n["rect"])?,
            format!("Semantic mismatch: {id}"),
        )?;
    }
    let focus: Vec<_> = tree
        .preorder()
        .iter()
        .filter(|n| n.focusable)
        .map(|n| n.id.as_str())
        .collect();
    require(
        focus
            == [
                "search",
                "editor",
                "files-window",
                "launch-editor",
                "reduced-effects",
                "reduced-motion",
            ],
        "Keyboard reachability",
    )?;
    require(
        nodes
            .iter()
            .find(|n| n.widget == "files-window")
            .is_some_and(|n| n.logical_count == 10000 && n.realized.len() <= 64),
        "List semantic count",
    )
}
fn capture_at(
    scene: &mut Scene,
    c: &Controller,
    engine: &mut text_core::Engine<'_>,
    surface: &mut Surface,
    scale: u16,
    path: &Path,
) -> Result<(Value, Value)> {
    let layout_clock = Instant::now();
    scene.layout();
    let layout_ns = layout_clock.elapsed().as_nanos();
    let raster_clock = Instant::now();
    let lease = checked(surface.acquire(scene.tree.generation))?;
    let (width, height, stride) = (surface.width, surface.height, surface.stride);
    let mut canvas = checked(paint::Canvas::new(
        checked(surface.write(lease))?,
        width,
        height,
        stride,
        scale,
    ))?;
    let painted = paint::render(
        scene,
        c.provider.model(),
        c.composition.as_ref().map(|p| p.preedit()),
        engine,
        &mut canvas,
    );
    let counters = match painted {
        Ok(c) => c,
        Err(error) => {
            checked(surface.abandon(lease))?;
            return Err(format!("custom UI paint: {error:?}"));
        }
    };
    let raster_ns = raster_clock.elapsed().as_nanos();
    checked(surface.commit(lease, &[[0, 0, width, height]]))?;
    let submission_clock = Instant::now();
    let pixels = checked(surface.read(lease))?;
    let digest = hash(pixels);
    let mut image = format!("P6\n{width} {height}\n255\n").into_bytes();
    for row in pixels.chunks_exact(stride as usize) {
        for p in row[..width as usize * 4].chunks_exact(4) {
            require(p[3] == 255, "Final desktop must be opaque")?;
            image.extend(&p[..3]);
        }
    }
    let submission_ns = submission_clock.elapsed().as_nanos();
    fs::write(path, &image).map_err(|e| e.to_string())?;
    checked(surface.release(lease))?;
    let sem = scene.semantics(c.provider.model());
    let semantic = semantic_json(&sem);
    let geometry = json!({"rgba_sha256":digest,"image_sha256":hash(&image),"width":width,"height":height,"stride":stride,"scale":scale,"semantic_sha256":hash(serde_json::to_string(&semantic).unwrap().as_bytes()),"realized":counters.realized,"missing_clusters":counters.missing,"shape_work":counters.shape_work,"nodes":counters.nodes,"touched":counters.touched,"glyph_cache_entries":counters.glyph_cache_entries,"glyph_cache_bytes":counters.glyph_cache_bytes,"scratch_peak_bytes":counters.scratch_peak,"surface_bytes":surface.allocated_bytes(),"surface_allocations":surface.allocations,"full_damage_pixels":width as u64*height as u64});
    let timing = json!({"layout_ns":layout_ns,"raster_ns":raster_ns,"submission_hash_and_rgb_copy_ns":submission_ns,"file_write_excluded":true});
    Ok((geometry, timing))
}
pub fn execute() -> Result<()> {
    let corpus = crate::ui_fixtures::checked_inputs()?;
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("fonts: {e:?}"))?;
    let run = root().join(format!(
        "build/m2/P2-05/run-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id()
    ));
    fs::create_dir_all(&run).map_err(|e| e.to_string())?;
    let outcome: Result<Value> = (|| {
        let mut observations = Vec::new();
        let mut timings = Vec::new();
        let mut traces = Vec::new();
        let mut semantics = Vec::new();
        for size in array(&corpus["corpus"]["physical_sizes"])? {
            let width = uint(&size[0])? as u32;
            let height = uint(&size[1])? as u32;
            for scale in [100, 150, 200] {
                for variant in array(&corpus["variants"]["variants"])? {
                    let variant_id = strv(&variant["id"])?;
                    let id = format!("{width}x{height}-{scale}-{variant_id}");
                    let mut scene = checked(Scene::new(
                        tree(&corpus["layout"])?,
                        tokens(&corpus["tokens"], variant)?,
                    ))?;
                    let mut c = controller(&corpus["corpus"])?;
                    let mut surface = checked(Surface::new(width, height, width * 4, 1))?;
                    for step in array(&corpus["workflow"]["steps"])? {
                        for v in array(&step["events"])? {
                            checked(c.event(&scene.tree, event(v)?))?;
                        }
                        scene.dirty.layout |= c.dirty.layout;
                        scene.dirty.paint |= c.dirty.paint;
                        scene.dirty.semantics |= c.dirty.semantics;
                        c.dirty = scene::Dirty::default();
                        let actual = assert_state(&c, &scene.tree, &step["assertions"])?;
                        let nodes = scene.tree.semantics(c.provider.model());
                        assert_semantics(&scene.tree, &nodes, &corpus["layout"])?;
                        let step_id = strv(&step["id"])?;
                        traces.push(json!({"configuration":id,"step":step_id,"state":actual}));
                        if array(&corpus["variants"]["capture_steps"])?
                            .iter()
                            .any(|s| s == step_id)
                        {
                            let name = format!("{id}-{step_id}");
                            let (data, timing) = capture_at(
                                &mut scene,
                                &c,
                                &mut engine,
                                &mut surface,
                                scale,
                                &run.join(format!("{name}.ppm")),
                            )?;
                            observations.push(json!({"id":name,"data":data}));
                            timings.push(json!({"id":name,"timing":timing}));
                            semantics.push(json!({"id":name,"nodes":semantic_json(&nodes)}));
                        }
                    }
                    let _ = capture_at(
                        &mut scene,
                        &c,
                        &mut engine,
                        &mut surface,
                        scale,
                        &run.join(format!("{id}-settled.ppm")),
                    )?;
                    require(
                        surface.commits == 6 && surface.releases == 6,
                        "Buffer cleanup",
                    )?;
                    let before = (
                        scene.layout_passes,
                        scene.paint_passes,
                        scene.semantic_passes,
                    );
                    let start = Instant::now();
                    let mut idle = 0;
                    for _ in 0..1000 {
                        if scene.dirty.layout || scene.dirty.paint || scene.dirty.semantics {
                            idle += 1;
                        }
                    }
                    require(
                        idle == 0
                            && before
                                == (
                                    scene.layout_passes,
                                    scene.paint_passes,
                                    scene.semantic_passes,
                                ),
                        "Idle requests",
                    )?;
                    timings.push(json!({"id":id,"idle_polls":1000,"idle_render_requests":idle,"idle_loop_ns":start.elapsed().as_nanos(),"scope":"deterministic scheduler decision, no 30-second CPU measurement"}));
                }
            }
        }
        require(observations.len() == 150, "Capture matrix incomplete")?;
        let negative = negatives(&corpus, &mut engine, &run)?;
        let text = text_cases(&mut engine, &run)?;
        let text_nodes = text_nodes(&corpus, &mut engine, &run)?;
        write_json(&run.join("timings.json"), &json!(timings))?;
        write_json(&run.join("traces.json"), &json!(traces))?;
        write_json(&run.join("semantics.json"), &json!(semantics))?;
        let goldens_path = root().join("fixtures/m2/custom-v1/goldens.json");
        let raster_manifest=json!(observations.iter().map(|v|json!({"id":v["id"],"rgba_sha256":v["data"]["rgba_sha256"],"semantic_sha256":v["data"]["semantic_sha256"]})).collect::<Vec<_>>());
        if goldens_path.is_file() {
            let expected = load("fixtures/m2/custom-v1/goldens.json")?;
            require(
                expected["captures"] == raster_manifest,
                "Reviewed candidate goldens differ; preserve output and review changes",
            )?;
        }
        write_json(
            &run.join("raster-manifest.json"),
            &json!({"schema_version":1,"captures":raster_manifest}),
        )?;
        Ok(
            json!({"schema_version":1,"status":"passed","candidate":"custom-rust-v1","configuration_count":30,"capture_count":150,"workflow_steps":traces.len(),"observations":observations,"negative":negative,"text":text,"text_nodes":text_nodes,"clock":"std::time::Instant (Windows QPC); per-process durations; diagnostic build unless acceptance records otherwise","renderer":"encoded-sRGB premultiplied RGBA8, software","font_lock_sha256":hash(&fs::read(root().join("fixtures/m2/text-v1/fonts.lock.json")).unwrap()),"corpus_lock_sha256":hash(&fs::read(root().join("fixtures/m2/ui-v1/lock.json")).unwrap()),"profile_sha256":hash(&fs::read(root().join("config/m2-profiles.json")).unwrap()),"native_ime":false,"native_accessibility":false,"guest_tested":false,"reload_tested":false,"goldens_checked":goldens_path.is_file()}),
        )
    })();
    match outcome {
        Ok(report) => {
            write_json(&run.join("result.json"), &report)?;
            println!("Custom UI checks passed: {}", run.display());
            Ok(())
        }
        Err(e) => {
            write_json(
                &run.join("result.json"),
                &json!({"status":"failed","reason":e}),
            )?;
            Err(format!("{e}; retained at {}", run.display()))
        }
    }
}

fn negatives(
    corpus: &std::collections::BTreeMap<String, Value>,
    engine: &mut text_core::Engine<'_>,
    run: &Path,
) -> Result<Value> {
    let baseline = tree(&corpus["layout"])?;
    let tokens = tokens(&corpus["tokens"], &corpus["variants"]["variants"][0])?;
    let mut rows = Vec::new();
    for name in [
        "duplicate-id",
        "nonfinite-geometry",
        "missing-image",
        "cycle",
        "orphan",
        "unknown-action",
        "stale-revision",
        "invalid-token",
    ] {
        let mut scene = checked(Scene::new(baseline.clone(), tokens.clone()))?;
        let mut candidate = baseline.clone();
        candidate.revision += 1;
        let mut t = tokens.clone();
        match name {
            "duplicate-id" => candidate.nodes[1].id = candidate.nodes[0].id.clone(),
            "nonfinite-geometry" => candidate.nodes[1].rect.0[0] = f32::NAN,
            "missing-image" => candidate.nodes[1].resource = "unknown.image".into(),
            "cycle" => candidate.nodes[1].children.push(candidate.root.clone()),
            "orphan" => {
                candidate.nodes[0].children.pop();
            }
            "unknown-action" => candidate.nodes[1].action = "execute@9".into(),
            "stale-revision" => candidate.revision = baseline.revision,
            "invalid-token" => t.spacing = f32::INFINITY,
            _ => unreachable!(),
        }
        let before = scene.tree.nodes.clone();
        let outcome = scene.publish(candidate, t);
        require(
            outcome.is_err()
                && scene.tree.nodes == before
                && scene.tree.revision == baseline.revision,
            format!("Atomic reject {name}"),
        )?;
        rows.push(json!({"case":name,"outcome":format!("{:?}",outcome.unwrap_err()),"old_state_preserved":true}));
    }
    let mut c = controller(&corpus["corpus"])?;
    let before = c.provider.model().encode().unwrap();
    let stale = Event {
        sequence: 1,
        time_us: 1,
        revision: 3,
        target: "files-window".into(),
        kind: EventKind::Pointer {
            x: 452.0,
            y: 32.0,
            button: 1,
            down: true,
        },
    };
    require(
        c.event(&baseline, stale).is_err() && c.provider.model().encode().unwrap() == before,
        "Stale pointer",
    )?;
    rows.push(json!({"case":"stale-pointer","old_state_preserved":true}));
    let key = Event {
        sequence: 1,
        time_us: 1,
        revision: 4,
        target: "editor".into(),
        kind: EventKind::Key {
            usage: 225,
            down: true,
        },
    };
    checked(c.event(&baseline, key.clone()))?;
    c.capture = "editor".into();
    c.composition = Some(text_core::Composition::begin(&c.provider));
    for _ in 0..256 {
        checked(c.enqueue(key.clone()))?
    }
    require(
        c.enqueue(key).is_err()
            && c.pending.is_empty()
            && c.keys.is_empty()
            && c.capture.is_empty()
            && c.composition.is_none()
            && c.dropped == 257,
        "Overflow reset",
    )?;
    rows.push(json!({"case":"overflow-reset","dropped":c.dropped,"reason":c.reset_reason}));
    c.invoke(Action::Focus("search".into()), Entry::Semantic)
        .map_err(|e| format!("focus: {e:?}"))?;
    let mut removed = baseline.clone();
    removed.nodes.retain(|n| n.id != "search");
    for n in &mut removed.nodes {
        n.children.retain(|id| id != "search")
    }
    checked(removed.validate())?;
    checked(c.reconcile(&removed))?;
    require(
        c.provider.model().session().focus == "editor",
        "Removed focus fallback",
    )?;
    rows.push(json!({"case":"removed-focus","focus":"editor"}));
    checked(c.invoke(Action::Focus("search".into()), Entry::Semantic))?;
    c.capture = "search".into();
    c.keys.push(225);
    c.composition = Some(text_core::Composition::begin(&c.provider));
    let document = c.provider.model().document().text().to_owned();
    let revision = c.provider.model().document().revision();
    checked(c.reconcile(&removed))?;
    require(
        c.capture.is_empty()
            && c.keys.is_empty()
            && c.composition.is_none()
            && c.reset_reason == "capture-owner-removed"
            && c.provider.model().document().text() == document
            && c.provider.model().document().revision() == revision,
        "Removed capture owner cancellation",
    )?;
    c.reset("owner-death-injection", 0);
    require(
        c.capture.is_empty() && c.keys.is_empty() && c.composition.is_none(),
        "Owner-death reset",
    )?;
    rows.push(json!({"case":"capture-owner-removal-and-modeled-owner-death","reason":c.reset_reason,"document_preserved":true,"process_killed":false}));
    for count in [10000, 1_000_000, 1_000_000_000] {
        let realized = checked(scene::realize(count - 100, count, 328.0, 24.0))?;
        require(realized.len() == 15, "Virtual bounded count")?;
        rows.push(json!({"case":"virtual-count","logical_count":count,"realized":realized.len(),"row_id":realized[0]}));
    }
    let mut surface = checked(Surface::new(3, 2, 16, 3))?;
    let a = checked(surface.acquire(7))?;
    let b = checked(surface.acquire(7))?;
    let d = checked(surface.acquire(7))?;
    require(surface.acquire(7).is_err(), "Buffer exhaustion")?;
    checked(surface.commit(a, &[[0, 0, 3, 2]]))?;
    require(
        surface.write(a).is_err()
            && surface.acquire(7).is_err()
            && surface
                .release(surface::Lease {
                    serial: a.serial + 1,
                    ..a
                })
                .is_err(),
        "Withheld/wrong release",
    )?;
    checked(surface.release(a))?;
    let new = checked(surface.acquire(8))?;
    require(
        surface.release(a).is_err() && surface.release(new).is_err(),
        "Old lease",
    )?;
    for l in [new, b, d] {
        checked(surface.commit(l, &[[0, 0, 3, 2]]))?;
        checked(surface.release(l))?;
    }
    rows.push(json!({"case":"buffer-ownership-stride-exhaustion","serials":[a.serial,new.serial],"buffers":3,"bytes":surface.allocated_bytes()}));
    let mut candidate = baseline.clone();
    candidate
        .nodes
        .iter_mut()
        .find(|n| n.id == "status")
        .unwrap()
        .resource = "icon.missing".into();
    let mut scene = checked(Scene::new(candidate, tokens.clone()))?;
    let c = controller(&corpus["corpus"])?;
    let mut surface = checked(Surface::new(1280, 720, 5120, 1))?;
    capture_at(
        &mut scene,
        &c,
        engine,
        &mut surface,
        100,
        &run.join("missing-icon.ppm"),
    )?;
    rows.push(json!({"case":"missing-icon","fallback":"authored-2x2-nearest","semantic_name":"Unsaved • UTF-8"}));
    let mut scene = checked(Scene::new(baseline.clone(), tokens.clone()))?;
    let mut changed = baseline;
    changed.revision += 1;
    changed
        .nodes
        .iter_mut()
        .find(|n| n.id == "editor")
        .unwrap()
        .rect
        .0[0] += 8.0;
    let mut t = tokens;
    t.spacing = 8.0;
    let damage = checked(scene.publish(changed, t))?;
    require(
        damage.contains(104.0, 76.0) && damage.contains(419.0, 303.0),
        "Old/new damage",
    )?;
    let (new, _) = capture_at(
        &mut scene,
        &c,
        engine,
        &mut surface,
        100,
        &run.join("changed-layout.ppm"),
    )?;
    rows.push(json!({"case":"stable-id-layout-spacing-variant","damage":damage.0,"capture":new,"live_reload_claim":false}));
    let paint = load("fixtures/m2/ui-v1/paint.json")?;
    for v in array(&paint["premultiply"])? {
        require(
            paint::premultiply(color(&v["straight"])?) == color(&v["expected"])?,
            "Premultiply vector",
        )?;
    }
    for v in array(&paint["source_over"])? {
        require(
            paint::over(color(&v["source"])?, color(&v["destination"])?) == color(&v["expected"])?,
            "Blend vector",
        )?;
    }
    let mut pixels = vec![[32, 64, 96, 255]; 31 * 29];
    checked(paint::blur(&mut pixels, 31, 29, 8, 3))?;
    require(
        pixels.iter().all(|p| *p == [32, 64, 96, 255]),
        "Constant blur",
    )?;
    rows.push(
        json!({"case":"independent-paint-vectors","premultiply":3,"over":3,"constant_blur":true}),
    );
    require(text_core::Engine::new(&[]).is_err(), "Missing fonts")?;
    rows.push(json!({"case":"missing-font","staging":"rejected"}));
    let mut root_node = scene.tree.node("desktop").map_err(|_| "root")?.clone();
    root_node.children = vec!["panel".into()];
    root_node.rect = Rect([0.0, 0.0, 4.0, 4.0]);
    let mut panel = scene
        .tree
        .node("editor-window")
        .map_err(|_| "panel")?
        .clone();
    panel.id = "panel".into();
    panel.children.clear();
    panel.rect = root_node.rect;
    panel.opacity = 0.5;
    panel.text.clear();
    let group_tree = Tree {
        nodes: vec![root_node.clone(), panel.clone()],
        root: "desktop".into(),
        revision: 4,
        generation: 7,
    };
    let mut group_tokens = scene.tokens.clone();
    group_tokens.radius = 0.0;
    let mut group_scene = checked(Scene::new(group_tree, group_tokens.clone()))?;
    let mut bytes = vec![0u8; 4 * 4 * 4];
    let mut canvas = checked(paint::Canvas::new(&mut bytes, 4, 4, 16, 100))?;
    checked(paint::render(
        &mut group_scene,
        c.provider.model(),
        None,
        engine,
        &mut canvas,
    ))?;
    require(
        bytes.chunks_exact(4).all(|p| p == [29, 34, 46, 255]),
        "Group opacity applied once",
    )?;
    rows.push(json!({"case":"isolated-group-opacity","expected_pixel":[29,34,46,255],"pixels_checked":16}));
    panel.opacity = 1.0;
    panel.clip = true;
    panel.children = vec!["child".into()];
    let mut child = panel.clone();
    child.id = "child".into();
    child.semantic_id = "sem.child".into();
    child.clip = false;
    child.children.clear();
    child.rect = Rect([-2.0, -2.0, 10.0, 10.0]);
    group_tokens.radius = 2.0;
    let mut clipped = checked(Scene::new(
        Tree {
            nodes: vec![root_node, panel, child],
            root: "desktop".into(),
            revision: 4,
            generation: 7,
        },
        group_tokens,
    ))?;
    let mut bytes = vec![0u8; 64];
    let mut canvas = checked(paint::Canvas::new(&mut bytes, 4, 4, 16, 100))?;
    checked(paint::render(
        &mut clipped,
        c.provider.model(),
        None,
        engine,
        &mut canvas,
    ))?;
    require(
        bytes[..4] == [24, 28, 38, 255] && bytes[20..24] == [34, 40, 54, 255],
        "Rounded ancestor clip",
    )?;
    rows.push(json!({"case":"rounded-ancestor-clip","corner":"surface","center":"panel"}));
    let config = text_core::Config {
        mono: false,
        size: 16,
        scale: 100,
        width: 512,
        ligatures: true,
    };
    for glyph in 1..200 {
        engine
            .rasterize(0, glyph, config)
            .map_err(|e| format!("cache: {e:?}"))?;
    }
    let (entries, bytes) = engine.cache_usage();
    require(
        entries <= text_core::MAX_CACHE_ENTRIES && bytes <= text_core::MAX_CACHE_BYTES,
        "Candidate cache budget",
    )?;
    require(
        scene::realize(0, 10000, 2000.0, 24.0) == Err(Error::Exhausted),
        "Realization budget exhaustion",
    )?;
    rows.push(json!({"case":"cache-and-realization-exhaustion","glyph_requests":199,"entries":entries,"bitmap_bytes":bytes,"entry_cap":128,"payload_cap":1048576,"oversized_viewport":"rejected"}));
    let mut parity = None;
    for entry in [
        Entry::Button,
        Entry::Shortcut,
        Entry::Semantic,
        Entry::Automation,
    ] {
        let mut adapter = controller(&corpus["corpus"])?;
        checked(adapter.activate(&scene.tree, "reduced-effects", entry))?;
        let bytes = adapter.provider.model().encode().unwrap();
        if let Some(ref first) = parity {
            require(first == &bytes, "Semantic/button action parity")?;
        } else {
            parity = Some(bytes.clone())
        }
        rows.push(json!({"case":"action-entry-parity","entry":format!("{entry:?}"),"checkpoint_sha256":hash(&bytes),"action":adapter.last_action}));
    }
    let mut caret = controller(&corpus["corpus"])?;
    let layout = engine
        .layout(caret.provider.model().document().text(), config)
        .map_err(|e| format!("caret: {e:?}"))?;
    for stop in &layout.lines[0].carets {
        checked(caret.select_at(&layout, 0, stop.x))?;
        require(
            text_core::is_boundary(
                caret.provider.model().document().text(),
                caret.provider.model().document().selection().caret as usize,
            ),
            "Candidate caret must be grapheme stop",
        )?;
    }
    rows.push(json!({"case":"candidate-caret-hit","stops_checked":layout.lines[0].carets.len(),"alignment":"grapheme"}));
    Ok(json!(rows))
}
fn text_cases(engine: &mut text_core::Engine<'_>, run: &Path) -> Result<Value> {
    let corpus = load("fixtures/m2/text-v1/corpus.json")?;
    let mut rows = Vec::new();
    for case in array(&corpus["cases"])? {
        for scale in [100, 150, 200] {
            let id = strv(&case["id"])?;
            let text = strv(&case["text"])?;
            let physical_width = case["width"].as_u64().unwrap_or(512) as u16;
            let cfg = text_core::Config {
                mono: case["slot"] == "ui-mono",
                size: 16,
                scale,
                width: physical_width,
                ligatures: true,
            };
            let layout = engine
                .layout(text, cfg)
                .map_err(|e| format!("text {id}: {e:?}"))?;
            require(
                layout.missing.len() as u64 == uint(&case["missing"])?,
                format!("Candidate missing count {id}"),
            )?;
            if let Some(font) = case["font"].as_u64() {
                require(
                    layout
                        .lines
                        .iter()
                        .flat_map(|l| &l.glyphs)
                        .all(|g| g.font == font as usize),
                    format!("Candidate fallback {id}"),
                )?;
            }
            let width = physical_width as u32 + 16;
            let height = (layout.lines.len().max(1) as f32 * cfg.pixels() * 1.5 + 16.0) as u32;
            let mut surface = checked(Surface::new(width, height, width * 4, 1))?;
            let lease = checked(surface.acquire(7))?;
            let mut canvas = checked(paint::Canvas::new(
                checked(surface.write(lease))?,
                width,
                height,
                width * 4,
                scale,
            ))?;
            let factor = scale as f32 / 100.0;
            let clip = Rect([0.0, 0.0, width as f32 / factor, height as f32 / factor]);
            canvas.fill(clip, clip, 0.0, [0, 0, 0, 255], 255);
            checked(canvas.text(
                engine,
                text,
                Rect([
                    8.0 / factor,
                    8.0 / factor,
                    physical_width as f32 / factor,
                    (height - 16) as f32 / factor,
                ]),
                clip,
                16,
                cfg.mono,
                [255; 4],
                None,
                [0, 128, 255, 255],
            ))?;
            checked(surface.commit(lease, &[[0, 0, width, height]]))?;
            let bytes = checked(surface.read(lease))?;
            let mut image = format!("P6\n{width} {height}\n255\n").into_bytes();
            for p in bytes.chunks_exact(4) {
                image.extend(&p[..3]);
            }
            fs::write(run.join(format!("text-{id}-{scale}.ppm")), &image)
                .map_err(|e| e.to_string())?;
            rows.push(json!({"id":id,"scale":scale,"missing":layout.missing.len(),"lines":layout.lines.len(),"glyphs":layout.lines.iter().map(|l|l.glyphs.len()).sum::<usize>(),"raster_sha256":hash(bytes),"shaping":"shared pinned P2-04 pipeline; separate oracle replay required by acceptance","bidi":"shared logical/visual runs","composition":"workflow commit/cancel plus P2-04 corpus","fallback":"locked six fonts, no discovery"}));
            checked(surface.release(lease))?;
        }
    }
    require(rows.len() == 45, "Candidate text matrix")?;
    Ok(json!(rows))
}

fn text_nodes(
    corpus: &std::collections::BTreeMap<String, Value>,
    engine: &mut text_core::Engine<'_>,
    run: &Path,
) -> Result<Value> {
    let mut rows = Vec::new();
    for (index, text) in array(&corpus["corpus"]["text_cases"])?.iter().enumerate() {
        for scale in [100, 150, 200] {
            for target in ["editor", "search", "status"] {
                let mut tree = tree(&corpus["layout"])?;
                let mut c = controller(&corpus["corpus"])?;
                match target {
                    "editor" => {
                        let len = c.provider.model().document().text().len() as u32;
                        checked(
                            c.invoke(Action::Select(Selection::new(0, len)), Entry::Automation),
                        )?;
                        checked(c.invoke(Action::Insert(strv(text)?.into()), Entry::Automation))?;
                    }
                    "search" => {
                        checked(c.invoke(Action::Search(strv(text)?.into()), Entry::Automation))?
                    }
                    _ => {
                        tree.nodes.iter_mut().find(|n| n.id == target).unwrap().text =
                            strv(text)?.into()
                    }
                }
                let mut scene = checked(Scene::new(
                    tree,
                    tokens(&corpus["tokens"], &corpus["variants"]["variants"][0])?,
                ))?;
                let mut surface = checked(Surface::new(1280, 720, 5120, 1))?;
                let name = format!("text-node-{index}-{scale}-{target}");
                let (capture, _) = capture_at(
                    &mut scene,
                    &c,
                    engine,
                    &mut surface,
                    scale,
                    &run.join(format!("{name}.ppm")),
                )?;
                rows.push(json!({"id":name,"target":target,"scale":scale,"text_sha256":hash(strv(text)?.as_bytes()),"capture":capture,"unsupported":"emoji ZWJ/color rendering remains declared missing; overflowing text clips at the fixed control"}));
            }
        }
    }
    require(rows.len() == 63, "Scene text insertion matrix")?;
    Ok(json!(rows))
}

/// Host-only JSON-line adapter. Output files are fixed under this process's run,
/// never taken from event payloads. No guest or public network protocol.
pub fn host() -> Result<()> {
    use std::io::{BufRead, Write};
    let corpus = crate::ui_fixtures::checked_inputs()?;
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("fonts: {e:?}"))?;
    let run = root().join(format!(
        "build/m2/P2-05/host-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id()
    ));
    fs::create_dir_all(&run).map_err(|e| e.to_string())?;
    let mut scene = checked(Scene::new(
        tree(&corpus["layout"])?,
        tokens(&corpus["tokens"], &corpus["variants"]["variants"][0])?,
    ))?;
    let mut c = controller(&corpus["corpus"])?;
    let mut surface = checked(Surface::new(1280, 720, 5120, 1))?;
    let start = Instant::now();
    let mut sequence = 0;
    let mut frames = 0;
    let mut traces = Vec::new();
    println!(
        "{}",
        json!({"ready":true,"run":run,"pid":std::process::id(),"size":[1280,720],"scale":100})
    );
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    loop {
        if traces.len() >= 4096 {
            return Err("Native host command budget exhausted".into());
        }
        // Bounded read, including unterminated oversized lines.
        let mut raw = Vec::new();
        use std::io::Read;
        let n = input
            .by_ref()
            .take(65537)
            .read_until(b'\n', &mut raw)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if n > 65536 || raw.last() != Some(&b'\n') {
            return Err("Host input exceeds bounded line".into());
        }
        let command: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        if command["quit"] == true {
            break;
        }
        let result: Result<Value> = (|| {
            if let Some(step) = command["step"].as_u64() {
                let step = array(&corpus["workflow"]["steps"])?
                    .get(step as usize)
                    .ok_or("Step bound")?;
                for v in array(&step["events"])? {
                    let e = event(v)?;
                    sequence = e.sequence;
                    checked(c.event(&scene.tree, e))?;
                }
                assert_state(&c, &scene.tree, &step["assertions"])?;
            } else if command.get("kind").is_some() {
                let mut v = command.clone();
                sequence += 1;
                v["sequence"] = json!(sequence);
                v["time_us"] = json!(start.elapsed().as_micros().min(60_000_000));
                v["scene_revision"] = json!(scene.tree.revision);
                if v["target"].is_null() {
                    v["target"] = json!(c.provider.model().session().focus);
                }
                if v["kind"] == "pointer" {
                    let x = num(&v["x"])?;
                    let y = num(&v["y"])?;
                    let target = if c.capture.is_empty() {
                        scene
                            .tree
                            .hit(x, y)
                            .map(|n| n.id.clone())
                            .unwrap_or_else(|| "desktop".into())
                    } else {
                        c.capture.clone()
                    };
                    v["target"] = json!(target);
                }
                let e = event(&v)?;
                let caret = match e.kind {
                    EventKind::Pointer {
                        x, y, down: true, ..
                    } if e.target == "editor" => Some((x, y)),
                    _ => None,
                };
                checked(c.event(&scene.tree, e))?;
                if let Some((x, y)) = caret {
                    let n = checked(scene.tree.node("editor"))?;
                    let cfg = text_core::Config {
                        mono: true,
                        size: scene.tokens.editor_size,
                        scale: 100,
                        width: (n.rect.0[2] - 2.0 * scene.tokens.spacing) as u16,
                        ligatures: true,
                    };
                    let layout = engine
                        .layout(c.provider.model().document().text(), cfg)
                        .map_err(|e| format!("caret shape: {e:?}"))?;
                    let line = ((y - n.rect.0[1] - 2.0).max(0.0) / (cfg.pixels() * 1.5)) as usize;
                    let line = line.min(layout.lines.len().saturating_sub(1));
                    checked(c.select_at(&layout, line, x - n.rect.0[0] - scene.tokens.spacing))?;
                }
            }
            scene.dirty.layout |= c.dirty.layout;
            scene.dirty.paint |= c.dirty.paint;
            scene.dirty.semantics |= c.dirty.semantics;
            c.dirty = scene::Dirty::default();
            let (capture, timing) = capture_at(
                &mut scene,
                &c,
                &mut engine,
                &mut surface,
                100,
                &run.join("frame.ppm"),
            )?;
            frames += 1;
            let raw = fs::read(run.join("frame.ppm")).map_err(|e| e.to_string())?;
            let offset = raw
                .iter()
                .enumerate()
                .filter(|(_, b)| **b == b'\n')
                .nth(2)
                .map(|(i, _)| i + 1)
                .ok_or("PPM header")?;
            let bmp = bitmap(&raw[offset..], 1280, 720)?;
            fs::write(run.join("frame.bmp"), bmp).map_err(|e| e.to_string())?;
            let value = json!({"ok":true,"state":state(&c,&scene.tree),"capture":capture,"timing":timing,"frame":run.join("frame.bmp"),"frame_number":frames,"submission_copy":"RGBA to RGB capture then bottom-up BGR24 BMP; WinForms clones into owned bitmap; explicit Rust lease released after RGB copy","semantics":semantic_json(&scene.tree.semantics(c.provider.model()))});
            traces.push(json!({"command":command,"frame":frames,"state":value["state"],"rgba_sha256":value["capture"]["rgba_sha256"]}));
            Ok(value)
        })();
        let response = match result {
            Ok(v) => v,
            Err(e) => json!({"ok":false,"reason":e}),
        };
        println!("{response}");
        std::io::stdout().flush().map_err(|e| e.to_string())?;
    }
    write_json(
        &run.join("host-traces.json"),
        &json!({"pid":std::process::id(),"frames":frames,"trace":traces,"elapsed_ns":start.elapsed().as_nanos(),"native_ime":false,"guest":false}),
    )?;
    Ok(())
}
fn bitmap(rgb: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    require(
        rgb.len() == width as usize * height as usize * 3,
        "BMP input geometry",
    )?;
    let stride = (width * 3).div_ceil(4) * 4;
    let size = 54 + stride * height;
    let mut bytes = vec![0; size as usize];
    bytes[..2].copy_from_slice(b"BM");
    bytes[2..6].copy_from_slice(&size.to_le_bytes());
    bytes[10..14].copy_from_slice(&54u32.to_le_bytes());
    bytes[14..18].copy_from_slice(&40u32.to_le_bytes());
    bytes[18..22].copy_from_slice(&width.to_le_bytes());
    bytes[22..26].copy_from_slice(&height.to_le_bytes());
    bytes[26..28].copy_from_slice(&1u16.to_le_bytes());
    bytes[28..30].copy_from_slice(&24u16.to_le_bytes());
    for y in 0..height {
        for x in 0..width {
            let src = ((y * width + x) * 3) as usize;
            let dst = (54 + (height - 1 - y) * stride + x * 3) as usize;
            bytes[dst..dst + 3].copy_from_slice(&[rgb[src + 2], rgb[src + 1], rgb[src]]);
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires locked P2-04 fonts; explicitly run by Test-P2-05, outside Phase 0"]
    fn failed_paint_abandons_unpublished_buffer_and_keeps_document() {
        let corpus = crate::ui_fixtures::checked_inputs().unwrap();
        let fonts = crate::text::inputs().unwrap();
        let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
        let mut engine = text_core::Engine::new(&refs).unwrap();
        let mut scene = Scene::new(
            tree(&corpus["layout"]).unwrap(),
            tokens(&corpus["tokens"], &corpus["variants"]["variants"][0]).unwrap(),
        )
        .unwrap();
        let text = "\n".repeat(300);
        let c =
            Controller::new(Model::new(1, 11, &text, "", Selection::new(0, 0)).unwrap()).unwrap();
        let before = c.provider.model().encode().unwrap();
        let mut surface = Surface::new(1280, 720, 5120, 1).unwrap();
        let run = root().join(format!(
            "build/m2/P2-05/paint-error-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&run).unwrap();
        let path = run.join("unpublished.ppm");
        assert!(capture_at(&mut scene, &c, &mut engine, &mut surface, 100, &path).is_err());
        assert!(!path.exists());
        assert_eq!(surface.commits, 0);
        assert_eq!(before, c.provider.model().encode().unwrap());
        let next = surface.acquire(7).unwrap();
        assert_eq!(next.serial, 2);
        surface.abandon(next).unwrap();
    }
}
