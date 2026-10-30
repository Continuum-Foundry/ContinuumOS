//! Named story tests; every launch gets its own immutable configured ISO.
use super::{
    build::{Flavor, build},
    inputs::preflight,
    launch::launch,
    package::package,
    process::read_log,
    support::{fresh, json_read, subdir},
};
use crate::{Result, sanitize, write_json};
use serde_json::{Value, json};
use std::path::Path;

const REJECTIONS: &[(&str, &str, &str)] = &[
    ("missing-required", "hhdm", "missing"),
    ("unsupported-revision", "memory", "revision"),
    ("range-overflow", "memory", "overflow"),
    ("excessive-count", "memory", "capacity"),
    ("unterminated-string", "executable_path", "unterminated"),
    ("framebuffer-pitch", "framebuffer", "pitch"),
    ("framebuffer-format", "framebuffer", "format"),
    ("framebuffer-range", "framebuffer", "range"),
];
fn rejection(fixture: &str) -> Option<(&'static str, &'static str)> {
    REJECTIONS
        .iter()
        .find(|(name, _, _)| *name == fixture)
        .map(|(_, field, reason)| (*field, *reason))
}
pub(super) fn boot_run(
    dir: &Path,
    label: &str,
    build_dir: &Path,
    manifest: &Value,
    fixture: &str,
    module: bool,
    test: bool,
    missing: bool,
) -> Result<Value> {
    let packaged = subdir(dir, &format!("{label}-package"))?;
    let image = package(&packaged, build_dir, manifest, missing, fixture, module)?;
    let run = subdir(dir, &format!("{label}-run"))?;
    let mut report = launch(&run, &packaged, &image, test, rejection(fixture))?;
    report["package"] = image;
    report["artifact_directory"] = json!(sanitize(&run.to_string_lossy()));
    if report["status"] == "passed" && rejection(fixture).is_none() {
        let records = json_read(&run.join("guest-records.json"))?;
        let rows = records["records"]
            .as_array()
            .ok_or("Missing guest records")?;
        let summary = rows
            .iter()
            .find(|r| r["event"] == "m1.boot_info_validated")
            .ok_or("Missing checked summary")?;
        let fields = &summary["fields"];
        let want_modules = if fixture == "static-loader"
            || fixture.starts_with("user-boundary-")
            || fixture.starts_with("scheduler-")
            || fixture == "capabilities"
            || fixture == "channels"
        {
            "3"
        } else if module {
            "1"
        } else {
            "0"
        };
        if fields["modules"] != want_modules
            || fields["acpi"] != "locator-only"
            || (fixture == "serial-only"
                && (fields["framebuffers"] != "0" || fields["mode"] != "serial-only"))
            || (fixture != "serial-only" && fields["framebuffers"] == "0")
        {
            return Err("Baseline/module/serial-only resource assertion failed".into());
        }
        report["checked_summary"] = fields.clone();
    }
    write_json(&run.join("case-result.json"), &report)?;
    Ok(report)
}
fn compare_identity(a: &Value, b: &Value) -> Result<Value> {
    if a["build_id"] != b["build_id"]
        || a["package"]["kernel_sha256"] != b["package"]["kernel_sha256"]
        || a["host_session_id"] == b["host_session_id"]
        || a["iso_sha256"] == b["iso_sha256"]
    {
        return Err("Fresh launch identity comparison failed".into());
    }
    Ok(
        json!({"same_build_id":true,"same_kernel_bytes":true,"distinct_host_and_guest_sessions":true,"distinct_configuration_isos":true}),
    )
}
pub(super) fn perform(action: &str, dir: &Path) -> Result<Value> {
    if action == "suite" {
        return super::suite::perform(dir);
    }
    if action != "build" {
        preflight(dir, "initial-preflight")?;
    }
    let flavor = match action {
        "owned-memory" => Flavor::RuntimeMemory,
        "static-loader" => Flavor::StaticLoader,
        "user-boundary" => Flavor::UserBoundary,
        "scheduler" => Flavor::Scheduler,
        "capabilities" => Flavor::Capabilities,
        "channels" => Flavor::Channels,
        "iso-smoke" | "boot-metadata" => Flavor::Smoke,
        "iso-negative" => Flavor::NoEntry,
        "boot-reject" => Flavor::Inject,
        "exception-invalid-opcode" | "exception-page-fault" | "panic" | "exception-emergency" => {
            Flavor::Exceptions
        }
        "memory-ownership"
        | "memory-exhaustion"
        | "guard-fault"
        | "write-protect-fault"
        | "execute-disable-fault"
        | "emergency-stack" => Flavor::Memory,
        "timer-progress" | "timer-masked" | "clock-read" | "idle-wakeup" | "timer-exception"
        | "acpi-corrupt" | "timer-unsupported" | "timer-calibration" => Flavor::Timer,
        _ => Flavor::Normal,
    };
    let manifest = build(dir, flavor)?;
    if action == "build" {
        return Ok(json!({"compile":"passed","package":"not_run","boot":"not_run"}));
    }
    if action == "iso" {
        let image = package(dir, dir, &manifest, false, "normal", false)?;
        return Ok(json!({"compile":"passed","package":"passed","boot":"not_run","image":image}));
    }
    if matches!(flavor, Flavor::Capabilities | Flavor::Channels) {
        let run = boot_run(
            dir,
            action,
            dir,
            &manifest,
            if flavor == Flavor::Channels {
                "channels"
            } else {
                "capabilities"
            },
            false,
            true,
            false,
        )?;
        accepted(&run)?;
        return Ok(
            json!({"compile":"passed","package":"passed","capabilities":"passed","run":run}),
        );
    }
    if flavor == Flavor::Scheduler {
        let mut runs = Vec::new();
        for fixture in ["scheduler-on", "scheduler-off", "scheduler-cpu"] {
            let run = boot_run(dir, fixture, dir, &manifest, fixture, false, true, false)?;
            accepted(&run)?;
            runs.push(run);
        }
        return Ok(json!({"compile":"passed","package":"passed","scheduler":"passed","runs":runs}));
    }
    if flavor == Flavor::UserBoundary {
        let mut runs = Vec::new();
        for suffix in ["a", "b", "c", "d"] {
            let fixture = format!("user-boundary-{suffix}");
            let run = boot_run(dir, &fixture, dir, &manifest, &fixture, false, true, false)?;
            accepted(&run)?;
            runs.push(run);
        }
        return Ok(
            json!({"compile":"passed","package":"passed","user_boundary":"passed","user_execution":true,"runs":runs}),
        );
    }
    if flavor == Flavor::StaticLoader {
        let run = boot_run(dir, action, dir, &manifest, action, false, true, false)?;
        accepted(&run)?;
        return Ok(
            json!({"compile":"passed","package":"passed","static_loader":"passed","user_execution":false,"run":run}),
        );
    }
    if flavor == Flavor::RuntimeMemory {
        let run = boot_run(dir, action, dir, &manifest, action, false, true, false)?;
        accepted(&run)?;
        let records = json_read(&dir.join("owned-memory-run/guest-records.json"))?;
        if records["records"]
            .as_array()
            .is_none_or(|rows| !rows.iter().any(|r| r["event"] == "m1.runtime_memory_ready"))
        {
            return Err("Missing runtime memory completion".into());
        }
        return Ok(
            json!({"compile":"passed","package":"passed","owned_memory":"passed","user_execution":false,"run":run}),
        );
    }
    if flavor == Flavor::Timer {
        let a = boot_run(dir, action, dir, &manifest, action, false, true, false)?;
        accepted(&a)?;
        let mut runs = vec![a];
        if action == "clock-read" {
            let b = boot_run(
                dir,
                "fresh-epoch",
                dir,
                &manifest,
                action,
                false,
                true,
                false,
            )?;
            accepted(&b)?;
            let identities = compare_identity(&runs[0], &b)?;
            runs.push(b);
            return Ok(
                json!({"compile":"passed","package":"passed","timer":"passed","identities":identities,"runs":runs}),
            );
        }
        return Ok(
            json!({"compile":"passed","package":"passed","timer":"passed","positive_boot": !["acpi-corrupt","timer-unsupported","timer-calibration","timer-exception"].contains(&action),"runs":runs}),
        );
    }
    if flavor == Flavor::Exceptions
        || (flavor == Flavor::Memory
            && !["memory-ownership", "memory-exhaustion"].contains(&action))
    {
        let run = boot_run(dir, action, dir, &manifest, action, false, true, false)?;
        accepted(&run)?;
        return Ok(
            json!({"compile":"passed","package":"passed","expected_fatal":"passed","positive_boot":false,"run":run}),
        );
    }
    if flavor == Flavor::Memory {
        let run = boot_run(dir, action, dir, &manifest, action, false, true, false)?;
        accepted(&run)?;
        return Ok(
            json!({"compile":"passed","package":"passed","memory":"passed","positive_boot":true,"run":run}),
        );
    }
    if action == "iso-negative" {
        let no_entry = boot_run(
            dir, "no-entry", dir, &manifest, "normal", false, true, false,
        )?;
        if no_entry["status"] != "failed"
            || no_entry["timed_out"] != true
            || no_entry["kernel_entry_observed"] != false
            || no_entry["cleanup_reaped"] != true
        {
            return Err("No-entry fixture did not fail with a clean timeout".into());
        }
        let good_dir = subdir(dir, "bad-config-kernel")?;
        let good = build(&good_dir, Flavor::Smoke)?;
        let bad = boot_run(
            dir,
            "bad-config",
            &good_dir,
            &good,
            "normal",
            false,
            true,
            true,
        )?;
        if bad["status"] != "failed"
            || bad["kernel_entry_observed"] != false
            || bad["cleanup_reaped"] != true
        {
            return Err("Missing configured kernel not rejected".into());
        }
        let diagnostic = plain_console(&read_log(&dir.join("bad-config-run/serial.log"))?);
        if !diagnostic.contains(
            "PANIC: limine: Failed to open executable with path `boot():/boot/nonexistent.elf`",
        ) {
            return Err("Missing actual Limine missing-executable panic".into());
        }
        return Ok(
            json!({"compile":"passed","package":"passed","negative_assertions":"passed","positive_boot":false,"no_entry":no_entry,"missing_configured_kernel":bad}),
        );
    }
    if action == "boot-reject" {
        let mut runs = Vec::new();
        for &(fixture, field, reason) in REJECTIONS {
            let report = boot_run(dir, fixture, dir, &manifest, fixture, false, true, false)?;
            accepted(&report)?;
            runs.push(json!({"fixture":fixture,"expected_field":field,"expected_reason":reason,"positive_boot":false,"run":report}));
        }
        return Ok(
            json!({"compile":"passed","package":"passed","rejection_assertions":"passed","positive_boot":false,"runs":runs}),
        );
    }
    let a = boot_run(
        dir,
        "cold-boot-a",
        dir,
        &manifest,
        "normal",
        false,
        action != "run",
        false,
    )?;
    accepted(&a)?;
    if action == "run" {
        return Ok(
            json!({"compile":"passed","package":"passed","boot_metadata":"observed","host_cleanup":"passed","run":a}),
        );
    }
    let b = boot_run(
        dir,
        "cold-boot-b",
        dir,
        &manifest,
        "normal",
        false,
        true,
        false,
    )?;
    accepted(&b)?;
    let identities = compare_identity(&a, &b)?;
    if action == "iso-smoke" {
        return Ok(
            json!({"compile":"passed","package":"passed","boot_metadata":"passed","identities":identities,"runs":[a,b]}),
        );
    }
    let module = boot_run(dir, "module", dir, &manifest, "normal", true, true, false)?;
    accepted(&module)?;
    let inject_dir = subdir(dir, "injection-kernel")?;
    let inject = build(&inject_dir, Flavor::Inject)?;
    let serial = boot_run(
        dir,
        "serial-only",
        &inject_dir,
        &inject,
        "serial-only",
        false,
        true,
        false,
    )?;
    accepted(&serial)?;
    Ok(
        json!({"compile":"passed","package":"passed","boot_metadata":"passed","identities":identities,"runs":[a,b,module,serial]}),
    )
}
pub(super) fn execute(action: &str) -> Result<()> {
    let dir = fresh(action)?;
    println!("M1 artifacts: {}", dir.display());
    match perform(action, &dir) {
        Ok(stages) => {
            write_json(
                &dir.join("result.json"),
                &json!({"schema_version":2,"action":action,"status":"passed","stages":stages}),
            )?;
            println!("{action}: passed; {}", dir.display());
            Ok(())
        }
        Err(e) => {
            write_json(
                &dir.join("result.json"),
                &json!({"schema_version":2,"action":action,"status":"failed","error":sanitize(&e)}),
            )?;
            Err(format!("{e}; artifacts retained at {}", dir.display()))
        }
    }
}

pub(super) fn accepted(r: &Value) -> Result<()> {
    if r["status"] == "passed" {
        Ok(())
    } else {
        Err(format!("VM failed: {}", r["error"]))
    }
}
fn plain_console(bytes: &[u8]) -> String {
    let mut output = String::new();
    let text = String::from_utf8_lossy(bytes);
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.next() == Some('[') {
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
        } else {
            output.push(c);
        }
    }
    output
}
