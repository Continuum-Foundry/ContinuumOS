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
