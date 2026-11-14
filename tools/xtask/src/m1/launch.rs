//! Validate packaged inputs and prepare an isolated QEMU invocation.
use super::{
    inputs::preflight,
    observe::observe,
    records::Expected,
    support::{digest, json_read},
};
use crate::{Result, sanitize, write_json};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command, time::Duration};

fn deadline_ms(flavor: &str, baseline: u64, profile: &Value) -> Result<u64> {
    if baseline != 30000 {
        return Err("Ordinary M1 deadline must remain 30000 ms".into());
    }
    if matches!(flavor, "stress-test" | "stress-trace") {
        if profile["schema_version"] != 2
            || profile["deadline_profile"] != "m3-stress-tcg-v2"
            || profile["original_guest_deadline_ms"] != 30000
            || profile["guest_deadline_ms"] != 60000
        {
            return Err("Unsupported scoped M3 stress deadline profile".into());
        }
        Ok(60000)
    } else {
        Ok(baseline)
    }
}

pub(super) fn launch(
    dir: &Path,
    iso_dir: &Path,
    image: &Value,
    test: bool,
    rejection: Option<(&str, &str)>,
) -> Result<Value> {
    let pins = preflight(dir, "run-preflight")?;
    let flavor = image["build_flavor"]
        .as_str()
        .ok_or("Missing packaged build flavor")?;
    let stress = matches!(flavor, "stress-test" | "stress-trace");
    let profile_path = crate::root().join("config/p3-13-stress.json");
    let profile = if stress {
        json_read(&profile_path)?
    } else {
        Value::Null
    };
    let deadline = Duration::from_millis(deadline_ms(
        flavor,
        pins.lock["vm"]["deadline_ms"]
            .as_u64()
            .ok_or("Invalid locked VM deadline")?,
        &profile,
    )?);
    let stress_profile_hash = if stress {
        let hash = digest(&profile_path)?;
        let inputs = json_read(
            &dir.parent()
                .ok_or("Missing artifact parent")?
                .join("build-inputs.json"),
        )?;
        if inputs["flavor"] != flavor || inputs["sources"]["config/p3-13-stress.json"] != hash {
            return Err("Stress profile changed after kernel build".into());
        }
        Some(hash)
    } else {
        None
    };
    if digest(&crate::input_lock_path())? != image["input_lock_sha256"]
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
    if flavor == "device-test" {
        args.extend(
            [
                "-device",
                "virtio-keyboard-pci,disable-legacy=on,addr=4",
                "-device",
                "virtio-tablet-pci,disable-legacy=on,addr=5",
                "-device",
                "virtio-serial-pci,disable-legacy=on,addr=6",
                "-chardev",
                "null,id=p403",
                "-device",
                "virtserialport,chardev=p403,name=org.continuum.dev",
            ]
            .map(String::from),
        );
    }
    if flavor == "display-test" {
        args.extend(["-qmp".into(), "stdio".into()]);
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
    vm["deadline_ms"] = json!(deadline.as_millis());
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
    let mut launch_manifest = json!({"schema_version":2,"build_id":id,"iso_sha256":image["iso_sha256"],"configuration_sha256":image["configuration_sha256"],"fixture":fixture,"host_session_id":session,"session_origin":"host; correlation only, not a security credential or reset detector","firmware_code_sha256":digest(pins.file("firmware_code")?)?,"vars_template_sha256":before,"vars_copy_before_sha256":before,"vm":vm,"test_device":test,"arguments":args.iter().map(|a|sanitize(a)).collect::<Vec<_>>()});
    if let Some(hash) = stress_profile_hash {
        launch_manifest["deadline_profile"] = json!("m3-stress-tcg-v2");
        launch_manifest["stress_profile_sha256"] = json!(hash);
        launch_manifest["original_guest_deadline_ms"] = json!(30000);
    }
    write_json(&dir.join("launch-manifest.json"), &launch_manifest)?;
    let mut c = Command::new(pins.file("qemu")?);
    c.args(&args);
    if flavor == "display-test" {
        c.stdin(std::process::Stdio::piped());
    }
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

#[cfg(test)]
mod deadline_tests {
    use super::*;

    #[test]
    fn slower_host_deadline_is_scoped_to_two_fixed_stress_flavors() {
        let profile = json!({"schema_version":2,"deadline_profile":"m3-stress-tcg-v2","original_guest_deadline_ms":30000,"guest_deadline_ms":60000});
        for flavor in [
            "normal",
            "sdk-runtime-test",
            "route-test",
            "stress-supervisor-test",
        ] {
            assert_eq!(deadline_ms(flavor, 30000, &Value::Null).unwrap(), 30000);
            assert_eq!(deadline_ms(flavor, 30000, &profile).unwrap(), 30000);
            assert!(deadline_ms(flavor, 60000, &profile).is_err());
        }
        for flavor in ["stress-test", "stress-trace"] {
            assert_eq!(deadline_ms(flavor, 30000, &profile).unwrap(), 60000);
            for (field, bad) in [
                ("schema_version", json!(1)),
                ("deadline_profile", json!("unbounded")),
                ("original_guest_deadline_ms", json!(60000)),
                ("guest_deadline_ms", json!(60001)),
                ("guest_deadline_ms", json!(true)),
            ] {
                let mut altered = profile.clone();
                altered[field] = bad;
                assert!(deadline_ms(flavor, 30000, &altered).is_err());
            }
        }
    }
}
