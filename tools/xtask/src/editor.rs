//! P2-03 headless replay. JSON fixture values are explicitly converted, never cast.
use crate::{Result, hash, root, write_json};
use editor_model::*;
use serde_json::{Value, json};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn err<T>(r: editor_model::Result<T>) -> Result<T> {
    r.map_err(|e| format!("editor model: {e:?}"))
}
fn integer(v: &Value) -> Result<u64> {
    v.as_u64()
        .filter(|n| *n <= 9_007_199_254_740_991)
        .ok_or_else(|| "Expected exact JSON unsigned integer".into())
}
fn string(v: &Value) -> Result<String> {
    v.as_str()
        .map(str::to_owned)
        .ok_or_else(|| "Expected string".into())
}
fn boolean(v: &Value) -> Result<bool> {
    v.as_bool().ok_or_else(|| "Expected bool".into())
}
fn u32_value(v: &Value) -> Result<u32> {
    u32::try_from(integer(v)?).map_err(|_| "u32 overflow".into())
}
fn selection(v: &Value) -> Result<Selection> {
    let a = v
        .as_array()
        .filter(|a| a.len() == 2)
        .ok_or("Expected selection pair")?;
    Ok(Selection::new(u32_value(&a[0])?, u32_value(&a[1])?))
}
fn keys(v: &Value, expected: &[&str]) -> Result<()> {
    let o = v.as_object().ok_or("Expected object")?;
    if o.len() != expected.len() || expected.iter().any(|key| !o.contains_key(*key)) {
        return Err("Missing or unknown field".into());
    }
    Ok(())
}
fn decode(v: &Value) -> Result<Request> {
    keys(
        v,
        &[
            "version",
            "caller",
            "id",
            "epoch",
            "document",
            "expected_document_revision",
            "expected_model_revision",
            "deadline",
            "context",
            "action",
        ],
    )?;
    keys(&v["action"], &["name", "value"])?;
    let name = string(&v["action"]["name"])?;
    let value = &v["action"]["value"];
    let action = match name.as_str() {
        "edit.insert" => Action::Insert(string(value)?),
        "edit.select" => Action::Select(selection(value)?),
        "search.set" => Action::Search(string(value)?),
        "document.scroll" => Action::Scroll(u32_value(value)?),
        "files.select" => Action::FileSelect(u32_value(value)?),
        "files.scroll" => Action::FileScroll(u32_value(value)?),
        "settings.motion" => Action::Motion(boolean(value)?),
        "settings.effects" => Action::Effects(boolean(value)?),
        "settings.text-size" => {
            Action::TextSize(u16::try_from(integer(value)?).map_err(|_| "u16 overflow")?)
        }
        "settings.theme" => Action::Theme(match value.as_str() {
            Some("light") => Theme::Light,
            Some("dark") => Theme::Dark,
            Some("high-contrast") => Theme::HighContrast,
            _ => return Err("Invalid theme".into()),
        }),
        "session.focus" => Action::Focus(string(value)?),
        "session.window" => {
            let a = value
                .as_array()
                .filter(|a| a.len() == 4)
                .ok_or("Expected window")?;
            let mut out = [0; 4];
            for (n, v) in out.iter_mut().zip(a) {
                *n = i32::try_from(v.as_i64().ok_or("Expected signed integer")?)
                    .map_err(|_| "i32 overflow")?;
            }
            Action::Window(out)
        }
        "edit.delete" | "edit.undo" | "edit.redo" | "launcher.open" => {
            if !value.is_null() {
                return Err("Expected null payload".into());
            }
            match name.as_str() {
                "edit.delete" => Action::Delete,
                "edit.undo" => Action::Undo,
                "edit.redo" => Action::Redo,
                _ => Action::LauncherOpen,
            }
        }
        _ => Action::Unknown(name),
    };
    Ok(Request {
        version: u16::try_from(integer(&v["version"])?).map_err(|_| "u16 overflow")?,
        caller: integer(&v["caller"])?,
        id: integer(&v["id"])?,
        epoch: integer(&v["epoch"])?,
        document: integer(&v["document"])?,
        expected_document_revision: integer(&v["expected_document_revision"])?,
        expected_model_revision: integer(&v["expected_model_revision"])?,
        deadline: integer(&v["deadline"])?,
        context: if v["context"].is_null() {
            None
        } else {
            Some(integer(&v["context"])?)
        },
        action,
    })
}
fn state(m: &Model) -> Value {
    let d = m.document();
    let p = m.preferences();
    let s = m.session();
    json!({"text":d.text(),"document_revision":d.revision(),"model_revision":m.revision(),"selection":[d.selection().anchor,d.selection().caret],"dirty":d.dirty(),"query":d.query(),"matches":d.matches(),"scroll":d.scroll(),"theme":format!("{:?}",p.theme),"text_size":p.text_size,"reduced_motion":p.reduced_motion,"reduced_effects":p.reduced_effects,"focus":s.focus,"window":s.window,"file_scroll":s.file_scroll,"selected_file":s.selected_file,"undo":d.undo_len(),"redo":d.redo_len()})
}
fn subset(actual: &Value, expected: &Value) -> Result<()> {
    for (key, value) in expected.as_object().ok_or("Expected assertion object")? {
        if actual.get(key) != Some(value) {
            return Err(format!(
                "Assertion {key}: expected {value}, actual {}",
                actual[key]
            ));
        }
    }
    Ok(())
}
pub fn execute() -> Result<()> {
    let fixture_path = root().join("fixtures/m2/actions-v1/replay.json");
    let raw = fs::read(&fixture_path).map_err(|e| e.to_string())?;
    let fixture: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    let initial = &fixture["initial"];
    let model = err(Model::new(
        integer(&initial["document"])?,
        integer(&initial["revision"])?,
        &string(&initial["text"])?,
        &string(&initial["saved"])?,
        selection(&initial["selection"])?,
    ))?;
    let hex = fs::read_to_string(root().join("fixtures/m2/actions-v1/initial.hex"))
        .map_err(|e| e.to_string())?;
    let hex = hex.trim();
    if hex.len() % 2 != 0 || !hex.is_ascii() {
        return Err("Invalid independent hex fixture".into());
    }
    let independent = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>>>()?;
    if err(model.encode())? != independent || err(Model::decode(&independent))? != model {
        return Err("Independent checkpoint fixture differs".into());
    }
    let run = root().join(format!(
        "build/m2/P2-03/run-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos(),
        std::process::id()
    ));
    fs::create_dir_all(&run).map_err(|e| e.to_string())?;
    let result: Result<Value> = (|| {
        let mut entries = Vec::new();
        let mut baseline = None;
        for entry in [
            Entry::Button,
            Entry::Shortcut,
            Entry::Semantic,
            Entry::Automation,
        ] {
            let mut provider = err(Provider::new(model.clone(), 7))?;
            err(provider.grant(1, Rights::ALL))?;
            let mut traces = Vec::new();
            for step in fixture["steps"].as_array().ok_or("Missing steps")? {
                let request = decode(&step["request"])?;
                let before = hash(&err(provider.model().encode())?);
                // Each iteration destroys and recreates the borrowed view adapter.
                let outcome = Adapter::new(&mut provider, entry).invoke(request.clone(), 1, false);
                subset(
                    &json!({"changed":err(outcome.status)?,"document_revision":outcome.document_revision,"model_revision":outcome.model_revision}),
                    &json!({"changed":step["expected"]["changed"],"document_revision":step["expected"]["document_revision"],"model_revision":step["expected"]["model_revision"]}),
                )?;
                let actual = state(provider.model());
                let mut expected = step["expected"].clone();
                expected
                    .as_object_mut()
                    .ok_or("assertions")?
                    .remove("changed");
                subset(&actual, &expected)?;
                let checkpoint = err(provider.model().encode())?;
                if err(Model::decode(&checkpoint))? != *provider.model() {
                    return Err("Snapshot roundtrip differs".into());
                }
                err(provider.restore(&checkpoint))?;
                traces.push(json!({"request_id":request.id,"action":request.action.name(),"version":request.version,"before_sha256":before,"after_sha256":hash(&checkpoint),"outcome":format!("{:?}",outcome.status),"state":actual}));
            }
            let actual = state(provider.model());
            subset(&actual, &fixture["final"])?;
            let bytes = err(provider.model().encode())?;
            if let Some(ref prior) = baseline {
                if prior != &bytes {
                    return Err("Entry parity failed".into());
                }
            } else {
                baseline = Some(bytes.clone());
            }
            let name = format!("{entry:?}").to_lowercase();
            fs::write(run.join(format!("{name}.checkpoint")), &bytes).map_err(|e| e.to_string())?;
            entries.push(json!({"entry":name,"steps":traces,"final_sha256":hash(&bytes),"final_state":actual}));
        }
        let negative = negative_cases(&model)?;
        Ok(
            json!({"schema_version":1,"status":"passed","fixture_sha256":hash(&raw),"state_version":STATE_VERSION,"action_version":ACTION_VERSION,"unicode_version":UNICODE_VERSION,"entries":entries,"negative_cases":negative,"guest_tested":false,"render_tested":false,"provider_replacement_tested":false}),
        )
    })();
    match result {
        Ok(report) => {
            write_json(&run.join("result.json"), &report)?;
            println!("Document action checks passed: {}", run.display());
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
fn negative_cases(model: &Model) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for entry in [
        Entry::Button,
        Entry::Shortcut,
        Entry::Semantic,
        Entry::Automation,
    ] {
        for name in [
            "stale-document",
            "stale-model",
            "wrong-epoch",
            "denied-rights",
            "unknown-caller",
            "unsupported-version",
            "unknown-action",
            "fabricated-context",
            "expired-context",
            "context-scope",
            "invalid-selection",
            "oversize-edit",
            "cancelled",
            "deadline",
        ] {
            let mut p = err(Provider::new(model.clone(), 7))?;
            err(p.grant(1, Rights::ALL))?;
            let mut r = Request {
                version: 1,
                caller: 1,
                id: 1,
                epoch: 7,
                document: 1,
                expected_document_revision: 11,
                expected_model_revision: 1,
                deadline: 100,
                context: None,
                action: Action::Insert("x".into()),
            };
            let mut cancel = false;
            let mut now = 1;
            let expected = match name {
                "stale-document" => {
                    r.expected_document_revision = 10;
                    Error::Conflict
                }
                "stale-model" => {
                    r.expected_model_revision = 2;
                    Error::Conflict
                }
                "wrong-epoch" => {
                    r.epoch = 8;
                    Error::Conflict
                }
                "denied-rights" => {
                    err(p.grant(
                        1,
                        Rights {
                            edit: false,
                            preferences: true,
                            session: true,
                        },
                    ))?;
                    Error::Denied
                }
                "unknown-caller" => {
                    r.caller = 2;
                    Error::Denied
                }
                "unsupported-version" => {
                    r.version = 2;
                    Error::Unsupported
                }
                "unknown-action" => {
                    r.action = Action::Unknown("file.save".into());
                    Error::Unsupported
                }
                "fabricated-context" => {
                    r.context = Some(99);
                    Error::Denied
                }
                "expired-context" | "context-scope" => {
                    err(p.issue_context(Context {
                        id: 3,
                        caller: 1,
                        provider_epoch: 7,
                        document: 1,
                        source_revision: 11,
                        provenance: "synthetic.owner".into(),
                        purpose: Effect::Document,
                        expires: 1,
                        size: 7,
                        selection: Selection::new(7, 14),
                        permitted_action: if name == "context-scope" {
                            "edit.delete"
                        } else {
                            "edit.insert"
                        }
                        .into(),
                    }))?;
                    r.context = Some(3);
                    if name == "context-scope" {
                        Error::Denied
                    } else {
                        Error::Expired
                    }
                }
                "invalid-selection" => {
                    r.action = Action::Select(Selection::new(999, 999));
                    Error::Invalid
                }
                "oversize-edit" => {
                    r.action = Action::Insert("x".repeat(MAX_QUERY + 1));
                    Error::Bounds
                }
                "cancelled" => {
                    cancel = true;
                    Error::Cancelled
                }
                _ => {
                    now = 100;
                    Error::Deadline
                }
            };
            let before = hash(&err(p.model().encode())?);
            let outcome = Adapter::new(&mut p, entry).invoke(r, now, cancel);
            let after = hash(&err(p.model().encode())?);
            if outcome.status != Err(expected) || before != after {
                return Err(format!("Negative case {name}/{entry:?} failed"));
            }
            rows.push(json!({"name":name,"entry":format!("{entry:?}"),"expected":format!("{expected:?}"),"actual":format!("{:?}",outcome.status),"before_sha256":before,"after_sha256":after}));
        }
    }
    Ok(rows)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_decoder_rejects_unknown_keys_overflows_and_wrong_payloads() {
        let f: Value =
            serde_json::from_str(include_str!("../../../fixtures/m2/actions-v1/replay.json"))
                .unwrap();
        let v = &f["steps"][0]["request"];
        for mutation in 0..5 {
            let mut x = v.clone();
            match mutation {
                0 => x["pointer"] = json!(17),
                1 => x["action"]["callback"] = json!("eval"),
                2 => x["version"] = json!(65536),
                3 => x["action"]["value"] = json!(3),
                _ => x["id"] = json!(-1),
            };
            assert!(decode(&x).is_err());
        }
    }
}
