use super::*;

#[test]
fn staged_frame_reuse_requires_unchanged_render_state_and_submits_fully() {
    let corpus = crate::ui_fixtures::checked_inputs().unwrap();
    let fonts = crate::text::inputs().unwrap();
    let refs: Vec<_> = fonts.iter().map(Vec::as_slice).collect();
    let mut engine = text_core::Engine::new(&refs).unwrap();
    let run = root().join(format!(
        "build/m2/performance-remediation/staging-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&run).unwrap();
    for case in [
        "unchanged",
        "edit",
        "revision",
        "tokens",
        "scale",
        "preedit",
        "caret",
        "motion",
        "dimensions",
    ] {
        let mut r = Resources::new().unwrap();
        let mut scene = Scene::new(r.source().clone(), r.active.tokens.clone()).unwrap();
        scene.selected = true;
        let mut c = custom::controller(&corpus["corpus"]).unwrap();
        let file = run.join(format!("{case}.json"));
        write_json(&file, &tests_run::next(&r)).unwrap();
        let mut frame = r
            .apply(&file, &mut scene, &mut c, &mut engine, 1280, 720, 100)
            .unwrap();
        let staged_passes = scene.paint_passes;
        let mut scale = 100;
        let mut preedit = None;
        match case {
            "edit" => {
                c.invoke(
                    editor_model::Action::Insert(" edit".into()),
                    editor_model::Entry::Automation,
                )
                .unwrap();
            }
            "revision" => scene.tree.revision += 1,
            "tokens" => scene.tokens.colors[1] = [23, 65, 91, 255],
            "scale" => scale = 150,
            "preedit" => preedit = Some("preedit العربية"),
            "caret" => scene.caret_hidden = !scene.caret_hidden,
            "dimensions" => {
                frame.width = 0;
                assert!(
                    frame
                        .draw(&mut scene, c.provider.model(), preedit, &mut engine, scale)
                        .is_err()
                );
                frame.width = 720;
                frame.height = 1280;
            }
            "motion" => {
                scene
                    .motion
                    .transition("launcher", 0.9, 0.4, 0, false)
                    .unwrap();
                scene.tick(60_000).unwrap();
            }
            _ => {}
        }
        let (damage, counters) = frame
            .draw(&mut scene, c.provider.model(), preedit, &mut engine, scale)
            .unwrap();
        assert_eq!(
            damage.regions,
            [[0, 0, frame.width, frame.height]],
            "{case}"
        );
        assert_eq!(counters.reused_frame, case == "unchanged", "{case}");
        assert_eq!(
            scene.paint_passes,
            staged_passes + u64::from(case != "unchanged"),
            "{case}"
        );
        let mut oracle = vec![0; frame.pixels.len()];
        let mut canvas = custom_ui::paint::Canvas::new(
            &mut oracle,
            frame.width,
            frame.height,
            frame.width * 4,
            scale,
        )
        .unwrap();
        custom_ui::paint::render(
            &mut scene,
            c.provider.model(),
            preedit,
            &mut engine,
            &mut canvas,
        )
        .unwrap();
        assert_eq!(frame.pixels, oracle, "{case}");
    }
}

#[test]
fn desktop_opacity_is_validated_before_a_generation_can_publish() {
    let valid = fixture().unwrap();
    let pinned = valid["assets"].clone();
    assert!(Resources::parse(valid.clone(), &pinned).is_ok());
    for alpha in [0, 128, 254] {
        let mut invalid = valid.clone();
        invalid["tokens"]["colors"]["surface"][3] = json!(alpha);
        invalid["variant"]["colors"]["surface"][3] = json!(alpha);
        let result = Resources::parse(invalid, &pinned);
        assert!(
            matches!(result, Err(ref reason) if reason == "Hosted desktop surface must be opaque")
        );
    }
}
#[test]
fn generation_codec_rejects_unknown_and_wrapping_numeric_fields() {
    let r = Resources::new().unwrap();
    let mut v = tests_run::next(&r);
    v["tokens"]["font_sizes"]["body"] = json!(65550);
    assert!(
        Resources::parse(v, &r.pinned)
            .err()
            .unwrap()
            .contains("bound")
    );
    let mut v = tests_run::next(&r);
    v["layout"]["nodes"][0]["callback"] = json!("x");
    assert!(
        Resources::parse(v, &r.pinned)
            .err()
            .unwrap()
            .contains("field")
    );
}
#[test]
fn immutable_reader_and_weak_registry_are_bounded() {
    let mut r = Resources::new().unwrap();
    let old = r.reader();
    let next = Resources::parse(tests_run::next(&r), &r.pinned).unwrap();
    let previous = std::mem::replace(&mut r.active, Arc::new(next));
    r.retired.push(Arc::downgrade(&previous));
    drop(previous);
    assert_eq!(old.id(), 7);
    assert_eq!(r.id(), 8);
    assert_eq!(r.live(), 2);
    drop(old);
    assert_eq!(r.live(), 1);
}

#[test]
fn trigger_retries_identical_content_after_interrupted_write() {
    let dir = root().join(format!(
        "build/m2/P2-11/trigger-test-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("resource.json");
    fs::write(&file, b"{}").unwrap();
    let mut t = Trigger::default();
    assert!(!t.poll(&file, 0).unwrap());
    fs::remove_file(&file).unwrap();
    assert!(t.poll(&file, 50_000).is_err());
    fs::write(&file, b"{}").unwrap();
    assert!(!t.poll(&file, 60_000).unwrap());
    assert!(t.poll(&file, 160_000).unwrap());
    assert!(!t.poll(&file, 260_000).unwrap());
}
