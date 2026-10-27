//! Strict version-5 guest record assertions. Firmware/loader output is separate.
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
    pub fault_rip: Option<u64>,
}
#[derive(Default)]
pub(super) struct Progress {
    pub entry: bool,
    pub validated: bool,
    pub complete: bool,
    pub boot_complete: bool,
    pub rejected: bool,
    pub exceptions: bool,
    pub fatal: bool,
    pub memory: bool,
    pub timer: bool,
    pub records: Vec<Value>,
}
impl Progress {
    pub fn ready(&self) -> bool {
        self.complete || self.rejected || self.fatal
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
    let mut runtime = None;
    let mut double = None;
    let mut switched = None;
    let mut frames = Vec::new();
    let mut armed = None;
    let mut usable = Vec::new();
    let mut reserved_ranges = Vec::new();
    let mut physical_regions = Vec::new();
    let mut discovery = false;
    let mut calibrated = None;
    let mut timer_ready = false;
    let mut samples = 0;
    let mut last_ticks = 0u64;
    let mut last_ns = 0u64;
    let mut masked = false;
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
        if get("version")? != "5"
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
        if get("emergency")?
            != if expected.fixture == "exception-emergency" && event == "m1.exception" {
                "true"
            } else {
                "false"
            }
        {
            return Err("Unexpected emergency logging state".into());
        }
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
                if !p.validated || p.exceptions || stage != "summary" || outcome != "ok" {
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
                        physical_regions.push((hex("base")?, hex("bytes")?, number("kind")?));
                        if number("kind")? == 0 {
                            usable.push((hex("base")?, hex("bytes")?));
                        }
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
                        reserved_ranges.push((hex("base")?, hex("bytes")?));
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
            "m1.exceptions_ready" => {
                if !p.validated
                    || p.exceptions
                    || stage != "exceptions"
                    || outcome != "ok"
                    || (regions, modules, framebuffers, acpi)
                        != (want_regions, want_modules, want_framebuffers, want_acpi)
                    || reservations < 2 + modules + framebuffers
                {
                    return Err("Incomplete resource/reservation summary".into());
                }
                if number("idt_entries")? != 256
                    || number("stack_bytes")? != 65536
                    || get("double_ist")? != "1"
                    || get("nmi_ist")? != "2"
                    || get("guards")? != "false"
                    || get("if")? != "false"
                {
                    return Err("Invalid exception setup".into());
                }
                let bases = [hex("runtime_base")?, hex("double_base")?, hex("nmi_base")?];
                for (i, &base) in bases.iter().enumerate() {
                    if base < 0xffff800000000000
                        || base % 4096 != 0
                        || base.checked_add(65536).is_none()
                        || bases[..i]
                            .iter()
                            .any(|&b| base < b + 65536 && b < base + 65536)
                    {
                        return Err("Invalid/overlapping exception stacks".into());
                    }
                }
                runtime = Some(bases[0]);
                double = Some(bases[1]);
                p.exceptions = true;
            }
            "m1.memory_switch" => {
                if !p.exceptions
                    || switched.is_some()
                    || stage != "memory"
                    || outcome != "ok"
                    || hex("before")? == hex("after")?
                    || hex("after")? == 0
                    || hex("after")? % 4096 != 0
                    || number("table_capacity")? != 2304
                    || number("table_pages")? > 2304
                    || number("table_pages")? < 4
                    || hex("physical_limit")? != 1 << 32
                    || number("eligible_frames")? < 8
                    || number("eligible_frames")? != number("mapped_direct_frames")?
                    || number("kernel_pages")? < 48
                    || get("reclaimed_bytes")? != "0"
                {
                    return Err("Invalid memory switch/accounting".into());
                }
                switched = Some(hex("after")?);
            }
            "m1.frame_test" => {
                let physical = hex("physical")?;
                if switched.is_none()
                    || p.memory
                    || stage != "memory"
                    || outcome != "ok"
                    || number("index")? != frames.len()
                    || frames.len() >= 8
                    || physical % 4096 != 0
                    || physical >= 1 << 32
                    || !usable
                        .iter()
                        .any(|&(b, n)| physical >= b && physical + 4096 <= b + n)
                    || reserved_ranges
                        .iter()
                        .any(|&(b, n)| physical < b + n && b < physical + 4096)
                    || frames.contains(&physical)
                    || get("zeroed")? != "true"
                    || get("unique")? != "true"
                    || get("reserved")? != "false"
                {
                    return Err("Invalid allocated frame/ownership evidence".into());
                }
                frames.push(physical);
            }
            "m1.memory_ready" => {
                if p.memory
                    || switched != Some(hex("cr3")?)
                    || frames.len() != 8
                    || stage != "memory"
                    || outcome != "ok"
                    || number("guards")? != 6
                    || number("aliases")? != 0
                    || number("batch")? != 8
                    || number("allocated")? != 0
                    || number("test_quota")?
                        != if expected.fixture == "memory-exhaustion" {
                            8
                        } else {
                            0
                        }
                    || get("exhaustion")?
                        != if expected.fixture == "memory-exhaustion" {
                            "passed"
                        } else {
                            "not-requested"
                        }
                {
                    return Err("Incomplete memory readiness".into());
                }
                for field in [
                    "wp",
                    "nx",
                    "supervisor",
                    "image_audited",
                    "direct_audited",
                    "reuse_zero",
                ] {
                    if get(field)? != "true" {
                        return Err("Missing enforced memory property".into());
                    }
                }
                for field in ["invalid_free", "double_free", "conflict"] {
                    if get(field)? != "rejected" {
                        return Err("Missing memory rejection check".into());
                    }
                }
                if get("invalidation")? != "passed" {
                    return Err("Missing live TLB check".into());
                }
                p.memory = true;
            }
            "m1.fault_armed" => {
                let (mode, target) = match expected.fixture {
                    "guard-fault" => (12, runtime.ok_or("Missing stack")? - 4096),
                    "write-protect-fault" => (13, 0xffffffff80000000),
                    "execute-disable-fault" => (14, expected.fault_rip.ok_or("Missing NX symbol")?),
                    "emergency-stack" => (15, runtime.ok_or("Missing stack")?),
                    _ => return Err("Unexpected fault injection".into()),
                };
                if !p.memory
                    || armed.is_some()
                    || stage != "memory"
                    || outcome != "ok"
                    || number("mode")? != mode
                    || hex("target")? != target
                    || Some(hex("site")?) != expected.fault_rip
                {
                    return Err("Incorrect armed protection fault".into());
                }
                armed = Some(target);
            }
            "m1.acpi_ready" => {
                let devices = [hex("apic")?, hex("ioapic")?, hex("hpet")?];
                for (i, &d) in devices.iter().enumerate() {
                    if d == 0
                        || d % 4096 != 0
                        || d.checked_add(4096).is_none_or(|e| e > 1u64 << 32)
                        || devices[..i].contains(&d)
                        || physical_regions
                            .iter()
                            .any(|&(b, n, k)| k != 1 && d < b + n && b < d + 4096)
                    {
                        return Err("Unsafe MMIO discovery".into());
                    }
                }
                if !p.memory
                    || discovery
                    || stage != "timer"
                    || outcome != "ok"
                    || get("checksums")? != "true"
                    || get("ranges")? != "true"
                    || number("tables")? == 0
                    || number("tables")? > 128
                    || hex("rsdp")? == 0
                    || hex("xsdt")? == 0
                    || hex("apic")? == hex("hpet")?
                    || hex("apic")? % 4096 != 0
                    || hex("hpet")? % 4096 != 0
                {
                    return Err("Invalid ACPI discovery".into());
                }
                discovery = true;
            }
            "m1.timer_calibrated" => {
                let elapsed = number("apic_elapsed")? as u64;
                let ticks = number("hpet_ticks")? as u64;
                let fs = number("period_fs")? as u64;
                if !discovery
                    || calibrated.is_some()
                    || timer_ready
                    || stage != "timer"
                    || outcome != "ok"
                    || get("cache")? != "uc"
                    || number("divide")? != 16
                    || fs == 0
                    || fs > 100_000_000
                    || elapsed == 0
                    || elapsed >= u32::MAX as u64
                {
                    return Err("Invalid timer calibration".into());
                }
                let ns = ticks as u128 * fs as u128 / 1_000_000;
                if !(10_000_000..=100_000_000).contains(&ns) {
                    return Err("Invalid calibration window".into());
                }
                let initial = elapsed as u128 * 10_000_000 / ns;
                let resolution = ns * initial / elapsed as u128;
                if initial == 0
                    || initial > u32::MAX as u128
                    || number("initial")? as u128 != initial
                    || number("resolution_ns")? as u128 != resolution
                    || resolution == 0
                {
                    return Err("Incorrect timer conversion".into());
                }
                calibrated = Some(resolution as u64);
            }
            "m1.timer_ready" => {
                if calibrated.is_none()
                    || timer_ready
                    || stage != "timer"
                    || outcome != "ok"
                    || number("vector")? != 64
                    || number("spurious")? != 255
                    || number("ticks")? != 0
                    || number("uptime_ns")? != 0
                    || Some(number("resolution_ns")? as u64) != calibrated
                    || get("epoch")? != "kernel"
                    || get("pic")? != "masked"
                    || get("lvt")? != "timer-only"
                {
                    return Err("Invalid timer epoch".into());
                }
                timer_ready = true;
            }
            "m1.clock_sample" => {
                let ticks = number("ticks")? as u64;
                let ns = number("uptime_ns")? as u64;
                if !timer_ready
                    || p.timer
                    || samples >= 100
                    || stage != "timer"
                    || outcome != "ok"
                    || number("index")? != samples
                    || ticks <= last_ticks
                    || ns <= last_ns
                    || ticks.checked_mul(calibrated.unwrap()) != Some(ns)
                    || get("continuation")? != "true"
                    || get("registers")? != "true"
                {
                    return Err("Invalid monotonic timer sample".into());
                }
                samples += 1;
                last_ticks = ticks;
                last_ns = ns;
            }
            "m1.timer_masked" => {
                let before = number("before")? as u64;
                let after = number("after")? as u64;
                let resumed = number("resumed")? as u64;
                if expected.fixture != "timer-masked"
                    || samples != 100
                    || masked
                    || p.timer
                    || stage != "timer"
                    || outcome != "ok"
                    || before < last_ticks
                    || before != after
                    || resumed <= after
                    || number("hpet_ticks")? == 0
                    || get("responsive")? != "true"
                {
                    return Err("Invalid timer masking interval".into());
                }
                last_ticks = resumed;
                masked = true;
            }
            "m1.timer_progress" => {
                let ticks = number("ticks")? as u64;
                let ns = number("uptime_ns")? as u64;
                if !timer_ready
                    || p.timer
                    || samples != 100
                    || ticks < 100
                    || ticks < last_ticks
                    || ns < last_ns
                    || ticks.checked_mul(calibrated.unwrap()) != Some(ns)
                    || stage != "timer"
                    || outcome != "ok"
                    || number("reads")? != 100
                    || number("changes")? != 100
                    || number("halts")? != 100
                    || get("registers")? != "true"
                    || (expected.fixture == "timer-masked" && !masked)
                {
                    return Err("Missing timer progress/idle evidence".into());
                }
                number("spurious")?;
                p.timer = true;
            }
            "m1.platform_rejected" => {
                let want = match expected.fixture {
                    "acpi-corrupt" => ("acpi", "checksum"),
                    "timer-unsupported" => ("hpet", "missing"),
                    "timer-calibration" => ("timer", "calibration"),
                    _ => return Err("Unexpected platform rejection".into()),
                };
                if !p.memory
                    || timer_ready
                    || stage != "timer"
                    || outcome != "rejected"
                    || (get("field")?, get("reason")?) != want
                {
                    return Err("Wrong platform rejection".into());
                }
                p.rejected = true;
            }
            "m1.exception" | "m1.panic" => {
                let rip = expected.fault_rip.ok_or("Unexpected fatal diagnostic")?;
                let panic = expected.fixture == "panic";
                let double_fault = expected.fixture == "emergency-stack";
                if !p.memory
                    || outcome != "fatal"
                    || (!double_fault && hex("rip")? != rip)
                    || event != if panic { "m1.panic" } else { "m1.exception" }
                    || stage != if panic { "panic" } else { "exceptions" }
                    || (expected.fixture == "timer-exception" && !p.timer)
                {
                    return Err("Wrong fatal context or symbolized RIP".into());
                }
                let rsp = hex("rsp")?;
                let base = runtime.ok_or("Missing runtime stack")?;
                if if double_fault {
                    rsp != base
                } else {
                    rsp < base || rsp >= base + 65536
                } {
                    return Err("Fault outside owned runtime stack".into());
                }
                if panic {
                    if get("vector")? != "none"
                        || get("reason")? != "panic"
                        || number("location_line")? == 0
                        || hex("error")? != 0
                        || hex("address")? != 0
                    {
                        return Err("Wrong panic context".into());
                    }
                } else {
                    let (vector, err, address) = match expected.fixture {
                        "exception-page-fault" => (14, 0, 0x400000000000),
                        "guard-fault" => (14, 0, armed.ok_or("Unarmed guard")?),
                        "write-protect-fault" => (14, 3, armed.ok_or("Unarmed WP")?),
                        "execute-disable-fault" => (14, 17, armed.ok_or("Unarmed NX")?),
                        "emergency-stack" => {
                            armed.ok_or("Unarmed double fault")?;
                            (8, 0, 0)
                        }
                        _ => (6, 0, 0),
                    };
                    if number("vector")? != vector
                        || hex("error")? != err
                        || hex("address")? != address
                        || hex("cs")? != 8
                        || (hex("flags")? & 0x200 != 0) != (expected.fixture == "timer-exception")
                    {
                        return Err("Wrong exception vector/error/address/privilege".into());
                    }
                    let frame = hex("frame")?;
                    let base = if double_fault {
                        double.ok_or("Missing IST")?
                    } else {
                        base
                    };
                    if frame < base
                        || frame.checked_add(56).is_none_or(|end| end > base + 65536)
                        || frame % 8 != 0
                    {
                        return Err("Invalid exception frame range".into());
                    }
                }
                p.fatal = true;
            }
            "m1.boot_complete" => {
                if !p.memory
                    || !p.timer
                    || p.boot_complete
                    || expected.fault_rip.is_some()
                    || stage != "boot"
                    || outcome != "ok"
                    || get("reclaimed_bytes")? != "0"
                {
                    return Err("Unexpected boot completion".into());
                }
                p.boot_complete = true;
            }
            "m1.runtime_memory"
            | "m1.runtime_walk"
            | "m1.runtime_space"
            | "m1.runtime_entry"
            | "m1.runtime_stack"
            | "m1.runtime_alias"
            | "m1.runtime_settlement"
            | "m1.runtime_memory_ready" => {
                if !p.memory || stage != "runtime-memory" || outcome != "ok" || p.boot_complete {
                    return Err("Unexpected runtime memory record".into());
                }
                if event != "m1.runtime_memory" && expected.fixture != "owned-memory" {
                    return Err("Runtime fixture disabled".into());
                }
            }
            "m1.loader_image" | "m1.loader_page" | "m1.loader_table" | "m1.loader_binding"
            | "m1.loader_rejected" | "m1.loader_ready" => {
                if !p.memory
                    || stage != "static-loader"
                    || outcome != "ok"
                    || p.boot_complete
                    || expected.fixture != "static-loader"
                {
                    return Err("Unexpected loader record".into());
                }
            }
            "m1.user_oom_injection"
            | "m1.user_runtime"
            | "m1.user_inventory"
            | "m1.user_fault"
            | "m1.user_bad_return"
            | "m1.user_call"
            | "m1.user_irq"
            | "m1.user_start"
            | "m1.user_end"
            | "m1.user_log"
            | "m1.user_settlement"
            | "m1.user_ready" => {
                if !p.memory
                    || p.boot_complete
                    || stage != "user-boundary"
                    || outcome != "ok"
                    || !expected.fixture.starts_with("user-boundary-")
                {
                    return Err("Unexpected user-boundary record".into());
                }
            }
            "m1.ready" => {
                if !p.boot_complete
                    || stage != "ready"
                    || outcome != "ok"
                    || get("baseline")? != "boot-memory-timer"
                    || get("reclaimed_bytes")? != "0"
                {
                    return Err("Unexpected final readiness".into());
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
    if expected.fixture == "static-loader" {
        let rows = p
            .records
            .iter()
            .filter(|r| r["event"] == "m1.loader_ready")
            .collect::<Vec<_>>();
        if rows.len() != 1
            || rows[0]["fields"]["images"] != "3"
            || rows[0]["fields"]["settled"] != "true"
            || rows[0]["fields"]["user_execution"] != "false"
        {
            return Err("Missing loader completion".into());
        }
    }
    if expected.fixture.starts_with("user-boundary-") {
        let rows = p
            .records
            .iter()
            .filter(|r| r["event"] == "m1.user_ready")
            .collect::<Vec<_>>();
        if rows.len() != 1
            || rows[0]["fields"]["runs"] != "9"
            || rows[0]["fields"]["settled"] != "true"
            || rows[0]["fields"]["copy_recovery"] != "true"
            || rows[0]["fields"]["user_execution"] != "true"
        {
            return Err("Missing user boundary completion".into());
        }
    }
    if expected.fixture == "owned-memory" {
        let final_rows = p
            .records
            .iter()
            .filter(|r| r["event"] == "m1.runtime_memory_ready")
            .collect::<Vec<_>>();
        if final_rows.len() != 1
            || final_rows[0]["fields"]["cycles"] != "1000"
            || final_rows[0]["fields"]["settled"] != "true"
            || final_rows[0]["fields"]["user_execution"] != "false"
        {
            return Err("Missing/invalid runtime memory completion".into());
        }
    }
    if !p.ready() {
        return Err("Missing complete guest outcome".into());
    }
    if expected.fault_rip.is_some() && !p.fatal {
        return Err("Missing expected fatal outcome".into());
    }
    let exit = if expected.fault_rip.is_some() {
        37
    } else if expected.rejection.is_some()
        || ["acpi-corrupt", "timer-unsupported", "timer-calibration"].contains(&expected.fixture)
    {
        35
    } else {
        33
    };
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
            "{event} version=5 build=abc session=def session_origin=host seq={seq} stage={stage} outcome={outcome} emergency=false {detail}\n"
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
                "index=0 base=0x10000 bytes=0x10000 kind=0",
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
            + &line(
                "m1.exceptions_ready",
                5,
                "exceptions",
                "ok",
                "idt_entries=256 runtime_base=0xffffffff80010000 double_base=0xffffffff80020000 nmi_base=0xffffffff80030000 stack_bytes=65536 double_ist=1 nmi_ist=2 guards=false if=false",
            )
            + &line(
                "m1.memory_switch",
                6,
                "memory",
                "ok",
                "before=0x1000 after=0x3000 table_pages=10 table_capacity=2304 physical_limit=0x100000000 eligible_frames=16 mapped_direct_frames=16 kernel_pages=100 reclaimed_bytes=0",
            )
            + &(0..8)
                .map(|i| {
                    line(
                        "m1.frame_test",
                        7 + i,
                        "memory",
                        "ok",
                        &format!(
                            "index={i} physical={:#x} zeroed=true unique=true reserved=false",
                            0x10000 + i * 4096
                        ),
                    )
                })
                .collect::<String>()
            + &line(
                "m1.memory_ready",
                15,
                "memory",
                "ok",
                "cr3=0x3000 wp=true nx=true supervisor=true image_audited=true direct_audited=true guards=6 aliases=0 batch=8 reuse_zero=true invalid_free=rejected double_free=rejected conflict=rejected invalidation=passed exhaustion=not-requested allocated=0 test_quota=0",
            )
            + &line(
                "m1.acpi_ready",
                16,
                "timer",
                "ok",
                "rsdp=0x1000 xsdt=0x2000 tables=2 apic=0xfee00000 ioapic=0xfec00000 hpet=0xfed00000 checksums=true ranges=true",
            )
            + &line(
                "m1.timer_calibrated",
                17,
                "timer",
                "ok",
                "apic_elapsed=1250000 hpet_ticks=2000000 period_fs=10000000 divide=16 initial=625000 resolution_ns=10000000 cache=uc",
            )
            + &line(
                "m1.timer_ready",
                18,
                "timer",
                "ok",
                "vector=64 spurious=255 ticks=0 uptime_ns=0 resolution_ns=10000000 epoch=kernel pic=masked lvt=timer-only",
            )
            + &(0..100)
                .map(|i| {
                    line(
                        "m1.clock_sample",
                        19 + i,
                        "timer",
                        "ok",
                        &format!(
                            "index={i} ticks={} uptime_ns={} continuation=true registers=true",
                            i + 1,
                            (i + 1) * 10000000
                        ),
                    )
                })
                .collect::<String>()
            + &line(
                "m1.timer_progress",
                119,
                "timer",
                "ok",
                "ticks=100 uptime_ns=1000000000 reads=100 changes=100 halts=100 registers=true spurious=0",
            )
            + &line("m1.boot_complete", 120, "boot", "ok", "reclaimed_bytes=0")
            + &line(
                "m1.ready",
                121,
                "ready",
                "ok",
                "baseline=boot-memory-timer reclaimed_bytes=0",
            )
    }
    #[test]
    fn rejects_false_or_incomplete_success() {
        let e = Expected {
            build: "abc",
            session: "def",
            fixture: "normal",
            rejection: None,
            fault_rip: None,
        };
        let good = good();
        assert!(assertions(good.as_bytes(), e, Some(33), false).is_ok());
        for bad in [
            good.replace("session=def", "session=other"),
            good.replace("build=abc", "build=other"),
            good.replace("seq=2", "seq=1"),
            good.replace("version=5", "version=1"),
            good.lines()
                .filter(|l| !l.starts_with("m1.ready "))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
            good.replace("m1.boot_complete", "m1.ready"),
            good.clone()
                + &line(
                    "m1.ready",
                    122,
                    "ready",
                    "ok",
                    "baseline=boot-memory-timer reclaimed_bytes=0",
                ),
            good.clone() + &line("m1.panic", 6, "panic", "rejected", "reason=panic"),
            good.clone() + &line("m1.kernel_entry", 6, "entry", "ok", "fixture=normal"),
            good.replace("regions=1", "regions=2"),
            good.replace("retained=true", "retained=false"),
            good.replace("wp=true", "wp=false"),
            good.replace("nx=true", "nx=false"),
            good.replace("guards=6", "guards=0"),
            good.replace("after=0x3000", "after=0x1000"),
            good.replace("physical=0x17000", "physical=0x16000"),
            good.replace("physical=0x17000", "physical=0x2000"),
            good.replace("allocated=0", "allocated=1"),
            good.replace("cr3=0x3000", "cr3=0x4000"),
            good.replace("resolution_ns=10000000", "resolution_ns=9999999"),
            good.replace("uptime_ns=1000000000", "uptime_ns=999999999"),
            good.replace("index=99 ticks=100", "index=99 ticks=99"),
            good.replace("epoch=kernel", "epoch=host"),
            good.replace("halts=100", "halts=99"),
            good.replace("registers=true", "registers=false"),
            good.replace("ioapic=0xfec00000", "ioapic=0x10000"),
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
            fault_rip: None,
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
    #[test]
    fn fatal_requires_exact_hardware_context_symbol_and_exit() {
        for (fixture, event, stage, detail) in [
            (
                "exception-invalid-opcode",
                "m1.exception",
                "exceptions",
                "vector=6 rip=0xffffffff80000010 error=0x0 address=0x0 rsp=0xffffffff8001ff00 cs=0x8 flags=0x2 frame=0xffffffff8001fec8",
            ),
            (
                "exception-page-fault",
                "m1.exception",
                "exceptions",
                "vector=14 rip=0xffffffff80000010 error=0x0 address=0x400000000000 rsp=0xffffffff8001ff00 cs=0x8 flags=0x2 frame=0xffffffff8001fec0",
            ),
            (
                "panic",
                "m1.panic",
                "panic",
                "vector=none rip=0xffffffff80000010 error=0x0 address=0x0 rsp=0xffffffff8001ff00 location_line=1 reason=panic",
            ),
            (
                "exception-emergency",
                "m1.exception",
                "exceptions",
                "vector=6 rip=0xffffffff80000010 error=0x0 address=0x0 rsp=0xffffffff8001ff00 cs=0x8 flags=0x2 frame=0xffffffff8001fec8",
            ),
        ] {
            let expected = Expected {
                build: "abc",
                session: "def",
                fixture,
                rejection: None,
                fault_rip: Some(0xffffffff80000010),
            };
            let prefix = good()
                .lines()
                .take(16)
                .collect::<Vec<_>>()
                .join("\n")
                .replace("fixture=normal", &format!("fixture={fixture}"))
                + "\n";
            let terminal = line(event, 16, stage, "fatal", detail);
            let terminal = if fixture == "exception-emergency" {
                terminal.replace("emergency=false", "emergency=true")
            } else {
                terminal
            };
            let log = prefix.clone() + &terminal;
            assert!(
                assertions(log.as_bytes(), expected, Some(37), false).is_ok(),
                "{log}"
            );
            for bad in [
                prefix,
                log.replace("rip=0xffffffff80000010", "rip=0xffffffff80000011"),
                log.replace("error=0x0", "error=0x1"),
                log.replace("rsp=0xffffffff8001ff00", "rsp=0x1000"),
                log.replace("outcome=fatal", "outcome=ok"),
                log.replace("seq=16", "seq=17"),
                log.replace(
                    "double_base=0xffffffff80020000",
                    "double_base=0xffffffff80010000",
                ),
                log.clone() + &line("m1.boot_complete", 7, "boot", "ok", "reclaimed_bytes=0"),
            ] {
                assert!(
                    assertions(bad.as_bytes(), expected, Some(37), false).is_err(),
                    "{bad}"
                );
            }
            for code in [None, Some(0), Some(33), Some(35)] {
                assert!(assertions(log.as_bytes(), expected, code, false).is_err());
            }
            assert!(assertions(log.as_bytes(), expected, Some(37), true).is_err());
        }
    }
    #[test]
    fn protection_and_real_double_fault_require_owned_context() {
        for (fixture, mode, target, vector, error, rsp, frame) in [
            (
                "guard-fault",
                12,
                0xffffffff8000f000u64,
                14,
                0,
                0xffffffff8001ff00u64,
                0xffffffff8001fec0u64,
            ),
            (
                "write-protect-fault",
                13,
                0xffffffff80000000,
                14,
                3,
                0xffffffff8001ff00,
                0xffffffff8001fec0,
            ),
            (
                "execute-disable-fault",
                14,
                0xffffffff80000010,
                14,
                17,
                0xffffffff8001ff00,
                0xffffffff8001fec0,
            ),
            (
                "emergency-stack",
                15,
                0xffffffff80010000,
                8,
                0,
                0xffffffff80010000,
                0xffffffff8002ffc0,
            ),
        ] {
            let e = Expected {
                build: "abc",
                session: "def",
                fixture,
                rejection: None,
                fault_rip: Some(0xffffffff80000010),
            };
            let prefix = good()
                .lines()
                .take(16)
                .collect::<Vec<_>>()
                .join("\n")
                .replace("fixture=normal", &format!("fixture={fixture}"))
                + "\n";
            let arm = line(
                "m1.fault_armed",
                16,
                "memory",
                "ok",
                &format!("mode={mode} target={target:#x} site=0xffffffff80000010"),
            );
            // #DF is an abort: the saved RIP is undefined and is not symbol asserted.
            let rip = if mode == 15 { 0 } else { 0xffffffff80000010u64 };
            let address = if mode == 15 { 0 } else { target };
            let fault = line(
                "m1.exception",
                17,
                "exceptions",
                "fatal",
                &format!(
                    "vector={vector} rip={rip:#x} error={error:#x} address={address:#x} rsp={rsp:#x} cs=0x8 flags=0x2 frame={frame:#x}"
                ),
            );
            let log = prefix.clone() + &arm + &fault;
            assert!(
                assertions(log.as_bytes(), e, Some(37), false).is_ok(),
                "{log}"
            );
            for bad in [
                (prefix + &fault).replace("seq=17", "seq=16"),
                log.replace(&format!("error={error:#x}"), "error=0xff"),
                log.replace(&format!("frame={frame:#x}"), "frame=0x1000"),
                log.replace(&format!("target={target:#x}"), "target=0x1000"),
                log.replace("site=0xffffffff80000010", "site=0xffffffff80000011"),
            ] {
                assert!(
                    assertions(bad.as_bytes(), e, Some(37), false).is_err(),
                    "{bad}"
                );
            }
            assert!(assertions(log.as_bytes(), e, Some(0), false).is_err());
        }
    }
}
