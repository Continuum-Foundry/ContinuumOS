//! Strict version-2 guest record assertions. Firmware/loader output is separate.
use super::process::LIMIT;
use crate::Result;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub(super) struct Expected<'a> {
    pub build: &'a str,
    pub session: &'a str,
    pub fixture: &'a str,
    pub rejection: Option<(&'a str, &'a str)>,
}
#[derive(Default)]
pub(super) struct Progress {
    pub entry: bool,
    pub validated: bool,
    pub complete: bool,
    pub rejected: bool,
    pub records: Vec<Value>,
}
impl Progress {
    pub fn ready(&self) -> bool {
        self.complete || self.rejected
    }
}
pub(super) fn check(serial: &[u8], expected: Expected<'_>, final_log: bool) -> Result<Progress> {
    if serial.len() as u64 > LIMIT {
        return Err("Serial output limit exceeded".into());
    }
    let mut p = Progress::default();
    let mut sequence = 0u64;
    let (mut regions, mut modules, mut framebuffers, mut reservations, mut acpi) = (0, 0, 0, 0, 0);
    let (mut want_regions, mut want_modules, mut want_framebuffers, mut want_acpi) = (0, 0, 0, 0);
    for line in serial.split_inclusive(|b| *b == b'\n') {
        if !line.starts_with(b"m1.") {
            continue;
        }
        if line.last() != Some(&b'\n') {
            if final_log {
                return Err("Incomplete guest record".into());
            }
            break;
        }
        if line.len() > 512 {
            return Err("Guest record length exceeded".into());
        }
        let text = std::str::from_utf8(line)
            .map_err(|_| "Non-UTF8 guest record")?
            .trim_end_matches(['\r', '\n']);
        let mut words = text.split_whitespace();
        let event = words.next().ok_or("Empty record")?;
        let mut fields = BTreeMap::new();
        for word in words {
            let (k, v) = word.split_once('=').ok_or("Malformed guest field")?;
            if v.is_empty() || fields.insert(k, v).is_some() {
                return Err("Empty/duplicate guest field".into());
            }
        }
        let get = |key: &str| -> Result<&str> {
            fields
                .get(key)
                .copied()
                .ok_or_else(|| format!("Missing guest field {key}"))
        };
        let number = |key: &str| -> Result<usize> {
            get(key)?
                .parse()
                .map_err(|_| format!("Invalid guest number {key}"))
        };
        let hex = |key: &str| -> Result<u64> {
            u64::from_str_radix(
                get(key)?
                    .strip_prefix("0x")
                    .ok_or("Missing hexadecimal prefix")?,
                16,
            )
            .map_err(|_| format!("Invalid hexadecimal guest field {key}"))
        };
        if get("version")? != "2"
            || get("build")? != expected.build
            || get("session")? != expected.session
            || get("session_origin")? != "host"
        {
            return Err("Guest version/build/session identity mismatch".into());
        }
        if get("seq")?.parse::<u64>().map_err(|_| "Invalid sequence")? != sequence {
            return Err("Guest sequence gap/duplicate".into());
        }
        sequence += 1;
        if p.ready() {
            return Err("Guest record after terminal outcome".into());
        }
        let (stage, outcome) = (get("stage")?, get("outcome")?);
        match event {
            "m1.kernel_entry" => {
                if p.entry
                    || sequence != 1
                    || stage != "entry"
                    || outcome != "ok"
                    || get("fixture")? != expected.fixture
                {
                    return Err("Duplicate/invalid kernel entry".into());
                }
                p.entry = true;
            }
            "m1.boot_info_validated" => {
                if !p.entry
                    || p.validated
                    || expected.rejection.is_some()
                    || stage != "metadata"
                    || outcome != "ok"
                    || get("boot_info")? != "1"
                {
                    return Err("Unexpected validation-success marker".into());
                }
                want_regions = number("regions")?;
                want_modules = number("modules")?;
                want_framebuffers = number("framebuffers")?;
                if want_regions == 0
                    || want_regions > 512
                    || want_modules > 16
                    || want_framebuffers > 4
                {
                    return Err("Invalid summary counts".into());
                }
                if get("mode")?
                    != if want_framebuffers == 0 {
                        "serial-only"
                    } else {
                        "framebuffer"
                    }
                {
                    return Err("Inconsistent framebuffer mode".into());
                }
                want_acpi = match get("acpi")? {
                    "locator-only" => 1,
                    "absent" => 0,
                    _ => return Err("Invalid ACPI state".into()),
                };
                hex("hhdm")?;
                p.validated = true;
            }
            "m1.boot_rejected" => {
                let (field, reason) = expected.rejection.ok_or("Unexpected guest rejection")?;
                if !p.entry
                    || p.validated
                    || stage != "metadata"
                    || outcome != "rejected"
                    || get("field")? != field
                    || get("reason")? != reason
                {
                    return Err("Wrong guest rejection".into());
                }
                p.rejected = true;
            }
            "m1.memory_region" | "m1.module" | "m1.framebuffer" | "m1.acpi_locator"
            | "m1.reservation" => {
                if !p.validated || stage != "summary" || outcome != "ok" {
                    return Err("Resource before validation".into());
                }
                if event != "m1.acpi_locator" {
                    let (base, bytes) = (hex("base")?, hex("bytes")?);
                    if bytes == 0 || base.checked_add(bytes).is_none() {
                        return Err("Invalid resource summary range".into());
                    }
                }
                match event {
                    "m1.memory_region" => {
                        if number("index")? != regions {
                            return Err("Memory index mismatch".into());
                        }
                        regions += 1;
                        number("kind")?;
                    }
                    "m1.module" => {
                        if number("index")? != modules {
                            return Err("Module index mismatch".into());
                        }
                        modules += 1;
                        if number("path_bytes")? >= 256 || number("argument_bytes")? >= 256 {
                            return Err("Invalid module string summary".into());
                        }
                    }
                    "m1.framebuffer" => {
                        if number("index")? != framebuffers {
                            return Err("Framebuffer index mismatch".into());
                        }
                        framebuffers += 1;
                        for key in ["width", "height", "pitch", "bpp"] {
                            if number(key)? == 0 {
                                return Err("Invalid framebuffer summary".into());
                            }
                        }
                        for key in ["red", "green", "blue", "cache"] {
                            get(key)?;
                        }
                    }
                    "m1.acpi_locator" => {
                        if get("table_validated")? != "false" {
                            return Err("Unexpected ACPI parsing claim".into());
                        }
                        acpi += 1;
                        if hex("physical")? == 0 {
                            return Err("Invalid ACPI locator summary".into());
                        }
                    }
                    _ => {
                        if get("retained")? != "true" {
                            return Err("Reservation not retained".into());
                        }
                        reservations += 1;
                        if ![
                            "Kernel",
                            "ExecutableFile",
                            "Module",
                            "Framebuffer",
                            "LoaderLive",
                            "Acpi",
                            "Firmware",
                        ]
                        .contains(&get("reason")?)
                        {
                            return Err("Unknown reservation reason".into());
                        }
                    }
                }
                if regions > want_regions
                    || modules > want_modules
                    || framebuffers > want_framebuffers
                    || acpi > want_acpi
                {
                    return Err("Excess resource records".into());
                }
            }
            "m1.boot_complete" => {
                if !p.validated
                    || stage != "boot"
                    || outcome != "ok"
                    || get("reclaimed_bytes")? != "0"
                    || (regions, modules, framebuffers, acpi)
                        != (want_regions, want_modules, want_framebuffers, want_acpi)
                    || reservations < 2 + modules + framebuffers
                {
                    return Err("Incomplete resource/reservation summary".into());
                }
                p.complete = true;
            }
            _ => return Err(format!("Unexpected guest event {event}")),
        }
        p.records.push(json!({"event":event,"fields":fields}));
    }
    Ok(p)
}
pub(super) fn assertions(
    serial: &[u8],
    expected: Expected<'_>,
    code: Option<i32>,
    timed_out: bool,
) -> Result<()> {
    if timed_out {
        return Err("Guest deadline exceeded".into());
    }
    let p = check(serial, expected, true)?;
    if !p.ready() {
        return Err("Missing complete guest outcome".into());
    }
    let exit = if expected.rejection.is_some() { 35 } else { 33 };
    if code != Some(exit) {
        return Err(format!("Expected emulator exit {exit}, got {code:?}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn line(event: &str, seq: usize, stage: &str, outcome: &str, detail: &str) -> String {
        format!(
            "{event} version=2 build=abc session=def session_origin=host seq={seq} stage={stage} outcome={outcome} {detail}\n"
        )
    }
    fn good() -> String {
        line("m1.kernel_entry", 0, "entry", "ok", "fixture=normal")
            + &line(
                "m1.boot_info_validated",
                1,
                "metadata",
                "ok",
                "boot_info=1 regions=1 modules=0 framebuffers=0 mode=serial-only acpi=absent hhdm=0xffff800000000000",
            )
            + &line(
                "m1.memory_region",
                2,
                "summary",
                "ok",
                "index=0 base=0x1000 bytes=0x1000 kind=5",
            )
            + &line(
                "m1.reservation",
                3,
                "summary",
                "ok",
                "base=0x1000 bytes=0x1000 reason=LoaderLive retained=true",
            )
            + &line(
                "m1.reservation",
                4,
                "summary",
                "ok",
                "base=0x2000 bytes=0x1000 reason=Kernel retained=true",
            )
            + &line("m1.boot_complete", 5, "boot", "ok", "reclaimed_bytes=0")
    }
    #[test]
    fn rejects_false_or_incomplete_success() {
        let e = Expected {
            build: "abc",
            session: "def",
            fixture: "normal",
            rejection: None,
        };
        let good = good();
        assert!(assertions(good.as_bytes(), e, Some(33), false).is_ok());
        for bad in [
            good.replace("session=def", "session=other"),
            good.replace("build=abc", "build=other"),
            good.replace("seq=2", "seq=1"),
            good.replace("version=2", "version=1"),
            good.clone() + &line("m1.panic", 6, "panic", "rejected", "reason=panic"),
            good.clone() + &line("m1.kernel_entry", 6, "entry", "ok", "fixture=normal"),
            good.replace("regions=1", "regions=2"),
            good.replace("retained=true", "retained=false"),
            good.replace("session=def", "session=def session=def"),
            good[..good.len() - 1].into(),
            String::new(),
        ] {
            assert!(
                assertions(bad.as_bytes(), e, Some(33), false).is_err(),
                "{bad}"
            );
        }
        for code in [None, Some(0), Some(35)] {
            assert!(assertions(good.as_bytes(), e, code, false).is_err());
        }
        assert!(assertions(good.as_bytes(), e, Some(33), true).is_err());
        assert!(check(&vec![b'x'; LIMIT as usize + 1], e, false).is_err());
        assert!(!check(b"m1.kernel_entry", e, false).unwrap().ready());
        assert!(check(b"m1.kernel_entry", e, true).is_err());
    }
    #[test]
    fn expected_rejection_needs_exact_error_and_exit_and_no_ready_marker() {
        let e = Expected {
            build: "abc",
            session: "def",
            fixture: "missing-required",
            rejection: Some(("required", "missing")),
        };
        let rejected = line(
            "m1.kernel_entry",
            0,
            "entry",
            "ok",
            "fixture=missing-required",
        ) + &line(
            "m1.boot_rejected",
            1,
            "metadata",
            "rejected",
            "field=required reason=missing",
        );
        assert!(assertions(rejected.as_bytes(), e, Some(35), false).is_ok());
        assert!(assertions(rejected.as_bytes(), e, Some(33), false).is_err());
        assert!(
            assertions(
                rejected
                    .replace("reason=missing", "reason=other")
                    .as_bytes(),
                e,
                Some(35),
                false
            )
            .is_err()
        );
        assert!(
            assertions(
                good()
                    .replace("fixture=normal", "fixture=missing-required")
                    .as_bytes(),
                e,
                Some(35),
                false
            )
            .is_err()
        );
        assert!(
            assertions(
                (rejected + &line("m1.boot_complete", 2, "boot", "ok", "reclaimed_bytes=0"))
                    .as_bytes(),
                e,
                Some(35),
                false
            )
            .is_err()
        );
    }
}
