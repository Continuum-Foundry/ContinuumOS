use super::*;

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
