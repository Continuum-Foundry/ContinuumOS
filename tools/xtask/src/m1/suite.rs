//! Complete M1 gate. A failed or absent row prevents a passing suite result.
use super::{
    build::{Flavor, build, source_inputs},
    cases::{accepted, boot_run, perform as case},
    process::capture,
    support::{digest, json_read, read, subdir},
};
use crate::{Result, hash, root, sanitize, tool, write_json};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path, process::Command};

const CASES: &[&str] = &[
    "run",
    "iso-smoke",
    "iso-negative",
    "boot-metadata",
    "boot-reject",
    "exception-invalid-opcode",
    "exception-page-fault",
    "panic",
    "exception-emergency",
    "memory-ownership",
    "memory-exhaustion",
    "guard-fault",
    "write-protect-fault",
    "execute-disable-fault",
    "emergency-stack",
    "timer-progress",
    "timer-masked",
    "clock-read",
    "idle-wakeup",
    "timer-exception",
    "acpi-corrupt",
    "timer-unsupported",
    "timer-calibration",
];

fn normalize_session(bytes: &[u8], session: &str) -> Result<(Vec<u8>, Vec<usize>)> {
    if session.len() != 64 || !session.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid comparison session".into());
    }
    let offsets = bytes
        .windows(64)
        .enumerate()
        .filter_map(|(i, b)| (b == session.as_bytes()).then_some(i))
        .collect::<Vec<_>>();
    if offsets.is_empty() {
        return Err("Session absent from ISO comparison".into());
    }
    let mut normalized = bytes.to_vec();
    for &i in &offsets {
        normalized[i..i + 64].fill(b'0');
    }
    Ok((normalized, offsets))
}

fn comparison(a: &Value, b: &Value, iso_a: &[u8], iso_b: &[u8]) -> Result<Value> {
    if a["build_id"] != b["build_id"]
        || a["kernel_sha256"] != b["kernel_sha256"]
        || a["host_session_id"] == b["host_session_id"]
        || iso_a == iso_b
    {
        return Err("Clean kernel/session comparison failed".into());
    }
    let mut payload_a = a["payload"].as_object().ok_or("Missing payload")?.clone();
    let mut payload_b = b["payload"].as_object().ok_or("Missing payload")?.clone();
    payload_a
        .remove("boot/limine/limine.conf")
        .ok_or("Missing configuration")?;
    payload_b
        .remove("boot/limine/limine.conf")
        .ok_or("Missing configuration")?;
    if payload_a != payload_b {
        return Err("Clean ISO payload differs beyond configuration".into());
    }
    let (na, oa) = normalize_session(
        iso_a,
        a["host_session_id"].as_str().ok_or("Missing session")?,
    )?;
    let (nb, ob) = normalize_session(
        iso_b,
        b["host_session_id"].as_str().ok_or("Missing session")?,
    )?;
    if na != nb || oa != ob {
        return Err("Unexplained ISO differences beyond session bytes".into());
    }
    Ok(
        json!({"kernel_bytes_identical":true,"build_ids_identical":true,
        "iso_bytes_identical":false,"iso_a_sha256":hash(iso_a),"iso_b_sha256":hash(iso_b),
        "normalized_iso_sha256":hash(&na),"session_byte_offsets":oa,
        "only_iso_difference":"64 ASCII host-session bytes at each listed offset",
        "timestamps":"volume and all file dates fixed to 2026102000000000",
        "gpt_guid":"61464790-57e2-4a64-a66d-7c6fd0e6770a; fixed experimental-image disk GUID and derived partition GUIDs",
        "order_and_path_effects":"whole normalized ISO byte comparison includes directory order, catalog and padding; second clean target path includes spaces",
        "release_reproducibility_claimed":false}),
    )
}

// Independently walk completed runs, checking every launch rather than trusting
// an aggregate count. Skip bulk Cargo and ISO extraction/staging directories.
fn inventory(
    dir: &Path,
    sources: &Value,
    builds: &mut Vec<Value>,
    launches: &mut Vec<Value>,
) -> Result<()> {
    if dir.join("build-inputs.json").is_file() {
        let inputs = json_read(&dir.join("build-inputs.json"))?;
        if inputs["sources"] != *sources {
            return Err("Stale suite build source inventory".into());
        }
        let manifest = json_read(&dir.join("build-manifest.json"))?;
        if digest(&dir.join("kernel.elf"))? != manifest["kernel_sha256"] {
            return Err("Suite kernel bytes changed".into());
        }
        builds.push(json!({"directory":sanitize(&dir.to_string_lossy()),"manifest":manifest,"inputs":inputs}));
    }
    if dir.join("run-result.json").is_file() {
        let r = json_read(&dir.join("run-result.json"))?;
        let launch = json_read(&dir.join("launch-manifest.json"))?;
        if r["cleanup_reaped"] != true
            || r["iso_unchanged_after_run"] != true
            || r["deadline_ms"] != 30000
            || launch["vm"]["deadline_ms"] != 30000
        {
            return Err("Suite lifecycle/deadline check failed".into());
        }
        // Only the two deliberately broken ISO fixtures may have failed boots.
        if r["status"] != "passed"
            && !["no-entry-run", "bad-config-run"]
                .iter()
                .any(|n| dir.file_name().is_some_and(|v| v == *n))
        {
            return Err("Unexpected failed launch in suite".into());
        }
        launches
            .push(json!({"directory":sanitize(&dir.to_string_lossy()),"result":r,"launch":launch}));
    }
    for e in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        if e.file_type().map_err(|e| e.to_string())?.is_dir()
            && !["cargo", "iso root with spaces", "extracted payload"]
                .iter()
                .any(|n| e.file_name() == *n)
        {
            inventory(&e.path(), sources, builds, launches)?;
        }
    }
    Ok(())
}

pub(super) fn perform(dir: &Path) -> Result<Value> {
    let sources = serde_json::to_value(source_inputs()?).map_err(|e| e.to_string())?;
    write_json(&dir.join("suite-source-inputs.json"), &sources)?;
    let mut rows = Vec::new();
    // Each command has separate raw logs, argv and a fixed host deadline.
    for (name, exe, args) in [
        ("format", "cargo", vec!["fmt", "--all", "--check"]),
        (
            "m0",
            "pwsh",
            vec!["-NoProfile", "-File", "tools/Test-M0.ps1"],
        ),
        (
            "dependencies",
            "pwsh",
            vec!["-NoProfile", "-File", "tools/Probe-Dependencies.ps1"],
        ),
        (
            "kernel-host",
            "cargo",
            vec!["test", "--locked", "-p", "continuum-kernel", "--lib"],
        ),
        (
            "doctor",
            "cargo",
            vec!["xtask", "doctor", "--profile", "m1", "--json"],
        ),
        (
            "prerequisites",
            "pwsh",
            vec!["-NoProfile", "-File", "tools/Test-P1-01.ps1"],
        ),
        ("docs", "node", vec!["audit/check-docs.cjs"]),
        (
            "contracts",
            "pwsh",
            vec!["-NoProfile", "-File", "audit/validate-contracts.ps1"],
        ),
        ("governance", "node", vec!["audit/check-governance.cjs"]),
    ] {
        println!("M1 suite: {name}");
        let check = subdir(dir, name)?;
        capture(
            Command::new(tool(exe)?).current_dir(root()).args(args),
            &check,
            "check",
            600,
        )?;
        rows.push(json!({"case":name,"status":"passed"}));
        write_json(
            &dir.join("suite-progress.json"),
            &json!({"status":"running","rows":rows}),
        )?;
    }
    let a_dir = subdir(dir, "clean-a")?;
    let b_dir = subdir(dir, "clean b with spaces")?;
    let a = build(&a_dir, Flavor::Normal)?;
    let b = build(&b_dir, Flavor::Normal)?;
    // Normal kernel, explicit emulator-only exit configuration, no fault feature.
    let first = boot_run(
        dir,
        "final-1",
        &a_dir,
        &a,
        "acceptance-exit",
        false,
        true,
        false,
    )?;
    accepted(&first)?;
    for n in 2..=5 {
        let r = boot_run(
            dir,
            &format!("final-{n}"),
            &a_dir,
            &a,
            "acceptance-exit",
            false,
            true,
            false,
        )?;
        accepted(&r)?;
    }
    let second = boot_run(
        dir,
        "second-clean",
        &b_dir,
        &b,
        "acceptance-exit",
        false,
        true,
        false,
    )?;
    accepted(&second)?;
    let compared = comparison(
        &first["package"],
        &second["package"],
        &read(&dir.join("final-1-package/continuum.iso"))?,
        &read(&dir.join("second-clean-package/continuum.iso"))?,
    )?;
    write_json(&dir.join("clean-comparison.json"), &compared)?;
    rows.push(
        json!({"case":"final-normal-boots","status":"passed","count":5,"second_clean_boot":true}),
    );
    for action in CASES {
        println!("M1 suite: {action}");
        let child = subdir(dir, action)?;
        let stages = case(action, &child)?;
        write_json(
            &child.join("result.json"),
            &json!({"schema_version":2,"action":action,"status":"passed","stages":stages}),
        )?;
        rows.push(json!({"case":action,"status":"passed"}));
        write_json(
            &dir.join("suite-progress.json"),
            &json!({"status":"running","rows":rows}),
        )?;
    }
    if sources != serde_json::to_value(source_inputs()?).map_err(|e| e.to_string())? {
        return Err("Build sources changed during acceptance".into());
    }
    let (mut builds, mut launches) = (Vec::new(), Vec::new());
    inventory(dir, &sources, &mut builds, &mut launches)?;
    let sessions = launches
        .iter()
        .map(|l| {
            l["result"]["host_session_id"]
                .as_str()
                .ok_or_else(|| "Missing launch session".to_string())
        })
        .collect::<Result<BTreeSet<_>>>()?;
    if launches.len() != 42 || sessions.len() != launches.len() || rows.len() != 33 {
        return Err("Missing/duplicate required suite row or launch".into());
    }
    // Same flavor must have exactly the same build ID and ELF throughout.
    for a in &builds {
        for b in &builds {
            if a["manifest"]["flavor"] == b["manifest"]["flavor"]
                && (a["manifest"]["build_id"] != b["manifest"]["build_id"]
                    || a["manifest"]["kernel_sha256"] != b["manifest"]["kernel_sha256"])
            {
                return Err("Within-flavor artifact identity mismatch".into());
            }
        }
    }
    write_json(
        &dir.join("suite-inventory.json"),
        &json!({"builds":builds,"launches":launches}),
    )?;
    Ok(
        json!({"matrix":"P1-07","required_cases":CASES,"rows":rows,"launch_count":launches.len(),
        "distinct_sessions":sessions.len(),"sources_unchanged":true,"clean_comparison":compared,
        "limits":"single-core experimental Q35/TCG baseline; no desktop, isolation, scheduler, persistence, live update, physical support or release reproducibility"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn iso_comparison_rejects_unexplained_changes() {
        let sa = "a".repeat(64);
        let sb = "b".repeat(64);
        let a = json!({"build_id":"id","kernel_sha256":"elf","host_session_id":sa,"payload":{"boot/limine/limine.conf":"ca","kernel":"same"}});
        let b = json!({"build_id":"id","kernel_sha256":"elf","host_session_id":sb,"payload":{"boot/limine/limine.conf":"cb","kernel":"same"}});
        let ba = format!("header{sa}padding").into_bytes();
        let bb = format!("header{sb}padding").into_bytes();
        assert!(comparison(&a, &b, &ba, &bb).is_ok());
        assert!(comparison(&a, &b, &ba, &[bb.clone(), vec![1]].concat()).is_err());
        assert!(comparison(&a, &b, &ba, &ba).is_err());
        assert!(comparison(&a, &b, b"missing", &bb).is_err());
        let mut altered = b.clone();
        altered["payload"]["kernel"] = json!("other");
        assert!(comparison(&a, &altered, &ba, &bb).is_err());
    }
}
