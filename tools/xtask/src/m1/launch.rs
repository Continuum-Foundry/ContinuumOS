//! Validate packaged inputs and prepare an isolated QEMU invocation.
use super::{inputs::preflight, observe::observe, records::Expected, support::digest};
use crate::{Result, root, sanitize, write_json};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command, time::Duration};

pub(super) fn launch(
    dir: &Path,
    iso_dir: &Path,
    image: &Value,
    test: bool,
    rejection: Option<(&str, &str)>,
) -> Result<Value> {
    let pins = preflight(dir, "run-preflight")?;
    let deadline = Duration::from_millis(
        pins.lock["vm"]["deadline_ms"]
            .as_u64()
            .filter(|v| *v > 0 && *v <= 30000)
            .ok_or("Invalid locked VM deadline")?,
    );
    if digest(&root().join("config/m1-inputs.lock.json"))? != image["input_lock_sha256"]
        || digest(pins.file("firmware_code")?)? != image["firmware_code_sha256"]
        || digest(pins.file("firmware_vars")?)? != image["firmware_vars_template_sha256"]
    {
        return Err("Pinned launch inputs changed after packaging".into());
    }
    let iso = iso_dir.join("continuum.iso");
    if digest(&iso)? != image["iso_sha256"] {
        return Err("ISO changed after packaging".into());
    }
    let vars = dir.join("fresh firmware vars.fd");
    fs::copy(pins.file("firmware_vars")?, &vars).map_err(|e| e.to_string())?;
    let before = digest(&vars)?;
    if before != digest(pins.file("firmware_vars")?)? {
        return Err("Firmware copy mismatch".into());
    }
    let mut args = Vec::new();
    for a in pins.lock["vm"]["arguments"]
        .as_array()
        .ok_or("Missing locked VM arguments")?
    {
        let a = a.as_str().ok_or("Invalid VM argument")?;
        args.push(
            a.replace("<new-run>", &dir.to_string_lossy())
                .replace(
                    "<locked-code>",
                    &pins.file("firmware_code")?.to_string_lossy(),
                )
                .replace("<fresh-vars-copy>", &vars.to_string_lossy())
                .replace("<exact-iso>", &iso.to_string_lossy()),
        );
    }
    if test {
        args.extend([
            "-device".into(),
            "isa-debug-exit,iobase=0xf4,iosize=0x04".into(),
        ]);
    }
    // QEMU's drive/serial backends interpret commas, even in an argv value.
    if [dir, iso_dir, pins.file("firmware_code")?]
        .iter()
        .any(|p| p.to_string_lossy().contains(','))
    {
        return Err("QEMU backend paths containing commas are unsupported".into());
    }
    let id = image["build_id"].as_str().ok_or("Missing build identity")?;
    let session = image["host_session_id"]
        .as_str()
        .ok_or("Missing packaged session")?;
    let fixture = image["fixture"]
        .as_str()
        .ok_or("Missing packaged fixture")?;
    let mut vm = pins.lock["vm"].clone();
    if fixture == "scheduler-cpu" {
        let cpu = args
            .iter()
            .position(|a| a == "-cpu")
            .ok_or("Missing CPU argument")?
            + 1;
        if args[cpu] != "qemu64-v1,+apic" {
            return Err("Unexpected CPU baseline".into());
        }
        args[cpu].push_str(",-sse2");
        vm["cpu"] = json!(args[cpu]);
        vm["negative_test"] = json!("P3-05 required SSE2 absent; no user entry allowed");
    }
    let symbol = match fixture {
        "exception-invalid-opcode" | "exception-emergency" | "timer-exception" => {
            Some("diagnostic_invalid_opcode")
        }
        "exception-page-fault" => Some("diagnostic_page_fault_site"),
        "panic" => Some("diagnostic_panic_report"),
        "guard-fault" => Some("diagnostic_guard_site"),
        "write-protect-fault" => Some("diagnostic_write_site"),
        "execute-disable-fault" => Some("diagnostic_nx_page"),
        "emergency-stack" => Some("diagnostic_double_site"),
        _ => None,
    };
    let fault_rip = symbol
        .map(|s| {
            image["diagnostic_sites"][s]
                .as_u64()
                .ok_or("Missing fault symbol")
        })
        .transpose()?;
    write_json(
        &dir.join("launch-manifest.json"),
        &json!({"schema_version":2,"build_id":id,"iso_sha256":image["iso_sha256"],"configuration_sha256":image["configuration_sha256"],"fixture":fixture,"host_session_id":session,"session_origin":"host; correlation only, not a security credential or reset detector","firmware_code_sha256":digest(pins.file("firmware_code")?)?,"vars_template_sha256":before,"vars_copy_before_sha256":before,"vm":vm,"test_device":test,"arguments":args.iter().map(|a|sanitize(a)).collect::<Vec<_>>()}),
    )?;
    let mut c = Command::new(pins.file("qemu")?);
    c.args(&args);
    let mut report = observe(
        &mut c,
        dir,
        Expected {
            build: id,
            session,
            fixture,
            rejection,
            fault_rip,
        },
        test,
        deadline,
    )?;
    report["vars_copy_after_sha256"] = json!(digest(&vars)?);
    report["iso_sha256"] = image["iso_sha256"].clone();
    report["iso_unchanged_after_run"] = json!(digest(&iso)? == image["iso_sha256"]);
    if report["iso_unchanged_after_run"] != true {
        report["status"] = json!("failed");
        report["error"] = json!("ISO changed during run");
    }
    write_json(&dir.join("run-result.json"), &report)?;
    Ok(report)
}
