//! Host-only conformance tooling for draft data. No renderer, IPC or guest code.
use crate::{Result, hash, root, write_json};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

const NAMES: [&str; 7] = [
    "surface", "tokens", "layout", "corpus", "workflow", "variants", "paint",
];
const BASE: &str = "fixtures/m2/ui-v1";
type Corpus = BTreeMap<String, Value>;
fn check(ok: bool, reason: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(reason.into()) }
}
fn array(v: &Value) -> &[Value] {
    v.as_array().unwrap()
}
fn uint(v: &Value) -> u64 {
    v.as_u64().unwrap_or_else(|| v.as_f64().unwrap() as u64)
}
fn text(v: &Value) -> &str {
    v.as_str().unwrap()
}
fn number(v: &Value) -> f64 {
    v.as_f64().unwrap()
}
fn equivalent(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(_), Value::Number(_)) => a == b || a.as_f64() == b.as_f64(),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equivalent(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, a)| b.get(k).is_some_and(|b| equivalent(a, b)))
        }
        _ => a == b,
    }
}

/// Deliberately small closed schema vocabulary, independently checked by Test-Json.
/// Unsupported schema keywords fail, rather than silently weakening the validator.
fn shape(v: &Value, s: &Value) -> Result<()> {
    for key in s.as_object().ok_or("schema")?.keys() {
        check(
            [
                "$schema",
                "$id",
                "type",
                "const",
                "enum",
                "minimum",
                "maximum",
                "minLength",
                "maxLength",
                "minItems",
                "maxItems",
                "minProperties",
                "items",
                "properties",
                "required",
                "additionalProperties",
            ]
            .contains(&key.as_str()),
            "schema",
        )?;
    }
    if let Some(c) = s.get("const") {
        check(equivalent(v, c), "shape")?;
    }
    if let Some(e) = s.get("enum") {
        check(array(e).iter().any(|c| equivalent(v, c)), "shape")?;
    }
    if let Some(t) = s.get("type") {
        check(
            match text(t) {
                "object" => v.is_object(),
                "array" => v.is_array(),
                "string" => v.is_string(),
                "boolean" => v.is_boolean(),
                "integer" => {
                    v.as_u64().is_some()
                        || v.as_i64().is_some()
                        || v.as_f64()
                            .is_some_and(|n| n.is_finite() && n.fract() == 0.0)
                }
                "number" => v.as_f64().is_some_and(f64::is_finite),
                _ => return Err("schema".into()),
            },
            "shape",
        )?;
    }
    if let Some(n) = v.as_f64() {
        if let Some(lo) = s.get("minimum") {
            check(n >= number(lo), "shape")?;
        }
        if let Some(hi) = s.get("maximum") {
            check(n <= number(hi), "shape")?;
        }
    }
    if let Some(t) = v.as_str() {
        let count = t.chars().count() as u64;
        if let Some(lo) = s.get("minLength") {
            check(count >= uint(lo), "shape")?;
        }
        if let Some(hi) = s.get("maxLength") {
            check(count <= uint(hi), "shape")?;
        }
    }
    if let Some(a) = v.as_array() {
        if let Some(lo) = s.get("minItems") {
            check(a.len() as u64 >= uint(lo), "shape")?;
        }
        if let Some(hi) = s.get("maxItems") {
            check(a.len() as u64 <= uint(hi), "shape")?;
        }
        if let Some(item) = s.get("items") {
            for v in a {
                shape(v, item)?;
            }
        }
    }
    if let Some(o) = v.as_object() {
        if let Some(lo) = s.get("minProperties") {
            check(o.len() as u64 >= uint(lo), "shape")?;
        }
        if let Some(r) = s.get("required") {
            for k in array(r) {
                check(o.contains_key(text(k)), "shape")?;
            }
        }
        let props = s.get("properties");
        for (key, value) in o {
            if let Some(sub) = props.and_then(|p| p.get(key)) {
                shape(value, sub)?;
            } else {
                check(
                    s.get("additionalProperties") != Some(&json!(false)),
                    "shape",
                )?;
            }
        }
    }
    Ok(())
}

fn rect(v: &Value) -> Result<()> {
    check(
        array(v).len() == 4
            && array(v).iter().all(|n| {
                n.as_f64()
                    .is_some_and(|n| n.is_finite() && n.abs() <= 16384.0)
            })
            && number(&v[2]) >= 0.0
            && number(&v[3]) >= 0.0,
        "geometry",
    )
}
fn surface_bytes(width: u64, height: u64, stride: u64) -> Result<u64> {
    let row = width.checked_mul(4).ok_or("overflow")?;
    check(stride >= row && stride.is_multiple_of(4), "stride")?;
    stride.checked_mul(height).ok_or("overflow".into())
}
fn revision(v: &Value) -> Result<()> {
    check(
        v["base_revision"] == 3 && v["new_revision"] == 4,
        "revision",
    )
}
fn surface(v: &Value) -> Result<()> {
    revision(v)?;
    check(v["generation"] == 7, "generation")?;
    let bytes = surface_bytes(uint(&v["width"]), uint(&v["height"]), uint(&v["stride"]))?;
    check(
        bytes == uint(&v["byte_length"]) && bytes <= 64 * 1024 * 1024,
        "length",
    )?;
    for r in array(&v["damage"]) {
        rect(r)?;
        check(
            number(&r[0]) >= 0.0
                && number(&r[1]) >= 0.0
                && number(&r[0]) + number(&r[2]) <= number(&v["width"])
                && number(&r[1]) + number(&r[3]) <= number(&v["height"]),
            "damage",
        )?;
    }
    let mut leases: BTreeMap<u64, (&str, u64, u64)> = BTreeMap::new();
    for e in array(&v["leases"]) {
        let id = uint(&e["buffer"]);
        let generation = uint(&e["generation"]);
        let serial = uint(&e["lease"]);
        let op = text(&e["op"]);
        let (state, old, old_serial) = leases.get(&id).copied().unwrap_or(("released", 0, 0));
        match op {
            "acquire" => check(
                state == "released" && generation >= old && serial > old_serial,
                "lease",
            )?,
            "write" => check(
                state == "acquired" && generation == old && serial == old_serial,
                "lease",
            )?,
            "commit" => check(
                state == "acquired" && generation == old && serial == old_serial,
                "lease",
            )?,
            "release" => check(
                state == "in_use" && generation == old && serial == old_serial,
                "lease",
            )?,
            _ => return Err("lease".into()),
        }
        leases.insert(
            id,
            (
                match op {
                    "acquire" | "write" => "acquired",
                    "commit" => "in_use",
                    _ => "released",
                },
                generation,
                serial,
            ),
        );
    }
    check(leases.values().all(|(s, _, _)| *s == "released"), "lease")
}
fn walk(
    id: &str,
    nodes: &BTreeMap<&str, &Value>,
    seen: &mut BTreeSet<String>,
    depth: usize,
) -> Result<()> {
    check(depth <= 32 && seen.insert(id.into()), "tree")?;
    let node = nodes.get(id).ok_or("tree")?;
    for child in array(&node["children"]) {
        walk(text(child), nodes, seen, depth + 1)?;
    }
    Ok(())
}
fn selection(v: &Value) -> Result<()> {
    if let (Some(t), Some(s)) = (v.get("document"), v.get("selection")) {
        let t = text(t);
        let start = uint(&s[0]) as usize;
        let end = uint(&s[1]) as usize;
        check(
            start <= end && end <= t.len() && t.is_char_boundary(start) && t.is_char_boundary(end),
            "state",
        )?;
    }
    Ok(())
}
fn validate(c: &Corpus, schemas: &Corpus) -> Result<()> {
    for name in NAMES {
        shape(&c[name], &schemas[name])?;
    }
    let corpus = &c["corpus"];
    surface(&c["surface"])?;
    check(
        c["tokens"]["generation"] == 7 && c["layout"]["generation"] == 7,
        "generation",
    )?;
    revision(&c["layout"])?;
    for slot in c["tokens"]["fonts"].as_object().unwrap().values() {
        check(array(&corpus["font_slots"]).contains(slot), "resource")?;
    }
    selection(&corpus["initial_state"])?;
    let mut nodes = BTreeMap::new();
    let mut semantics = BTreeSet::new();
    for node in array(&c["layout"]["nodes"]) {
        check(
            nodes.insert(text(&node["id"]), node).is_none()
                && semantics.insert(text(&node["semantic"]["id"])),
            "identity",
        )?;
        rect(&node["rect"])?;
        if node["resource"] != "" {
            check(
                array(&corpus["resources"]).contains(&node["resource"]),
                "resource",
            )?;
        }
        if node["action"] != "" {
            check(
                array(&corpus["actions"]).contains(&node["action"]),
                "action",
            )?;
        }
        if node["focusable"] == true {
            check(node["action"] != "", "action")?;
        }
    }
    let mut seen = BTreeSet::new();
    walk(text(&c["layout"]["root"]), &nodes, &mut seen, 1)?;
    check(seen.len() == nodes.len(), "tree")?;
    let focusable = |id: &str| nodes.get(id).is_some_and(|n| n["focusable"] == true);
    check(focusable(text(&corpus["initial_state"]["focus"])), "focus")?;
    let mut seq = 0;
    let mut time = 0;
    let mut composition = false;
    let mut keys = BTreeSet::new();
    let mut buttons = BTreeSet::new();
    let mut capture = "";
    let mut step_ids = BTreeSet::new();
    for step in array(&c["workflow"]["steps"]) {
        check(step_ids.insert(text(&step["id"])), "identity")?;
        for event in array(&step["events"]) {
            check(
                uint(&event["sequence"]) == seq + 1 && uint(&event["time_us"]) >= time,
                "ordering",
            )?;
            seq += 1;
            time = uint(&event["time_us"]);
            check(event["scene_revision"] == 4, "revision")?;
            let kind = text(&event["kind"]);
            let target = text(&event["target"]);
            check(
                nodes.contains_key(target)
                    || (target.is_empty() && ["capture", "reset"].contains(&kind)),
                "identity",
            )?;
            match kind {
                "focus" => check(focusable(target), "focus")?,
                "capture" => {
                    check(target.is_empty() || focusable(target), "focus")?;
                    capture = target;
                }
                "key" => {
                    let usage = uint(&event["usage"]);
                    check(usage != 0, "input")?;
                    check(
                        if event["pressed"] == true {
                            keys.insert(usage)
                        } else {
                            keys.remove(&usage)
                        },
                        "input",
                    )?;
                }
                "text" => check(
                    !text(&event["text"]).is_empty() && !composition && focusable(target),
                    "input",
                )?,
                "composition" => match text(&event["phase"]) {
                    "start" => {
                        check(!composition, "input")?;
                        composition = true;
                    }
                    "update" => check(composition, "input")?,
                    "commit" | "cancel" => {
                        check(composition, "input")?;
                        composition = false;
                    }
                    _ => return Err("input".into()),
                },
                "pointer" => {
                    let button = uint(&event["button"]);
                    match text(&event["phase"]) {
                        "down" => check(button != 0 && buttons.insert(button), "input")?,
                        "up" => check(buttons.remove(&button), "input")?,
                        "move" => (),
                        _ => return Err("input".into()),
                    }
                }
                "reset" => {
                    composition = false;
                    keys.clear();
                    buttons.clear();
                    capture = "";
                }
                "scroll" => (),
                _ => return Err("input".into()),
            }
        }
        selection(&step["assertions"])?;
        if let Some(f) = step["assertions"].get("focus") {
            check(focusable(text(f)), "focus")?;
        }
        if let Some(a) = step["assertions"].get("action") {
            check(array(&corpus["actions"]).contains(a), "action")?;
        }
        if let Some(a) = step["assertions"].get("composition_active") {
            check(a == composition, "input")?;
        }
    }
    check(
        !composition && keys.is_empty() && buttons.is_empty() && capture.is_empty(),
        "input",
    )?;
    check(seq <= 4096, "input")?;
    for (index, variant) in array(&c["variants"]["variants"]).iter().enumerate() {
        check(variant["id"] == corpus["variants"][index], "variant")?;
    }
    for capture in array(&c["variants"]["capture_steps"]) {
        check(step_ids.contains(text(capture)), "variant")?;
    }
    for vector in array(&c["paint"]["premultiply"]) {
        let s = &vector["straight"];
        for i in 0..4 {
            let expected = if i == 3 {
                uint(&s[3])
            } else {
                (uint(&s[i]) * uint(&s[3]) + 127) / 255
            };
            check(uint(&vector["expected"][i]) == expected, "color")?;
        }
    }
    for vector in array(&c["paint"]["source_over"]) {
        let s = &vector["source"];
        let d = &vector["destination"];
        for i in 0..3 {
            check(
                uint(&s[i]) <= uint(&s[3]) && uint(&d[i]) <= uint(&d[3]),
                "color",
            )?;
        }
        for i in 0..4 {
            let expected = uint(&s[i]) + (uint(&d[i]) * (255 - uint(&s[3])) + 127) / 255;
            check(uint(&vector["expected"][i]) == expected, "color")?;
        }
    }
    check(
        c["paint"]["constant_blur"]["input"] == c["paint"]["constant_blur"]["expected"],
        "color",
    )
}

fn read(path: &str) -> Result<Value> {
    let bytes = fs::read(root().join(path)).map_err(|e| format!("{path}: {e}"))?;
    check(bytes.len() <= 1024 * 1024, "fixture-size")?;
    check(!bytes.contains(&b'\r'), "noncanonical-line-ending")?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))
}
fn load() -> Result<(Corpus, Corpus)> {
    let mut fixtures = BTreeMap::new();
    let mut schemas = BTreeMap::new();
    for name in NAMES {
        fixtures.insert(name.into(), read(&format!("{BASE}/{name}.json"))?);
        schemas.insert(
            name.into(),
            read(&format!(
                "docs/06-contracts/schemas/hosted-{name}.schema.json"
            ))?,
        );
    }
    Ok((fixtures, schemas))
}
fn mutated(c: &Corpus, case: &Value) -> Result<Corpus> {
    let mut c = c.clone();
    let file = c.get_mut(text(&case["file"])).ok_or("negative-file")?;
    let path = text(&case["path"]);
    let value = if let Some(repeat) = case.get("repeat") {
        let a = file
            .pointer(path)
            .and_then(Value::as_array)
            .ok_or("negative-path")?;
        Value::Array((0..uint(repeat)).flat_map(|_| a.iter().cloned()).collect())
    } else {
        case["value"].clone()
    };
    if path.is_empty() {
        *file = value;
    } else {
        let (parent, key) = path.rsplit_once('/').ok_or("negative-path")?;
        let p = file.pointer_mut(parent).ok_or("negative-path")?;
        if let Some(o) = p.as_object_mut() {
            o.insert(key.into(), value);
        } else if let Some(a) = p.as_array_mut() {
            *a.get_mut(key.parse::<usize>().map_err(|_| "negative-path")?)
                .ok_or("negative-path")? = value;
        } else {
            return Err("negative-path".into());
        }
    }
    Ok(c)
}
/// Canonical UTF-8 JSON lines; arithmetic wraps modulo 2^32, locale independent.
fn generated(seed: u32) -> Vec<u8> {
    let mut state = seed;
    let mut bytes = Vec::new();
    for i in 0..10000_u32 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let row = json!({"id":format!("file.{i:05}"),"label":format!("File {i:05} — {:08x}.txt",state),"size":state % 1048576});
        bytes.extend(serde_json::to_vec(&row).unwrap());
        bytes.push(b'\n');
    }
    bytes
}
fn acceptance(c: &Corpus, schemas: &Corpus) -> Result<Value> {
    validate(c, schemas)?;
    let negative = read(&format!("{BASE}/negative.json"))?;
    let mut rows = Vec::new();
    for case in array(&negative["cases"]) {
        let actual = validate(&mutated(c, case)?, schemas).err();
        check(
            actual.as_deref() == case["error"].as_str(),
            &format!(
                "negative {} expected {}, got {actual:?}",
                case["name"], case["error"]
            ),
        )?;
        rows.push(
            json!({"name":case["name"],"expected":case["error"],"actual":actual,"passed":true}),
        );
    }
    check(
        surface_bytes(u64::MAX, 1, 4) == Err("overflow".into()),
        "width-overflow-test",
    )?;
    check(
        surface_bytes(1, u64::MAX, 8) == Err("overflow".into()),
        "length-overflow-test",
    )?;
    for raw in [r#"{"x":NaN}"#, r#"{"x":Infinity}"#, r#"{"x":1e999}"#] {
        check(
            serde_json::from_str::<Value>(raw).is_err(),
            "nonfinite-parser-test",
        )?;
    }
    let profiles = read("config/m2-profiles.json")?;
    check(
        array(&profiles["scales"])
            .iter()
            .map(number)
            .collect::<Vec<_>>()
            == array(&c["corpus"]["scales"])
                .iter()
                .map(number)
                .collect::<Vec<_>>()
            && profiles["measurement"]["seed"] == c["corpus"]["seed"]
            && profiles["profiles"]["hd"] == json!({"width":1280,"height":720})
            && profiles["profiles"]["full-hd"] == json!({"width":1920,"height":1080}),
        "profile-lock",
    )?;
    let data = generated(20261020);
    check(
        data == generated(20261020) && data != generated(20261021),
        "determinism",
    )?;
    let lock = read(&format!("{BASE}/lock.json"))?;
    check(lock["schema_version"] == 1, "lock-version")?;
    let mut expected_paths = BTreeSet::from([
        format!("{BASE}/negative.json"),
        "config/m2-profiles.json".into(),
    ]);
    for name in NAMES {
        expected_paths.insert(format!("{BASE}/{name}.json"));
        expected_paths.insert(format!(
            "docs/06-contracts/schemas/hosted-{name}.schema.json"
        ));
    }
    let members = lock["members"].as_array().ok_or("lock-members")?;
    let actual_paths = members
        .iter()
        .map(|m| m["path"].as_str().unwrap_or("").to_owned())
        .collect::<BTreeSet<_>>();
    check(
        expected_paths == actual_paths && actual_paths.len() == members.len(),
        "lock-members",
    )?;
    check(
        lock["generated_file_list_sha256"] == hash(&data),
        "generated-hash",
    )?;
    for member in array(&lock["members"]) {
        let path = text(&member["path"]);
        check(
            !path.contains("..")
                && !path.contains('\\')
                && (path.starts_with(BASE)
                    || path.starts_with("docs/06-contracts/schemas/hosted-")
                    || path == "config/m2-profiles.json"),
            "lock-path",
        )?;
        let bytes = fs::read(root().join(path)).map_err(|e| e.to_string())?;
        check(
            member["sha256"] == hash(&bytes),
            &format!("lock mismatch: {path}"),
        )?;
    }
    Ok(
        json!({"schema_version":1,"status":"passed","positive_fixture_files":7,
        "negative_cases":rows,"arithmetic_overflow_cases":2,"nonfinite_parser_cases":3,
        "generated_file_list_sha256":hash(&data),"generated_rows":10000,
        "deterministic_bytes":true,"declared_profile_configurations":30,"declared_capture_cases":150,"render_tested":false,
        "action_execution_tested":false,"guest_tested":false}),
    )
}
pub(crate) fn checked_inputs() -> Result<Corpus> {
    let (c, schemas) = load()?;
    acceptance(&c, &schemas)?;
    Ok(c)
}
pub fn execute() -> Result<()> {
    let (c, schemas) = load()?;
    let result = acceptance(&c, &schemas)?;
    let run = root().join(format!(
        "build/m2/P2-02/run-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos(),
        std::process::id()
    ));
    fs::create_dir_all(&run).map_err(|e| e.to_string())?;
    fs::write(run.join("files.jsonl"), generated(20261020)).map_err(|e| e.to_string())?;
    write_json(&run.join("result.json"), &result)?;
    println!("Shared UI fixture checks passed: {}", run.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_corpus_and_all_negative_cases() {
        let (c, schemas) = load().unwrap();
        acceptance(&c, &schemas).unwrap();
    }
    #[test]
    fn unsupported_schema_keyword_rejects() {
        assert_eq!(shape(&json!(0), &json!({"not":{}})), Err("schema".into()));
    }
    #[test]
    fn schema_numeric_values_match_json_schema_semantics() {
        assert!(
            shape(
                &json!([1.0, 2.0]),
                &json!({"const":[1,2],"type":"array","items":{"type":"integer"}})
            )
            .is_ok()
        );
        assert!(shape(&json!(1.5), &json!({"type":"integer"})).is_err());
    }
}
