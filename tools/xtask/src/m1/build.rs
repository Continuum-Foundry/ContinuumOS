//! Kernel build identity, compilation and independent binary audits.
use super::{
    process::{capture, capture_audit},
    support::{digest, read},
};
use crate::{Result, check_pin, elf, hash, output, root, rust_version, tool, write_json};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

const TARGET: &str = "x86_64-unknown-none";

fn collect(dir: &Path, result: &mut BTreeMap<String, String>) -> Result<()> {
    for e in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        if e.file_type().map_err(|e| e.to_string())?.is_dir() {
            collect(&e.path(), result)?;
        } else {
            let path = e.path();
            let key = path
                .strip_prefix(root())
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            result.insert(key, digest(&path)?);
        }
    }
    Ok(())
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Flavor {
    Normal,
    Smoke,
    NoEntry,
    Inject,
    Exceptions,
    Memory,
    Timer,
    RuntimeMemory,
    StaticLoader,
    UserBoundary,
    Scheduler,
    Capabilities,
    Channels,
    Waits,
    Supervisor,
    Policy,
    Routes,
}
impl Flavor {
    fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Smoke => "iso-smoke",
            Self::NoEntry => "fault-before-entry",
            Self::Inject => "boot-inject",
            Self::Exceptions => "exception-test",
            Self::Memory => "memory-test",
            Self::Timer => "timer-test",
            Self::RuntimeMemory => "runtime-memory-test",
            Self::StaticLoader => "static-loader-test",
            Self::UserBoundary => "user-boundary-test",
            Self::Scheduler => "scheduler-test",
            Self::Capabilities => "capability-test",
            Self::Channels => "channel-test",
            Self::Waits => "wait-test",
            Self::Supervisor => "supervisor-test",
            Self::Policy => "policy-test",
            Self::Routes => "route-test",
        }
    }
    fn features(self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Smoke => "emulator-test",
            Self::NoEntry => "fault-before-entry",
            Self::Inject => "boot-inject",
            Self::Exceptions => "exception-test",
            Self::Memory => "memory-test",
            Self::Timer => "timer-test",
            Self::RuntimeMemory => "runtime-memory-test",
            Self::StaticLoader => "static-loader-test",
            Self::UserBoundary => "user-boundary-test",
            Self::Scheduler => "scheduler-test",
            Self::Capabilities => "capability-test",
            Self::Channels => "channel-test",
            Self::Waits => "wait-test",
            Self::Supervisor => "supervisor-test",
            Self::Policy => "policy-test",
            Self::Routes => "route-test",
        }
    }
}
fn audit_requests(bytes: &[u8], inspection: &Value) -> Result<()> {
    let words = |a: &[u64]| a.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>();
    let locate = |needle: Vec<u8>| -> Result<usize> {
        let positions = bytes
            .windows(needle.len())
            .enumerate()
            .filter(|(_, b)| *b == needle)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        if positions.len() != 1 || positions[0] % 8 != 0 {
            return Err("Expected one aligned request marker".into());
        }
        Ok(positions[0])
    };
    let start = locate(words(&[
        0xf6b8f4b39de7d1ae,
        0xfab91a6940fcb9cf,
        0x785c6ed015d3e316,
        0x181e920a7852b9d9,
    ]))?;
    let end = locate(words(&[0xadc0e0531bb10d03, 0x9572709f31764c62]))?;
    if end <= start || end - start > 4096 {
        return Err("Invalid request region extent".into());
    }
    let expected = [
        (0x48dcf1cb8ad2b852, 0x63984e959a98244b),
        (0x67cf3d9d378a806f, 0xe304acdfc50c3c62),
        (0x71ba76863cc55f63, 0xb2644a48c516a487),
        (0xad97e90e83f1ed67, 0x31eb5d1c5ff23b69),
        (0x3e7e279702be32af, 0xca1c4f3bd1280cee),
        (0x9d5827dcd881dd75, 0xa3148604f6fab11b),
        (0xc5e77b6b397e7b43, 0x27637845accdcf3c),
        (0xf55038d8e2a1202f, 0x279426fcf5f59740),
        (0x95c1a0edab0944cb, 0xa4e5cb3842f7488a),
    ];
    let word = |i: usize| -> Result<u64> {
        Ok(u64::from_le_bytes(
            bytes
                .get(i..i + 8)
                .ok_or("Truncated request")?
                .try_into()
                .unwrap(),
        ))
    };
    let mut seen = [false; 9];
    let mut base = false;
    let mut at = start + 32;
    while at < end {
        let first = word(at)?;
        if first == 0 {
            at += 8;
            continue;
        }
        if first == 0xf9562b2d5c95a6c8 && word(at + 8)? == 0x6a7b384944536bdc {
            if base || word(at + 16)? != 3 {
                return Err("Invalid/duplicate base revision".into());
            }
            base = true;
            at += 24;
            continue;
        }
        if first != 0xc7b1dd30df4c8b88 || word(at + 8)? != 0x0a82e883a194f07b {
            return Err("Unknown request bytes".into());
        }
        let id = (word(at + 16)?, word(at + 24)?);
        let i = expected
            .iter()
            .position(|&v| v == id)
            .ok_or("Unexpected request ID")?;
        if seen[i] || word(at + 32)? != if i == 8 { 1 } else { 0 } || word(at + 40)? != 0 {
            return Err("Invalid request revision/response/duplicate".into());
        }
        seen[i] = true;
        if i == 8 && (word(at + 48)? != 0 || word(at + 56)? != 0 || word(at + 64)? != 0) {
            return Err("Paging must be four-level only".into());
        }
        at += if i == 8 { 72 } else { 48 };
    }
    if at != end || !base || seen.iter().any(|&v| !v) {
        return Err("Missing required retained requests".into());
    }
    if !inspection["segments"]
        .as_array()
        .ok_or("Missing segments")?
        .iter()
        .any(|s| {
            s["flags"] == 6
                && s["offset"].as_u64().is_some_and(|o| {
                    start as u64 >= o
                        && (end + 16) as u64 <= o + s["file_size"].as_u64().unwrap_or(0)
                })
        })
    {
        return Err("Requests are not in a file-backed RW load segment".into());
    }
    Ok(())
}
// Cross-check independent LLVM symbols against ELF program headers. Linker
// orphans following __image_end must never escape the explicit kernel range.
fn audit_image_bounds(text: &str, inspection: &Value) -> Result<Value> {
    let mut values = BTreeMap::new();
    let mut name = None;
    for line in text.lines().map(str::trim) {
        if let Some(field) = line.strip_prefix("Name: ") {
            name = match field.split_whitespace().next() {
                Some(n @ ("__image_start" | "__image_end")) => Some(n),
                _ => None,
            };
        }
        if let Some(value) = line.strip_prefix("Value: ") {
            if let Some(n) = name.take() {
                let value = u64::from_str_radix(
                    value.strip_prefix("0x").ok_or("Invalid image symbol")?,
                    16,
                )
                .map_err(|_| "Invalid image symbol")?;
                if values.insert(n, value).is_some() {
                    return Err("Duplicate image boundary symbol".into());
                }
            }
        }
    }
    let start = *values.get("__image_start").ok_or("Missing image start")?;
    let end = *values.get("__image_end").ok_or("Missing image end")?;
    let mut minimum = u64::MAX;
    let mut maximum = 0;
    for segment in inspection["segments"]
        .as_array()
        .ok_or("Missing load segments")?
    {
        let address = u64::from_str_radix(
            segment["address"]
                .as_str()
                .ok_or("Missing load address")?
                .strip_prefix("0x")
                .ok_or("Invalid load address")?,
            16,
        )
        .map_err(|_| "Invalid load address")?;
        let size = segment["memory_size"]
            .as_u64()
            .ok_or("Missing load memory size")?;
        if size != 0 {
            minimum = minimum.min(address);
            maximum = maximum.max(address.checked_add(size).ok_or("Load memory overflow")?);
        }
    }
    if start != minimum || end <= start || end < maximum {
        return Err(
            "Kernel image symbols do not cover all PT_LOAD memory including BSS/orphans".into(),
        );
    }
    Ok(
        json!({"start":format!("{start:#x}"),"end":format!("{end:#x}"),"load_memory_end":format!("{maximum:#x}"),"all_load_memory_including_bss":true}),
    )
}
pub(super) fn audit_disassembly(text: &str) -> Result<()> {
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        // LLVM AT&T register spellings plus x87/implicit vector instructions.
        let mnemonic = lower
            .split_once(':')
            .map(|(_, v)| v.split_whitespace().next().unwrap_or(""))
            .unwrap_or("");
        if [
            "%xmm", "%ymm", "%zmm", "%mm", "%st", "%k0", "%k1", "%k2", "%k3", "%k4", "%k5", "%k6",
            "%k7",
        ]
        .iter()
        .any(|r| lower.contains(r))
            || mnemonic.starts_with('f')
            || mnemonic.starts_with('v')
            || ["emms", "ldmxcsr", "stmxcsr", "xsave", "xrstor"]
                .iter()
                .any(|m| mnemonic.starts_with(m))
        {
            return Err(format!("Unexpected FP/SIMD instruction: {line}"));
        }
    }
    Ok(())
}
pub(super) fn source_inputs() -> Result<BTreeMap<String, String>> {
    let mut sources = BTreeMap::new();
    for p in [
        "kernel",
        "tools/xtask",
        "tools/windows",
        "experiments/user-images",
    ] {
        collect(&root().join(p), &mut sources)?;
    }
    for p in [
        "contracts/runtime-v0.json",
        "contracts/policy-v0.json",
        "contracts/routes-v0.json",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        "config/m1-inputs.lock.json",
        "config/m2-profiles.json",
        "config/m3-profiles.json",
        "fixtures/m2/runner-input.txt",
        "tools/Enter-Dev.ps1",
        "tools/Test-M0.ps1",
        "tools/Probe-Dependencies.ps1",
        "tools/Test-M1Prerequisites.ps1",
        "tools/Test-P1-03.ps1",
        "tools/Test-P1-04.ps1",
        "tools/Test-P1-05.ps1",
        "tools/Test-P1-06.ps1",
        "tools/Test-P1-07.ps1",
    ] {
        sources.insert(p.into(), digest(&root().join(p))?);
    }
    Ok(sources)
}
pub(super) fn build(dir: &Path, flavor: Flavor) -> Result<Value> {
    check_pin()?;
    let user_descriptors = if matches!(
        flavor,
        Flavor::StaticLoader
            | Flavor::UserBoundary
            | Flavor::Scheduler
            | Flavor::Capabilities
            | Flavor::Channels
            | Flavor::Waits
            | Flavor::Supervisor
            | Flavor::Policy
            | Flavor::Routes
    ) {
        Some(super::user_images::build(
            dir,
            flavor == Flavor::UserBoundary,
            matches!(
                flavor,
                Flavor::Scheduler
                    | Flavor::Capabilities
                    | Flavor::Channels
                    | Flavor::Waits
                    | Flavor::Supervisor
                    | Flavor::Policy
                    | Flavor::Routes
            ),
            flavor == Flavor::Capabilities,
            flavor == Flavor::Channels,
            flavor == Flavor::Waits,
            flavor == Flavor::Supervisor,
            flavor == Flavor::Policy,
            flavor == Flavor::Routes,
        )?)
    } else {
        None
    };
    let sources = source_inputs()?;
    let compiler = rust_version()?;
    let sysroot = output(Command::new(tool("rustc")?).args(["--print", "sysroot"]))?;
    let llvm = Path::new(&sysroot).join("lib/rustlib/x86_64-pc-windows-msvc/bin");
    let compiler_hash = digest(&Path::new(&sysroot).join("bin/rustc.exe"))?;
    let flags = [
        "code-model=kernel",
        "relocation-model=static",
        "link-arg=--build-id=none",
        "link-arg=-Tkernel/link.ld",
        "remap-path-prefix=/continuum",
    ];
    let identity = json!({"schema_version":1,"sources":sources,"compiler":compiler,"rustc_sha256":compiler_hash,"profile":"kernel-dev","target":TARGET,"flavor":flavor.name(),"flags":flags,"user_images":if user_descriptors.is_some(){Some(digest(&dir.join("user-images.json"))?)}else{None}});
    let id = hash(&serde_json::to_vec(&identity).map_err(|e| e.to_string())?);
    write_json(&dir.join("build-inputs.json"), &identity)?;
    let mut c = Command::new(tool("cargo")?);
    c.current_dir(root())
        .args([
            "rustc",
            "--locked",
            "-p",
            "continuum-kernel",
            "--bin",
            "continuum-kernel",
            "--target",
            TARGET,
            "--profile",
            "kernel-dev",
            "--target-dir",
        ])
        .arg(dir.join("cargo"));
    if !flavor.features().is_empty() {
        c.args(["--features", flavor.features()]);
    }
    if let Some(path) = user_descriptors {
        c.env("CONTINUUM_TRUSTED_IMAGES", path);
        c.env("CONTINUUM_RUNTIME_IDS", dir.join("runtime-ids.rs"));
    }
    c.args([
        "--",
        "-C",
        "code-model=kernel",
        "-C",
        "relocation-model=static",
        "-C",
        "link-arg=--build-id=none",
        "-C",
    ])
    .arg(format!(
        "link-arg=-T{}",
        root().join("kernel/link.ld").display()
    ))
    .arg(format!(
        "--remap-path-prefix={}=/continuum",
        root().display()
    ))
    .env("CONTINUUM_BUILD_ID", &id)
    .env("RUSTC", tool("rustc")?);
    for k in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_RUSTFLAGS",
        "CARGO_TARGET_X86_64_UNKNOWN_NONE_RUSTFLAGS",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
    ] {
        c.env_remove(k);
    }
    capture(&mut c, dir, "kernel-build", 180)?;
    let bytes = read(&dir.join(format!("cargo/{TARGET}/kernel-dev/continuum-kernel")))?;
    fs::write(dir.join("kernel.elf"), &bytes).map_err(|e| e.to_string())?;
    let mut inspection = elf::inspect(&bytes, "continuum-kernel")?;
    audit_requests(&bytes, &inspection)?;
    for (name, args) in [
        (
            "llvm-readobj",
            vec![
                "--file-headers",
                "--program-headers",
                "--sections",
                "--symbols",
                "--relocations",
                "--needed-libs",
            ],
        ),
        ("llvm-nm", vec!["--undefined-only"]),
        ("llvm-objdump", vec!["--disassemble", "--no-show-raw-insn"]),
    ] {
        let mut cmd = Command::new(llvm.join(format!("{name}.exe")));
        let text = if name == "llvm-objdump" {
            capture_audit(cmd.args(args).arg(dir.join("kernel.elf")), dir, name, 30)?
        } else {
            capture(cmd.args(args).arg(dir.join("kernel.elf")), dir, name, 30)?
        };
        if name == "llvm-readobj" {
            inspection["kernel_image_bounds"] = audit_image_bounds(&text, &inspection)?;
            let mut sites = serde_json::Map::new();
            let mut symbol = None;
            for line in text.lines().map(str::trim) {
                if let Some(name) = line.strip_prefix("Name: ") {
                    symbol = name.split_whitespace().next().filter(|name| {
                        [
                            "diagnostic_invalid_opcode",
                            "diagnostic_page_fault_site",
                            "diagnostic_panic_report",
                            "diagnostic_guard_site",
                            "diagnostic_write_site",
                            "diagnostic_nx_page",
                            "diagnostic_double_site",
                        ]
                        .contains(name)
                    });
                }
                if let (Some(name), Some(value)) = (symbol, line.strip_prefix("Value: 0x")) {
                    sites.insert(
                        name.into(),
                        json!(
                            u64::from_str_radix(value, 16)
                                .map_err(|_| "Invalid diagnostic symbol")?
                        ),
                    );
                    symbol = None;
                }
            }
            if sites.len() != 7 {
                return Err("Missing diagnostic symbols".into());
            }
            inspection["diagnostic_sites"] = Value::Object(sites);
        }
        if name == "llvm-nm" && !text.trim().is_empty() {
            return Err("LLVM found undefined symbols".into());
        }
        if name == "llvm-objdump" {
            if matches!(
                flavor,
                Flavor::Scheduler
                    | Flavor::Capabilities
                    | Flavor::Channels
                    | Flavor::Waits
                    | Flavor::Supervisor
                    | Flavor::Policy
                    | Flavor::Routes
            ) {
                let mut symbol = "";
                let mut filtered = String::new();
                let mut saves = 0;
                let mut restores = 0;
                for line in text.lines() {
                    if line.ends_with(">:") {
                        symbol = line.split('<').nth(1).unwrap_or("").trim_end_matches(">:");
                    }
                    if symbol == "task_save_fp" && line.contains("fxsave64") {
                        saves += 1;
                        continue;
                    }
                    if symbol == "task_resume" && line.contains("fxrstor64") {
                        restores += 1;
                        continue;
                    }
                    filtered.push_str(line);
                    filtered.push('\n');
                }
                if saves != 1 || restores != 1 {
                    return Err("Missing narrow eager FP helpers".into());
                }
                audit_disassembly(&filtered)?;
            } else {
                audit_disassembly(&text)?;
            }
            inspection["returning_interrupt_entry"] = audit_interrupt_entry(&text)?;
        }
    }
    let commit = output(
        Command::new("git")
            .current_dir(root())
            .args(["rev-parse", "HEAD"]),
    )?;
    let dirty = output(
        Command::new("git")
            .current_dir(root())
            .args(["status", "--porcelain=v1"]),
    )?;
    let manifest = json!({"schema_version":1,"build_id":id,"flavor":flavor.name(),"source_commit":commit,"source_dirty":!dirty.is_empty(),"source_status":dirty,"source_identity":"build-inputs.json contains hashes of all build source inputs","kernel_sha256":hash(&bytes),"inspection":inspection,"loader":"12.9.3","binding":"limine =0.5.0; defaults off","protocol_commit":"3a0526b700e356f0eac1b71a77697b3fd1c707a3","base_revision":3,"entry_boundary":"SysV without FP/SIMD; loader stack >=64 KiB; no return; no memory reclaimed","assembly":"fatal exception/GDT/stack wrappers; see kernel/src/exception-entry.S and exception-diagnostics-runbook.md","boot_tested":false});
    write_json(&dir.join("build-manifest.json"), &manifest)?;
    Ok(manifest)
}

fn audit_interrupt_entry(text: &str) -> Result<Value> {
    let normalized = text.replace("\r\n", "\n");
    let text = normalized.as_str();
    let registers = [
        "rax", "rcx", "rdx", "rbx", "rbp", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13",
        "r14", "r15",
    ];
    for name in ["timer_vector", "spurious_vector"] {
        let marker = format!("<{name}>:\n");
        let body = text
            .split_once(&marker)
            .ok_or("Missing returning interrupt entry")?
            .1
            .split("\n\n")
            .next()
            .unwrap();
        let instructions = body
            .lines()
            .map(|l| {
                l.split_once(':')
                    .map(|(_, v)| v.split_whitespace().collect::<Vec<_>>().join(" "))
                    .ok_or_else(|| "Malformed entry disassembly".to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        if instructions.len() != 45 {
            return Err("Unexpected interrupt entry instruction count".into());
        }
        for (i, register) in registers.iter().enumerate() {
            if instructions[i] != format!("pushq %{register}")
                || instructions[43 - i] != format!("popq %{register}")
            {
                return Err("Interrupt entry does not preserve all GPRs".into());
            }
        }
        if instructions[15] != "cld"
            || instructions[16] != "movq %cr0, %rax"
            || instructions[17] != "pushq %rax"
            || instructions[18] != "orq $0x8, %rax"
            || instructions[19] != "movq %rax, %cr0"
            || instructions[20] != "leaq 0x8(%rsp), %rbx"
            || instructions[21] != "andq $-0x10, %rsp"
            || instructions[23] != "movq 0x80(%rbx), %rsi"
            || instructions[24] != "movq %rbx, %rdx"
            || !instructions[25].starts_with("callq ")
            || instructions[26] != "movq -0x8(%rbx), %rax"
            || instructions[27] != "movq %rax, %cr0"
            || instructions[28] != "movq %rbx, %rsp"
            || instructions[44] != "iretq"
            || instructions[22]
                != format!(
                    "movl ${}, %edi",
                    if name == "timer_vector" {
                        "0x40"
                    } else {
                        "0xff"
                    }
                )
        {
            return Err("Invalid interrupt ABI boundary".into());
        }
    }
    Ok(
        json!({"gprs":15,"df_cleared_before_rust":true,"sysv_alignment":16,"return":"iretq","vectors":[64,255]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_bounds_include_bss_and_orphan_load_memory() {
        let image = json!({"segments":[{"address":"0x1000","file_size":256,"memory_size":512}]});
        let symbols = "Symbol {\n Name: __image_start (1)\n Value: 0x1000\n}\nSymbol {\n Name: __image_end (2)\n Value: 0x1200\n}";
        assert!(audit_image_bounds(symbols, &image).is_ok());
        // Orphan GOT after the old marker, or BSS omitted by using file_size.
        assert!(audit_image_bounds(&symbols.replace("0x1200", "0x1160"), &image).is_err());
        assert!(
            audit_image_bounds(&symbols.replace("Value: 0x1000", "Value: 0x1001"), &image).is_err()
        );
        assert!(audit_image_bounds("", &image).is_err());
        assert!(
            audit_image_bounds(
                &format!("{symbols}\nName: __image_end\nValue: 0x1200"),
                &image
            )
            .is_err()
        );
    }
    #[test]
    fn rejects_vector_and_fp_code() {
        assert!(audit_disassembly("1000: movq %rax, %rbx\n1004: outl %eax, %dx").is_ok());
        for text in [
            "1000: movaps %xmm0, %xmm1",
            "1000: fld1",
            "1000: vzeroupper",
            "1000: ldmxcsr (%rax)",
        ] {
            assert!(audit_disassembly(text).is_err());
        }
    }
}
