use super::*;
use custom_ui::{
    input::{Event, EventKind},
    surface::Surface,
};
use editor_model::{Action, Entry, Selection};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn next(r: &Resources) -> Value {
    let mut v = r.payload();
    let id = r.id() + 1;
    v["base_generation"] = json!(r.id());
    v["generation"] = json!(id);
    v["build_id"] = json!(format!("reload-{id}"));
    v["tokens"]["generation"] = json!(id);
    v["layout"]["generation"] = json!(id);
    v
}
fn send(c: &mut Controller, t: &Tree, seq: &mut u64, kind: EventKind, target: &str) -> Result<()> {
    *seq += 1;
    checked(c.event(
        t,
        Event {
            sequence: *seq,
            time_us: *seq * 1000,
            revision: t.revision,
            target: target.into(),
            kind,
        },
    ))
    .map_err(|e| format!("Reload input event {seq} for {target}: {e}"))
}
fn invoke(c: &mut Controller, a: Action) -> Result<()> {
    checked(c.invoke(a, Entry::Automation))
}
fn remove(v: &mut Value, id: &str) {
    v["layout"]["nodes"]
        .as_array_mut()
        .unwrap()
        .retain(|n| n["id"] != id);
    for n in v["layout"]["nodes"].as_array_mut().unwrap() {
        n["children"].as_array_mut().unwrap().retain(|x| x != id);
    }
}
fn save(v: &Value, p: &Path) -> Result<Value> {
    let start = Instant::now();
    write_json(p, v)?;
    let bytes = fs::read(p).map_err(|e| e.to_string())?;
    Ok(
        json!({"build_id":v["build_id"],"build_ns":start.elapsed().as_nanos(),"source_sha256":hash(&bytes),"completed":"before receive, same-process QPC duration"}),
    )
}
pub fn execute() -> Result<()> {
    let run = root().join(format!(
        "build/m2/P2-11/reload-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        std::process::id()
    ));
    fs::create_dir_all(&run).map_err(|e| e.to_string())?;
    let mut r = Resources::new()?;
    let c = crate::ui_fixtures::checked_inputs()?;
    let mut owner = custom::controller(&c["corpus"])?;
    let owner_address = &owner.provider as *const _ as usize;
    let mut scene = checked(Scene::new(r.active.source.clone(), r.active.tokens.clone()))?;
    scene.selected = true;
    checked(scene.motion.timers(true, false, false, 1000, false))?;
    let fonts = crate::text::inputs()?;
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).map_err(|e| format!("font: {e:?}"))?;
    let file = run.join("generation.json");
    let mut builds = Vec::new();
    let mut seq = 0;
    invoke(&mut owner, Action::Insert(" unsaved".into()))?;
    invoke(&mut owner, Action::Select(Selection::new(2, 5)))?;
    invoke(&mut owner, Action::Search("a".into()))?;
    invoke(&mut owner, Action::FileScroll(537))?;
    invoke(&mut owner, Action::Scroll(3))?;
    invoke(&mut owner, Action::Window([90, 70, 1280, 720]))?;
    // Keep both undo and redo stacks nonempty across a publication.
    invoke(&mut owner, Action::Insert("history".into()))?;
    invoke(&mut owner, Action::Undo)?;
    let before = owner
        .provider
        .model()
        .encode()
        .map_err(|e| format!("checkpoint: {e:?}"))?;
    let before_state = custom::state(&owner, &scene.tree);
    let mut surface = checked(Surface::new(1280, 720, 5120, 1))?;
    let (before_pixels, _) = custom::capture_at(
        custom::Candidate::Custom,
        &mut scene,
        &owner,
        &mut engine,
        &mut surface,
        100,
        &run.join("before.ppm"),
    )?;
    let reader7 = r.reader();
    let mut v = next(&r);
    v["tokens"]["colors"]["panel"] = json!([45, 55, 76, 255]);
    v["variant"]["colors"] = v["tokens"]["colors"].clone();
    v["tokens"]["spacing"]["small"] = json!(8);
    let settings_shift = v["layout"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == "settings")
        .unwrap()["rect"][0]
        .as_f64()
        .unwrap()
        - 340.0;
    for n in v["layout"]["nodes"].as_array_mut().unwrap() {
        if n["id"] == "reduced-effects" || n["id"] == "reduced-motion" {
            n["rect"][0] = json!(n["rect"][0].as_f64().unwrap() - settings_shift);
        }
        if n["id"] == "settings" {
            n["rect"][0] = json!(340);
            n["semantic"]["name"] = json!("Reloaded settings");
        }
    }
    // A real topology change: status moves from editor-window to sidebar.
    for n in v["layout"]["nodes"].as_array_mut().unwrap() {
        if n["id"] == "editor-window" {
            n["children"]
                .as_array_mut()
                .unwrap()
                .retain(|c| c != "status");
        }
        if n["id"] == "sidebar" {
            n["children"].as_array_mut().unwrap().push(json!("status"));
        }
    }
    builds.push(save(&v, &file)?);
    let completed = run.join("completed.json");
    build(&file, &completed)?;
    r.apply(
        &completed,
        &mut scene,
        &mut owner,
        &mut engine,
        1280,
        720,
        100,
    )?;
    let (after_pixels, _) = custom::capture_at(
        custom::Candidate::Custom,
        &mut scene,
        &owner,
        &mut engine,
        &mut surface,
        100,
        &run.join("after.ppm"),
    )?;
    r.first_frame();
    need(
        scene.motion.next_deadline().is_some(),
        "Reload dropped caret deadline",
    )?;
    need(
        owner.provider.model().encode().unwrap() == before,
        "Reload changed complete model checkpoint",
    )?;
    need(
        owner_address == &owner.provider as *const _ as usize,
        "Authoritative provider moved/recreated",
    )?;
    need(
        before_pixels["rgba_sha256"] != after_pixels["rgba_sha256"],
        "Reload did not change pixels",
    )?;
    need(
        scene
            .tree
            .node("sidebar")
            .unwrap()
            .children
            .contains(&"status".into()),
        "Structure not published",
    )?;
    let semantic = crate::selected::snapshot_json(&checked(
        custom_ui::semantics::Snapshot::build(&scene.tree, owner.provider.model()),
    )?);
    let after_state = custom::state(&owner, &scene.tree);
    invoke(&mut owner, Action::Redo)?;
    invoke(&mut owner, Action::Undo)?;
    need(
        owner.provider.model().document().text() == before_state["document"].as_str().unwrap(),
        "History content changed",
    )?;
    // Keep one frame in use independently of resource publication/retirement.
    let mut held = checked(Surface::new(4, 4, 16, 1))?;
    let lease = checked(held.acquire(reader7.id()))?;
    checked(held.write(lease))?.fill(91);
    checked(held.commit(lease, &[[0, 0, 4, 4]]))?;
    let reader8 = r.reader();
    let stable_reader_hash = reader7.digest().to_string();
    builds.push(save(&next(&r), &file)?);
    r.apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)?;
    need(
        r.live() == 3
            && reader7.digest() == stable_reader_hash
            && checked(held.read(lease))?.iter().all(|b| *b == 91),
        "In-use generation/buffer changed",
    )?;
    let stable = owner.provider.model().encode().unwrap();
    let generation = r.id();
    builds.push(save(&next(&r), &file)?);
    let headroom = r
        .apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)
        .unwrap_err();
    need(
        headroom.contains("headroom")
            && r.id() == generation
            && owner.provider.model().encode().unwrap() == stable,
        "Headroom rejection changed active state",
    )?;
    checked(held.release(lease))?;
    drop(reader7);
    drop(reader8);
    r.apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)?;
    need(r.live() == 1, "Retired roots remain after release")?;
    let mut negative = vec![
        json!({"case":"withheld-readers-headroom","reason":headroom,"old_buffer_preserved":true,"reclaimed_on_release":true}),
    ];
    for case in [
        "truncated",
        "version",
        "unknown",
        "stale",
        "oversized",
        "duplicate-id",
        "cycle",
        "missing-font",
        "missing-image",
        "corrupt-asset",
        "font-version",
        "numeric-overflow",
        "nonfinite",
        "no-focus",
        "missing-shell",
        "bad-action",
        "render-failure",
    ] {
        let mut v = next(&r);
        match case {
            "version" => v["schema_version"] = json!(9),
            "unknown" => v["execute"] = json!("no"),
            "stale" => v["base_generation"] = json!(1),
            "duplicate-id" => {
                let n = v["layout"]["nodes"][0].clone();
                v["layout"]["nodes"].as_array_mut().unwrap().push(n);
            }
            "cycle" => v["layout"]["nodes"][0]["children"] = json!(["desktop"]),
            "missing-font" => {
                v["assets"].as_array_mut().unwrap().pop();
            }
            "missing-image" => v["layout"]["nodes"][3]["resource"] = json!("icon.missing"),
            "corrupt-asset" => v["assets"][0]["sha256"] = json!("bad"),
            "font-version" => v["tokens"]["fonts"]["editor"] = json!("missing"),
            "numeric-overflow" => v["tokens"]["font_sizes"]["editor"] = json!(65550),
            "nonfinite" => v["layout"]["nodes"][0]["rect"][0] = Value::Null,
            "no-focus" => {
                for n in v["layout"]["nodes"].as_array_mut().unwrap() {
                    n["focusable"] = json!(false);
                }
            }
            "missing-shell" => remove(&mut v, "editor-window"),
            "bad-action" => v["layout"]["nodes"][4]["action"] = json!("search.set@1"),
            // A valid deep clipped subtree exceeds selected paint scratch only
            // after schema/tree validation, exercising transactional stage failure.
            "render-failure" => {
                let mut child = v["layout"]["nodes"][2].clone();
                v["layout"]["nodes"][0]["children"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("deep-0"));
                for i in 0..10 {
                    child["id"] = json!(format!("deep-{i}"));
                    child["semantic"]["id"] = json!(format!("sem.deep-{i}"));
                    child["clip"] = json!(true);
                    child["rect"] = json!([0, 0, 640, 360]);
                    child["children"] = if i == 9 {
                        json!([])
                    } else {
                        json!([format!("deep-{}", i + 1)])
                    };
                    v["layout"]["nodes"]
                        .as_array_mut()
                        .unwrap()
                        .push(child.clone());
                }
            }
            _ => {}
        }
        builds.push(save(&v, &file)?);
        if case == "truncated" {
            fs::write(&file, b"{\"schema_version\":").map_err(|e| e.to_string())?;
        }
        if case == "oversized" {
            fs::write(&file, vec![b' '; MAX_FILE + 1]).map_err(|e| e.to_string())?;
        }
        let checkpoint = owner.provider.model().encode().unwrap();
        let digest = r.active.digest.clone();
        let rev = scene.tree.revision;
        let error = r
            .apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)
            .err()
            .ok_or(format!("Negative case accepted: {case}"))?;
        need(
            r.active.digest == digest
                && scene.tree.revision == rev
                && owner.provider.model().encode().unwrap() == checkpoint,
            "Rejected generation changed state",
        )?;
        negative.push(json!({"case":case,"reason":error,"checkpoint_preserved":true,"active_generation":r.id()}));
    }
    let completed_before = fs::read(&completed).map_err(|e| e.to_string())?;
    let failed_source = run.join("failed-source.json");
    fs::write(&failed_source, b"{").map_err(|e| e.to_string())?;
    let failed_build = build(&failed_source, &completed).unwrap_err();
    need(
        fs::read(&completed).map_err(|e| e.to_string())? == completed_before,
        "Failed builder replaced completed output",
    )?;
    negative.push(json!({"case":"failed-builder-preserves-output","reason":failed_build,"previous_output_sha256":hash(&completed_before)}));
    // A failed/missing builder output never activates. Query, then retry a build.
    let missing = r
        .apply(
            &run.join("failed-build.json"),
            &mut scene,
            &mut owner,
            &mut engine,
            1280,
            720,
            100,
        )
        .unwrap_err();
    negative.push(json!({"case":"failed-build","reason":missing,"active_generation":r.id()}));
    // Mid-typing: ordered queued committed text finishes against old geometry.
    seq += 1;
    checked(owner.enqueue(Event {
        sequence: seq,
        time_us: seq * 1000,
        revision: scene.tree.revision,
        target: "editor".into(),
        kind: EventKind::Text(" queued".into()),
    }))?;
    builds.push(save(&next(&r), &file)?);
    r.apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)?;
    need(
        owner.provider.model().document().text().contains(" queued"),
        "Accepted queued edit lost",
    )?;
    let queued = custom::state(&owner, &scene.tree);
    send(
        &mut owner,
        &scene.tree,
        &mut seq,
        EventKind::Pointer {
            x: 120.0,
            y: 90.0,
            button: 1,
            down: true,
        },
        "editor",
    )?;
    let mut v = next(&r);
    for n in v["layout"]["nodes"].as_array_mut().unwrap() {
        if n["id"] == "editor" {
            n["rect"][0] = json!(120);
        }
    }
    builds.push(save(&v, &file)?);
    r.apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)?;
    need(
        owner.capture.is_empty() && owner.buttons.is_empty() && owner.keys.is_empty(),
        "Moved capture stuck",
    )?;
    let drag = custom::state(&owner, &scene.tree);
    send(
        &mut owner,
        &scene.tree,
        &mut seq,
        EventKind::Composition {
            phase: "start".into(),
            text: "preedit".into(),
        },
        "editor",
    )?;
    let document = owner.provider.model().document().text().to_string();
    builds.push(save(&next(&r), &file)?);
    r.apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)?;
    need(
        owner.composition.is_some() && owner.provider.model().document().text() == document,
        "Compatible preedit lost/committed",
    )?;
    let composition = custom::state(&owner, &scene.tree);
    send(
        &mut owner,
        &scene.tree,
        &mut seq,
        EventKind::Composition {
            phase: "commit".into(),
            text: "committed".into(),
        },
        "editor",
    )?;
    send(
        &mut owner,
        &scene.tree,
        &mut seq,
        EventKind::Composition {
            phase: "start".into(),
            text: "cancel-me".into(),
        },
        "editor",
    )?;
    let mut v = next(&r);
    remove(&mut v, "editor");
    builds.push(save(&v, &file)?);
    r.apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)?;
    need(
        owner.composition.is_none() && owner.provider.model().session().focus == "search",
        "Removed focus did not cancel/fallback",
    )?;
    let removed = custom::state(&owner, &scene.tree);
    let doc = owner.provider.model().document().text().to_string();
    for i in 0..257 {
        let outcome = owner.enqueue(Event {
            sequence: seq + i + 1,
            time_us: (seq + i + 1) * 1000,
            revision: scene.tree.revision,
            target: "editor".into(),
            kind: EventKind::Key {
                usage: 225,
                down: true,
            },
        });
        if i == 256 {
            need(outcome.is_err(), "Overflow accepted")?;
        }
    }
    builds.push(save(&next(&r), &file)?);
    r.apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)?;
    need(
        owner.keys.is_empty()
            && owner.capture.is_empty()
            && owner.provider.model().document().text() == doc,
        "Overflow lost accepted edits/stuck keys",
    )?;
    let overflow = custom::state(&owner, &scene.tree);
    // Real file-trigger coalescing with deterministic monotonic injected clock.
    let mut trigger = Trigger::default();
    let mut v = next(&r);
    builds.push(save(&v, &file)?);
    need(!trigger.poll(&file, 0)?, "Debounce fired early")?;
    v["build_id"] = json!("rapid-save-final");
    builds.push(save(&v, &file)?);
    need(
        !trigger.poll(&file, 50_000)?
            && !trigger.poll(&file, 149_999)?
            && trigger.poll(&file, 150_000)?
            && !trigger.poll(&file, 250_000)?,
        "Debounce/coalescing",
    )?;
    r.apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)?;
    for _ in 0..12 {
        builds.push(save(&next(&r), &file)?);
        r.apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)?;
        need(r.live() == 1, "Old generations accumulate")?;
    }
    // The active answer remains authoritative even when a publish acknowledgement
    // is ignored by the client; repeating the old base rejects, never reapplies.
    let active = r.id();
    let retry = r
        .apply(&file, &mut scene, &mut owner, &mut engine, 1280, 720, 100)
        .unwrap_err();
    need(
        r.query()["active_generation"] == active && retry.contains("Stale"),
        "Lost ACK query/retry",
    )?;
    write_json(
        &run.join("result.json"),
        &json!({"status":"passed","pid":std::process::id(),"hosted":true,"guest":false,"native_behavior":false,"checkpoint_exact":true,"provider_address_stable":true,"before":before_state,"after":after_state,"before_pixels":before_pixels,"after_pixels":after_pixels,"semantics_after":semantic,"negative":negative,"queued":queued,"drag":drag,"composition":composition,"removed":removed,"overflow":overflow,"coalesced":trigger.coalesced,"query":r.query(),"builds":builds,"trace":r.trace,"clock":"Instant/QPC within one process; build_ns separate from receive/validate/stage/publish/first-frame"}),
    )?;
    println!("Resource reload checks passed: {}", run.display());
    Ok(())
}
