use super::*;

#[test]
fn rejects_malformed_sfnt_before_parser() {
    for len in 0..40 {
        assert_eq!(preflight(&vec![0; len]), Err(Error::InvalidFont));
    }
    let mut data = vec![0; 28];
    data[..4].copy_from_slice(b"OTTO");
    data[5] = 1;
    data[20..24].copy_from_slice(&u32::MAX.to_be_bytes());
    data[24..28].copy_from_slice(&1u32.to_be_bytes());
    assert_eq!(preflight(&data), Err(Error::InvalidFont));
    assert_eq!(preflight(&vec![0; MAX_FONT_BYTES + 1]), Err(Error::Bounds));
}

#[test]
fn composition_stays_outside_document_until_authorized_commit() {
    use editor_model::*;
    let mut provider = Provider::new(
        Model::new(1, 1, "ab", "ab", Selection::new(1, 2)).unwrap(),
        7,
    )
    .unwrap();
    provider.grant(1, Rights::ALL).unwrap();
    let original = provider.model().encode().unwrap();
    let mut composition = Composition::begin(&provider);
    assert_eq!(
        composition.update("e\u{301}", 1),
        Err(super::Error::InvalidSelection)
    );
    assert_eq!(
        composition.update(&"x".repeat(1025), 0),
        Err(super::Error::Bounds)
    );
    composition.update("e\u{301}", 3).unwrap();
    assert_eq!(provider.model().encode().unwrap(), original);
    composition.clone().cancel();
    assert_eq!(provider.model().encode().unwrap(), original);
    let request = Request {
        version: 1,
        caller: 1,
        id: 1,
        epoch: 7,
        document: 1,
        expected_document_revision: 1,
        expected_model_revision: 1,
        deadline: 10,
        context: None,
        action: Action::Delete,
    };
    let mut denied = request.clone();
    denied.caller = 2;
    assert_eq!(
        composition
            .commit(&mut provider, denied, 1, false)
            .unwrap()
            .status,
        Err(editor_model::Error::Denied)
    );
    assert_eq!(provider.model().encode().unwrap(), original);
    assert_eq!(
        composition
            .commit(&mut provider, request, 1, false)
            .unwrap()
            .status,
        Ok(true)
    );
    assert_eq!(provider.model().document().text(), "ae\u{301}");
    assert_eq!(composition.preedit(), "e\u{301}");
}
#[test]
fn visual_navigation_preserves_bidi_affinity_and_wraps_lines() {
    use super::*;
    let layout = Layout {
        lines: vec![
            Line {
                bytes: 0..6,
                glyphs: vec![],
                carets: vec![
                    Caret { byte: 0, x: 0.0 },
                    Caret { byte: 1, x: 10.0 },
                    Caret { byte: 5, x: 20.0 },
                    Caret { byte: 3, x: 30.0 },
                    Caret { byte: 1, x: 40.0 },
                    Caret { byte: 6, x: 50.0 },
                ],
                width: 50.0,
            },
            Line {
                bytes: 6..7,
                glyphs: vec![],
                carets: vec![Caret { byte: 6, x: 0.0 }, Caret { byte: 7, x: 10.0 }],
                width: 10.0,
            },
        ],
        missing: vec![],
        shape_work: 0,
        graphemes: vec![0, 1, 3, 5, 6, 7],
    };
    let a = layout.visual_hit(0, 39.0).unwrap();
    assert_eq!((a.byte, a.x), (1, 40.0));
    let b = layout.visual_move(a.byte, Some(a), 80).unwrap();
    assert_eq!((b.byte, b.x), (3, 30.0));
    let c = layout.visual_move(b.byte, Some(b), 79).unwrap();
    assert_eq!(c, a);
    let end = layout.visual_move(c.byte, Some(c), 77).unwrap();
    assert_eq!((end.byte, end.x), (6, 50.0));
    let wrapped = layout.visual_move(end.byte, Some(end), 79).unwrap();
    assert_eq!((wrapped.line, wrapped.byte), (1, 6));
    let up = layout.visual_move(wrapped.byte, Some(wrapped), 82).unwrap();
    assert_eq!((up.line, up.x), (0, 0.0));
    assert!(layout.visual_hit(0, f32::NAN).is_err());
}
