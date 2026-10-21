use crate::{Result, hash, root, write_json};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use text_core::{Config, Engine, Error, Layout, boundaries, is_boundary};

fn require(ok: bool, message: impl Into<String>) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn load(path: &str) -> Result<Value> {
    serde_json::from_slice(&fs::read(root().join(path)).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
fn inputs() -> Result<Vec<Vec<u8>>> {
    let lock = load("fixtures/m2/text-v1/fonts.lock.json")?;
    require(
        lock["slots"]["ui-sans"] == json!([0, 1, 2, 3, 4, 5])
            && lock["slots"]["ui-mono"] == json!([5, 0, 1, 2, 3, 4])
            && lock["slots"]["complex-fallback"] == json!([0, 1, 2, 3, 4, 5]),
        "UI font slot binding mismatch",
    )?;
    let files = lock["fonts"].as_array().ok_or("Missing fonts")?;
    require(files.len() == 6, "Incorrect font lock count")?;
    files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let expected = format!("work/text-inputs/{}", text_core::FONT_NAMES[i]);
            require(file["path"] == expected, "Unexpected font path/order")?;
            let bytes = fs::read(root().join(&expected))
                .map_err(|e| format!("Pinned font missing: {expected}: {e}"))?;
            require(
                file["sha256"] == hash(&bytes) && file["bytes"] == bytes.len(),
                format!("Font identity mismatch: {expected}"),
            )?;
            Ok(bytes)
        })
        .collect()
}
fn official_breaks(path: &str, grapheme: bool) -> Result<usize> {
    use unicode_segmentation::UnicodeSegmentation;
    let data = fs::read_to_string(root().join(path)).map_err(|e| e.to_string())?;
    let mut count = 0;
    for (line_number, line) in data.lines().enumerate() {
        let content = line.split('#').next().unwrap().trim();
        if content.is_empty() {
            continue;
        }
        let mut text = String::new();
        let mut expected = Vec::new();
        for token in content.split_whitespace() {
            match token {
                "÷" => expected.push(text.len()),
                "×" => {}
                _ => text.push(
                    char::from_u32(u32::from_str_radix(token, 16).map_err(|e| e.to_string())?)
                        .ok_or("Invalid Unicode scalar fixture")?,
                ),
            }
        }
        let actual = if grapheme {
            boundaries(&text)
        } else {
            text.split_word_bound_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(text.len()))
                .collect()
        };
        require(
            actual == expected,
            format!(
                "Official breaks differ: {path}:{} expected {expected:?}, actual {actual:?}",
                line_number + 1
            ),
        )?;
        count += 1;
    }
    require(count > 500, "Official fixture unexpectedly small")?;
    Ok(count)
}
fn check_data() -> Result<Value> {
    let lock = load("fixtures/m2/text-v1/data.lock.json")?;
    let mut counts = Vec::new();
    for (i, file) in lock["files"]
        .as_array()
        .ok_or("Missing data lock")?
        .iter()
        .enumerate()
    {
        let path = file["path"].as_str().ok_or("Missing data path")?;
        require(
            path == if i == 0 {
                "work/text-inputs/GraphemeBreakTest-17.0.0.txt"
            } else {
                "work/text-inputs/WordBreakTest-17.0.0.txt"
            },
            "Unexpected data path",
        )?;
        let bytes = fs::read(root().join(path)).map_err(|e| e.to_string())?;
        require(
            file["sha256"] == hash(&bytes) && file["bytes"] == bytes.len(),
            "Official data identity mismatch",
        )?;
        counts.push(json!({"path":path,"cases":official_breaks(path,i==0)?,"sha256":hash(&bytes)}));
    }
    require(counts.len() == 2, "Incomplete official data lock")?;
    Ok(json!(counts))
}
fn glyphs(layout: &Layout) -> Value {
    json!(layout.lines.iter().flat_map(|l|l.glyphs.iter()).map(|g|json!({"font":g.font,"id":g.id,"cluster":[g.cluster.start,g.cluster.end],"x":g.x,"advance":g.advance,"offset":g.offset,"rtl":g.rtl})).collect::<Vec<_>>())
}
fn check_oracle(case: &Value, layout: &Layout, cfg: Config, oracle: &Value) -> Result<()> {
    let Some(reference) = oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == case["id"])
    else {
        return Ok(());
    };
    let actual = layout
        .lines
        .iter()
        .flat_map(|l| &l.glyphs)
        .collect::<Vec<_>>();
    let expected = reference["glyphs"].as_array().unwrap();
    require(
        actual.len() == expected.len(),
        format!("Shaping count mismatch: {}", case["id"]),
    )?;
    // These five locked Noto TTFs declare 1000 units/em. Independently checked by native oracle.
    let factor = cfg.pixels() / 1000.0;
    for (a, e) in actual.iter().zip(expected) {
        require(
            json!(a.id) == e["id"]
                && json!(a.font) == reference["font"]
                && json!(a.cluster.start) == e["cluster"],
            format!(
                "Shaping glyph/cluster mismatch: {}: {a:?} vs {e}",
                case["id"]
            ),
        )?;
        require(
            (a.advance - e["advance"].as_f64().unwrap() as f32 * factor).abs() < 0.002
                && a.offset
                    .iter()
                    .zip(e["offset"].as_array().unwrap())
                    .all(|(a, e)| (*a - e.as_f64().unwrap() as f32 * factor).abs() < 0.002),
            format!("Shaping positions differ: {}", case["id"]),
        )?;
    }
    Ok(())
}
fn render(engine: &mut Engine<'_>, layout: &Layout, cfg: Config, path: &Path) -> Result<String> {
    let width = cfg.width as usize + 32;
    let line_height = (cfg.pixels() * 2.0) as usize;
    let height = (layout.lines.len() * line_height + 32).max(64);
    require(width * height <= 16 * 1024 * 1024, "Preview image bound")?;
    let mut pixels = vec![0u8; width * height];
    for (row, line) in layout.lines.iter().enumerate() {
        let baseline = (row * line_height) as i32 + 16 + cfg.pixels() as i32;
        for g in &line.glyphs {
            let bitmap = engine
                .rasterize(g.font, g.id, cfg)
                .map_err(|e| format!("Raster failure: {e:?}"))?;
            let left = 16 + (g.x + g.offset[0]).round() as i32 + bitmap.xmin;
            let top = baseline - bitmap.height as i32 - bitmap.ymin - g.offset[1].round() as i32;
            for y in 0..bitmap.height {
                for x in 0..bitmap.width {
                    let dx = left + x as i32;
                    let dy = top + y as i32;
                    if dx >= 0 && dy >= 0 && dx < width as i32 && dy < height as i32 {
                        let dest = &mut pixels[dy as usize * width + dx as usize];
                        let src = bitmap.pixels[y * bitmap.width + x];
                        *dest = 255 - ((255 - *dest) as u16 * (255 - src) as u16 / 255) as u8;
                    }
                }
            }
        }
    }
    let mut image = format!("P5\n{width} {height}\n255\n").into_bytes();
    image.extend(&pixels);
    fs::write(path, &image).map_err(|e| e.to_string())?;
    Ok(hash(&image))
}
fn negatives(engine: &mut Engine<'_>, fonts: &[Vec<u8>]) -> Result<Value> {
    let cfg = Config {
        mono: false,
        size: 16,
        scale: 100,
        width: 512,
        ligatures: true,
    };
    let mut rows = Vec::new();
    for (name, text, config, error) in [
        (
            "text-limit",
            "x".repeat(text_core::MAX_TEXT + 1),
            cfg,
            Error::Bounds,
        ),
        (
            "line-limit",
            "\n".repeat(text_core::MAX_LINES),
            cfg,
            Error::Bounds,
        ),
        (
            "size-zero",
            "a".into(),
            Config { size: 0, ..cfg },
            Error::Bounds,
        ),
        (
            "size-large",
            "a".into(),
            Config { size: 49, ..cfg },
            Error::Bounds,
        ),
        (
            "scale-invalid",
            "a".into(),
            Config { scale: 125, ..cfg },
            Error::Bounds,
        ),
        (
            "width-zero",
            "a".into(),
            Config { width: 0, ..cfg },
            Error::Bounds,
        ),
        (
            "width-large",
            "a".into(),
            Config { width: 4097, ..cfg },
            Error::Bounds,
        ),
        ("control-nul", "\0".into(), cfg, Error::Unsupported),
        (
            "bidi-override",
            "a\u{202e}b".into(),
            cfg,
            Error::Unsupported,
        ),
        (
            "shape-budget",
            "x".repeat(4096),
            Config { width: 1, ..cfg },
            Error::Bounds,
        ),
    ] {
        require(
            engine.layout(&text, config) == Err(error),
            format!("Negative did not reject: {name}"),
        )?;
        rows.push(json!({"case":name,"result":format!("{error:?}")}));
    }
    require(
        engine
            .layout(&"x".repeat(4096), Config { width: 4096, ..cfg })
            .is_ok(),
        "Maximum input must be usable",
    )?;
    let mut corrupt = fonts.to_vec();
    let last = corrupt[0].len() - 1;
    corrupt[0][last] ^= 1;
    require(
        matches!(
            Engine::new(&corrupt.iter().map(Vec::as_slice).collect::<Vec<_>>()),
            Err(Error::UnpinnedFont)
        ),
        "Unpinned font must reject before deep parse",
    )?;
    require(
        matches!(
            Engine::new(&fonts[..4].iter().map(Vec::as_slice).collect::<Vec<_>>()),
            Err(Error::UnpinnedFont)
        ),
        "Missing fallback font must reject",
    )?;
    rows.push(json!({"case":"font-corrupted","result":"UnpinnedFont"}));
    rows.push(json!({"case":"font-missing","result":"UnpinnedFont"}));
    for (font, glyph) in [(6, 0), (0, u16::MAX)] {
        require(
            engine.rasterize(font, glyph, cfg) == Err(Error::Bounds),
            "Raster index bounds",
        )?;
    }
    rows.push(json!({"case":"raster-indices","result":"Bounds"}));
    for glyph in 0..180 {
        engine
            .rasterize(0, glyph, cfg)
            .map_err(|e| format!("Cache exercise {e:?}"))?;
    }
    let usage = engine.cache_usage();
    require(
        usage.0 == text_core::MAX_CACHE_ENTRIES && usage.1 <= text_core::MAX_CACHE_BYTES,
        "Bitmap LRU exceeds bound",
    )?;
    let before = engine
        .rasterize(0, 36, cfg)
        .map_err(|e| format!("{e:?}"))?
        .clone();
    let scaled = engine
        .rasterize(0, 36, Config { scale: 200, ..cfg })
        .map_err(|e| format!("{e:?}"))?
        .clone();
    require(before != scaled, "Scale is absent from raster cache key")?;
    require(
        engine.rasterize(0, 36, cfg).map_err(|e| format!("{e:?}"))? == &before,
        "Evicted bitmap failed reproducible regeneration",
    )?;
    rows.push(json!({"case":"cache-lru-scale","entries":usage.0,"bitmap_bytes":usage.1,"result":"passed"}));
    Ok(json!(rows))
}
fn compositions(engine: &Engine<'_>) -> Result<Value> {
    use editor_model::{Action, Model, Provider, Request, Rights, Selection};
    let fixture = load("fixtures/m2/text-v1/composition.json")?;
    let mut rows = Vec::new();
    for case in fixture["cases"]
        .as_array()
        .ok_or("Missing composition cases")?
    {
        let mut provider = Provider::new(
            Model::new(
                1,
                1,
                fixture["initial"].as_str().unwrap(),
                "",
                Selection::new(7, 14),
            )
            .map_err(|e| format!("{e:?}"))?,
            7,
        )
        .map_err(|e| format!("{e:?}"))?;
        provider
            .grant(1, Rights::ALL)
            .map_err(|e| format!("{e:?}"))?;
        let original = provider.model().encode().map_err(|e| format!("{e:?}"))?;
        let mut preedit = text_core::Composition::begin(&provider);
        preedit
            .update(
                fixture["preedit"].as_str().unwrap(),
                fixture["caret"].as_u64().unwrap() as usize,
            )
            .map_err(|e| format!("{e:?}"))?;
        let before = preedit.clone();
        for invalid in fixture["invalid_updates"].as_array().unwrap() {
            let error = preedit
                .update(
                    invalid["text"].as_str().unwrap(),
                    invalid["caret"].as_u64().unwrap() as usize,
                )
                .err()
                .ok_or("Invalid preedit accepted")?;
            require(
                format!("{error:?}") == invalid["status"].as_str().unwrap() && preedit == before,
                "Rejected preedit must be atomic",
            )?;
        }
        // Discard two text layouts/views at different scales while retaining the same preedit/provider.
        for scale in [100, 200] {
            let _view = engine
                .layout(
                    preedit.preedit(),
                    Config {
                        mono: false,
                        size: 16,
                        scale,
                        width: 512,
                        ligatures: true,
                    },
                )
                .map_err(|e| format!("{e:?}"))?;
        }
        require(
            preedit == before
                && provider.model().encode().map_err(|e| format!("{e:?}"))? == original,
            "Relayout changed composition or document",
        )?;
        let make_request = |p: &Provider, id, action| Request {
            version: 1,
            caller: 1,
            id,
            epoch: 7,
            document: 1,
            expected_document_revision: p.model().document().revision(),
            expected_model_revision: p.model().revision(),
            deadline: 2,
            context: None,
            action,
        };
        let mode = case["mode"].as_str().unwrap();
        let status = if mode == "cancel" {
            preedit.cancel();
            "cancelled preedit".into()
        } else {
            if mode == "stale" {
                let request = make_request(&provider, 1, Action::Insert("x".into()));
                require(
                    provider.execute(request, 1, false).status == Ok(true),
                    "Stale setup failed",
                )?;
            }
            let mut request = make_request(&provider, 2, Action::Delete);
            if mode == "denied" {
                request.caller = 2;
            }
            match preedit.commit(
                &mut provider,
                request,
                if mode == "deadline" { 3 } else { 1 },
                mode == "cancelled",
            ) {
                Ok(outcome) => format!("{:?}", outcome.status),
                Err(error) => format!("{error:?}"),
            }
        };
        let doc = provider.model().document();
        require(
            status == case["status"].as_str().unwrap()
                && json!(doc.text()) == case["text"]
                && json!([doc.selection().anchor, doc.selection().caret]) == case["selection"]
                && json!(doc.undo_len()) == case["undo"],
            format!("Composition mismatch: {}: {status}", case["id"]),
        )?;
        rows.push(json!({"case":case["id"],"status":status,"text":doc.text(),"selection":[doc.selection().anchor,doc.selection().caret],"undo":doc.undo_len(),"checkpoint_sha256":hash(&provider.model().encode().map_err(|e|format!("{e:?}"))?)}));
    }
    Ok(json!(rows))
}
pub fn execute() -> Result<()> {
    let lock = load("fixtures/m2/text-v1/lock.json")?;
    let allowed = [
        "corpus.json",
        "composition.json",
        "fonts.lock.json",
        "data.lock.json",
        "shaping-reference.json",
    ];
    let files = lock["files"]
        .as_array()
        .ok_or("Text fixture lock missing")?;
    require(
        lock["schema_version"] == 1 && files.len() == allowed.len(),
        "Text fixture lock version/count",
    )?;
    for name in allowed {
        let matches = files
            .iter()
            .filter(|file| file["path"] == name)
            .collect::<Vec<_>>();
        require(
            matches.len() == 1,
            "Duplicate/missing text fixture lock member",
        )?;
        require(
            matches[0]["sha256"]
                == hash(
                    &fs::read(root().join("fixtures/m2/text-v1").join(name))
                        .map_err(|e| e.to_string())?,
                ),
            format!("Text fixture changed: {name}"),
        )?;
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let directory = root().join(format!("build/m2/P2-04/run-{stamp}"));
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let corpus = load("fixtures/m2/text-v1/corpus.json")?;
    let oracle = load("fixtures/m2/text-v1/shaping-reference.json")?;
    require(
        oracle["corpus_sha256"]
            == hash(
                &fs::read(root().join("fixtures/m2/text-v1/corpus.json"))
                    .map_err(|e| e.to_string())?,
            ),
        "Oracle corpus identity mismatch",
    )?;
    let fonts = inputs()?;
    let references = fonts.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let mut engine =
        Engine::new(&references).map_err(|e| format!("Pinned engine rejected: {e:?}"))?;
    let data = check_data()?;
    let mut results = Vec::new();
    for case in corpus["cases"].as_array().ok_or("Missing text cases")? {
        let id = case["id"].as_str().ok_or("Case id")?;
        let text = case["text"].as_str().ok_or("Case text")?;
        if let Some(expected) = case["graphemes"].as_array() {
            let mut at = 0;
            let mut stops = Vec::new();
            for g in expected {
                stops.push(at);
                at += g.as_str().unwrap().len();
            }
            stops.push(at);
            require(
                at == text.len() && boundaries(text) == stops,
                format!("Authored graphemes differ: {id}"),
            )?;
        }
        // Every byte offset goes through the existing document validator, including UTF-8 interiors.
        for i in 0..=text.len() {
            require(
                editor_model::Model::new(
                    1,
                    1,
                    text,
                    "",
                    editor_model::Selection::new(i as u32, i as u32),
                )
                .is_ok()
                    == is_boundary(text, i),
                format!("Document/cluster boundary mismatch {id}:{i}"),
            )?;
        }
        for scale in [100, 150, 200] {
            let cfg = Config {
                mono: case["slot"] == "ui-mono",
                size: 16,
                scale,
                width: case["width"].as_u64().unwrap_or(512) as u16,
                ligatures: true,
            };
            let layout = engine
                .layout(text, cfg)
                .map_err(|e| format!("Layout {id}/{scale}: {e:?}"))?;
            require(
                layout.missing.len() as u64 == case["missing"].as_u64().unwrap(),
                format!("Fallback mismatch {id}: {:?}", layout.missing),
            )?;
            if let Some(font) = case["font"].as_u64() {
                require(
                    layout
                        .lines
                        .iter()
                        .flat_map(|l| &l.glyphs)
                        .all(|g| g.font == font as usize),
                    format!("Font choice mismatch: {id}"),
                )?;
            }
            if scale == 100 {
                if let Some(ends) = case["line_ends"].as_array() {
                    require(
                        json!(layout.lines.iter().map(|l| l.bytes.end).collect::<Vec<_>>())
                            == json!(ends),
                        format!(
                            "Line wraps differ {id}: {:?}",
                            layout.lines.iter().map(|l| &l.bytes).collect::<Vec<_>>()
                        ),
                    )?;
                }
            }
            check_oracle(case, &layout, cfg, &oracle)?;
            for line in &layout.lines {
                let selected = line
                    .selection(line.bytes.clone())
                    .map_err(|e| format!("Selection {id}: {e:?}"))?;
                let selected_width = selected.iter().map(|r| r[1] - r[0]).sum::<f32>();
                require(
                    (selected_width - line.width).abs() < 0.01,
                    format!(
                        "Full selection width mismatch {id}: {selected_width} vs {}",
                        line.width
                    ),
                )?;
                require(
                    line.selection(line.bytes.start..line.bytes.start) == Ok(Vec::new()),
                    "Collapsed selection must be empty",
                )?;
                for caret in &line.carets {
                    require(
                        is_boundary(text, caret.byte),
                        format!("Caret splits grapheme {id}:{caret:?}"),
                    )?;
                    let hit = line.hit(caret.x).map_err(|e| format!("{e:?}"))?;
                    require(
                        is_boundary(text, hit)
                            && line
                                .carets
                                .iter()
                                .any(|c| c.byte == hit && (c.x - caret.x).abs() < 0.002),
                        format!("Hit not at equivalent visual boundary {id}:{caret:?}->{hit}"),
                    )?;
                }
                require(
                    line.hit(f32::NAN) == Err(Error::Bounds),
                    "NaN hit must reject",
                )?;
            }
            if id == "mixed-bidi" {
                let clusters = layout.lines[0]
                    .glyphs
                    .iter()
                    .map(|g| g.cluster.start)
                    .collect::<Vec<_>>();
                require(
                    clusters == [0, 1, 16, 12, 10, 9, 7, 8, 6, 4, 2],
                    format!("Authored visual bidi order mismatch: {clusters:?}"),
                )?;
            }
            if let Some(expected) = case["caret_order"].as_array() {
                require(
                    json!(
                        layout.lines[0]
                            .carets
                            .iter()
                            .map(|c| c.byte)
                            .collect::<Vec<_>>()
                    ) == json!(expected),
                    format!("Authored visual caret order mismatch {id}"),
                )?;
            }
            if id == "mono-spacing" {
                let glyphs = &layout.lines[0].glyphs;
                require(
                    glyphs.len() == 4
                        && glyphs
                            .iter()
                            .all(|g| (g.advance - glyphs[0].advance).abs() < 0.002),
                    "Monospace fixture must have equal advances for i/W/0",
                )?;
            }
            if id == "ligature" {
                require(
                    layout.lines[0].glyphs.len() == 1 && layout.lines[0].carets.len() == 4,
                    "Ligature caret fixture",
                )?;
                require(
                    engine
                        .layout(
                            text,
                            Config {
                                ligatures: false,
                                ..cfg
                            },
                        )
                        .map_err(|e| format!("{e:?}"))?
                        .lines[0]
                        .glyphs
                        .len()
                        == 3,
                    "Feature change must reshape",
                )?;
                let rect = layout.lines[0]
                    .selection(1..2)
                    .map_err(|e| format!("{e:?}"))?;
                require(
                    rect.len() == 1
                        && (rect[0][1] - rect[0][0] - layout.lines[0].width / 3.0).abs() < 0.002,
                    "Authored partial ligature selection",
                )?;
            }
            let preview = format!("{id}-{scale}.pgm");
            let image_hash = render(&mut engine, &layout, cfg, &directory.join(&preview))?;
            results.push(json!({"case":id,"scale":scale,"missing":layout.missing.iter().map(|r|[r.start,r.end]).collect::<Vec<_>>(),"line_ends":layout.lines.iter().map(|l|l.bytes.end).collect::<Vec<_>>(),"glyphs":glyphs(&layout),"carets":layout.lines.iter().map(|l|l.carets.iter().map(|c|json!([c.byte,c.x])).collect::<Vec<_>>()).collect::<Vec<_>>(),"preview":preview,"image_sha256":image_hash,"shaping":"custom rustybuzz; native HarfBuzz reference where declared","bidi":"unicode-bidi; authored mixed order","fallback":"locked grapheme-level first-covering face or reported tofu","slint":"pending P2-06"}));
        }
    }
    let negative = negatives(&mut engine, &fonts)?;
    let composition = compositions(&engine)?;
    write_json(
        &directory.join("result.json"),
        &json!({"schema_version":1,"status":"passed","official_breaks":data,"observations":results,"negative":negative,"composition":composition,"guest_tested":false,"native_ime_tested":false,"candidate_rendered":false,"text_raster_rendered":true}),
    )?;
    println!("Text conformance checks passed: {}", directory.display());
    Ok(())
}
