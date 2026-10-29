//! Fresh session configuration, ISO construction and extracted-payload checks.
use super::{
    inputs::preflight,
    process::capture,
    support::{digest, read},
};
use crate::{Result, hash, root, write_json};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn config(missing: bool, session: &str, fixture: &str, module: bool) -> String {
    format!(
        "timeout: 0\nserial: yes\nverbose: yes\nremember_last_entry: no\n/ContinuumOS\n    protocol: limine\n    path: boot():/boot/{}\n    cmdline: session={session} fixture={fixture}\n{}",
        if missing {
            "nonexistent.elf"
        } else {
            "kernel.elf"
        },
        if module {
            "    module_path: boot():/boot/fixture.bin\n    module_string: p1-03-fixture\n"
        } else {
            ""
        }
    )
}
// This pinned MSYS2 runtime uses /cygdrive, and xorriso treats a native C:\\...
// source path as relative. argv quoting alone does not translate filesystem paths.
fn iso_path(path: &Path) -> Result<String> {
    let s = path.to_string_lossy().replace('\\', "/");
    let b = s.as_bytes();
    if b.len() < 3 || !b[0].is_ascii_alphabetic() || &b[1..3] != b":/" {
        return Err("xorriso requires an absolute Windows drive path (UNC unsupported)".into());
    }
    Ok(format!(
        "/cygdrive/{}/{}",
        (b[0] as char).to_ascii_lowercase(),
        &s[3..]
    ))
}
pub(super) fn package(
    dir: &Path,
    build_dir: &Path,
    manifest: &Value,
    missing: bool,
    fixture: &str,
    module: bool,
) -> Result<Value> {
    let pins = preflight(dir, "package-preflight")?;
    let stage = dir.join("iso root with spaces");
    let files = [
        (build_dir.join("kernel.elf"), "boot/kernel.elf"),
        (
            pins.suffix("limine-binary/limine-uefi-cd.bin")?.to_owned(),
            "boot/limine/limine-uefi-cd.bin",
        ),
        (
            pins.suffix("limine-binary/BOOTX64.EFI")?.to_owned(),
            "EFI/BOOT/BOOTX64.EFI",
        ),
        (
            pins.suffix("limine-binary/LICENSE")?.to_owned(),
            "notices/limine-LICENSE",
        ),
        (
            pins.suffix("limine-binary/3RDPARTY.md")?.to_owned(),
            "notices/limine-3RDPARTY.md",
        ),
        (
            pins.suffix("limine-0.5.0/LICENSE-MIT")?.to_owned(),
            "notices/limine-rs-LICENSE-MIT",
        ),
        (
            pins.suffix("limine-0.5.0/LICENSE-APACHE")?.to_owned(),
            "notices/limine-rs-LICENSE-APACHE",
        ),
    ];
    if digest(&files[0].0)? != manifest["kernel_sha256"] {
        return Err("Kernel changed since build".into());
    }
    let mut payload = BTreeMap::new();
    for (source, name) in files {
        let target = stage.join(name);
        fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::copy(source, &target).map_err(|e| e.to_string())?;
        payload.insert(name.to_string(), digest(&target)?);
    }
    // Generated before configuration/packaging; the kernel build is independent.
    let session = hash(
        format!(
            "{}:{}:{}",
            dir.display(),
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos()
        )
        .as_bytes(),
    );
    let mut configuration = config(missing, &session, fixture, module);
    if fixture == "static-loader"
        || fixture.starts_with("user-boundary-")
        || fixture.starts_with("scheduler-")
        || fixture == "capabilities"
    {
        for name in ["user_a", "user_b", "service"] {
            let filename = format!("boot/{name}.elf");
            fs::copy(build_dir.join(format!("{name}.elf")), stage.join(&filename))
                .map_err(|e| e.to_string())?;
            payload.insert(filename.clone(), digest(&stage.join(&filename))?);
            configuration.push_str(&format!(
                "    module_path: boot():/{filename}\n    module_string: p3-03-{name}\n"
            ));
        }
    }
    fs::write(stage.join("boot/limine/limine.conf"), configuration).map_err(|e| e.to_string())?;
    if module {
        fs::write(
            stage.join("boot/fixture.bin"),
            b"P1-03 immutable module fixture\n",
        )
        .map_err(|e| e.to_string())?;
        payload.insert(
            "boot/fixture.bin".into(),
            digest(&stage.join("boot/fixture.bin"))?,
        );
    }
    payload.insert(
        "boot/limine/limine.conf".into(),
        digest(&stage.join("boot/limine/limine.conf"))?,
    );
    let iso = dir.join("continuum.iso");
    let mut c = Command::new(pins.file("xorriso")?);
    c.args([
        "-as",
        "mkisofs",
        "-R",
        "-r",
        "-J",
        "--modification-date=2026102000000000",
        "--set_all_file_dates",
        "2026102000000000",
        "--gpt_disk_guid",
        "61464790-57e2-4a64-a66d-7c6fd0e6770a",
        "--efi-boot",
        "boot/limine/limine-uefi-cd.bin",
        "-efi-boot-part",
        "--efi-boot-image",
        "--protective-msdos-label",
    ])
    .arg(iso_path(&stage)?)
    .arg("-o")
    .arg(iso_path(&iso)?);
    capture(&mut c, dir, "xorriso-package", 60)?;
    let mut c = Command::new(pins.file("xorriso")?);
    let catalog = capture(
        c.arg("-indev")
            .arg(iso_path(&iso)?)
            .args(["-report_el_torito", "plain"]),
        dir,
        "optical-catalog",
        30,
    )?;
    // xorriso may send reports to stderr; preserve and inspect both streams.
    let catalog = format!(
        "{catalog}\n{}",
        String::from_utf8_lossy(&read(&dir.join("optical-catalog.stderr.log"))?)
    );
    if !catalog.lines().any(|l| {
        l.contains("El Torito boot img")
            && l.contains("UEFI")
            && l.split_whitespace().any(|w| w == "y")
    }) {
        return Err("No bootable UEFI optical catalog entry".into());
    }
    let extracted = dir.join("extracted payload");
    let mut c = Command::new(pins.file("xorriso")?);
    capture(
        c.arg("-osirrox")
            .arg("on")
            .arg("-indev")
            .arg(iso_path(&iso)?)
            .args(["-extract", "/"])
            .arg(iso_path(&extracted)?),
        dir,
        "extract-payload",
        30,
    )?;
    for (name, expected) in &payload {
        if digest(&extracted.join(name))? != *expected {
            return Err(format!("ISO payload mismatch: {name}"));
        }
    }
    let report = json!({"schema_version":2,"build_id":manifest["build_id"],"kernel_sha256":manifest["kernel_sha256"],"diagnostic_sites":manifest["inspection"]["diagnostic_sites"],"host_session_id":session,"session_origin":"host; correlation only, not a security credential or reset detector","fixture":fixture,"module_fixture":module,"build_flavor":manifest["flavor"],"configuration_sha256":payload["boot/limine/limine.conf"],"iso_sha256":digest(&iso)?,"payload":payload,"uefi_optical_catalog":true,"payload_extraction_verified":true,"fault":if missing {"missing-configured-kernel"} else {fixture},"input_lock_sha256":digest(&root().join("config/m1-inputs.lock.json"))?,"firmware_code_sha256":digest(pins.file("firmware_code")?)?,"firmware_vars_template_sha256":digest(pins.file("firmware_vars")?)?,"bios_tested":false,"boot_tested":false});
    write_json(&dir.join("iso-manifest.json"), &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_paths_are_translated_without_shell_quoting() {
        assert_eq!(
            iso_path(Path::new("C:\\space directory\\image.iso")).unwrap(),
            "/cygdrive/c/space directory/image.iso"
        );
        assert!(iso_path(Path::new("relative.iso")).is_err());
        assert!(iso_path(Path::new("\\\\server\\share")).is_err());
    }
}
