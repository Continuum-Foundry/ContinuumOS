//! Minimal ISO integration; this is not a guest ELF loader or full M1 suite.
use super::{Result, check_pin, elf, hash, output, root, sanitize, tool, write_json};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const TARGET: &str = "x86_64-unknown-none";
const LIMIT: u64 = 2 * 1024 * 1024;
const DEADLINE: Duration = Duration::from_secs(30);
#[path = "m1_records.rs"]
mod records;
use records::Expected;

fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}
fn digest(path: &Path) -> Result<String> {
    Ok(hash(&read(path)?))
}
fn read_log(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err(format!("log limit exceeded: {}", path.display()));
    }
    Ok(bytes)
}
fn fresh(label: &str) -> Result<PathBuf> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let p = root()
        .join("build/m1")
        .join(format!("{label}-{time}-{}", std::process::id()));
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}
fn json_read(p: &Path) -> Result<Value> {
    serde_json::from_slice(&read(p)?).map_err(|e| e.to_string())
}
fn command_record(c: &Command) -> Value {
    json!({"program":sanitize(&c.get_program().to_string_lossy()),"arguments":c.get_args().map(|a|sanitize(&a.to_string_lossy())).collect::<Vec<_>>()})
}
fn hidden(c: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
}
// Owning guard: early error returns also kill and wait for our child. QEMU is
// launched directly, without a shell, helper, persistent service or descendants.
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}
fn logs(c: &mut Command, dir: &Path, name: &str) -> Result<(PathBuf, PathBuf)> {
    let out = dir.join(format!("{name}.stdout.log"));
    let err = dir.join(format!("{name}.stderr.log"));
    c.stdin(Stdio::null())
        .stdout(fs::File::create_new(&out).map_err(|e| e.to_string())?)
        .stderr(fs::File::create_new(&err).map_err(|e| e.to_string())?);
    hidden(c);
    write_json(
        &dir.join(format!("{name}.command.json")),
        &command_record(c),
    )?;
    Ok((out, err))
}
fn oversize(paths: &[&Path]) -> bool {
    paths
        .iter()
        .any(|p| fs::metadata(p).is_ok_and(|m| m.len() > LIMIT))
}
fn capture(c: &mut Command, dir: &Path, name: &str, seconds: u64) -> Result<String> {
    let (out, err) = logs(c, dir, name)?;
    let mut child = OwnedChild(c.spawn().map_err(|e| e.to_string())?);
    let start = Instant::now();
    let status = loop {
        if oversize(&[&out, &err]) {
            return Err(format!("{name}: output limit; logs retained"));
        }
        if let Some(status) = child.0.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        if start.elapsed() > Duration::from_secs(seconds) {
            return Err(format!("{name}: host command timeout; logs retained"));
        }
        thread::sleep(Duration::from_millis(20));
    };
    if !status.success() {
        return Err(format!(
            "{name}: exit {:?}; see {}",
            status.code(),
            dir.display()
        ));
    }
    String::from_utf8(read_log(&out)?).map_err(|e| e.to_string())
}
fn validate_file(path: &Path, expected: &Value) -> Result<()> {
    if fs::metadata(path)
        .map_err(|e| format!("Missing input {}: {e}", path.display()))?
        .len()
        != expected["bytes"].as_u64().ok_or("Invalid pinned size")?
        || digest(path)? != expected["sha256"].as_str().ok_or("Invalid pinned digest")?
    {
        return Err(format!("Input hash/size mismatch: {}", path.display()));
    }
    Ok(())
}
struct Pins {
    lock: Value,
    paths: BTreeMap<String, PathBuf>,
}
impl Pins {
    fn load() -> Result<Self> {
        let lock = json_read(&root().join("config/m1-inputs.lock.json"))?;
        Self::from_lock(lock)
    }
    fn from_lock(lock: Value) -> Result<Self> {
        if lock["schema_version"] != 1 {
            return Err("Unsupported M1 lock".into());
        }
        let mut paths = BTreeMap::new();
        for f in lock["files"].as_array().ok_or("Missing pin files")? {
            let mut path = root().join(f["path"].as_str().ok_or("Missing pin path")?);
            if let Some(key) = f["override"].as_str() {
                if let Some(value) = env::var_os(key).filter(|s| !s.is_empty()) {
                    path = value.into();
                }
            }
            validate_file(&path, f)?;
            paths.insert(f["id"].as_str().ok_or("Missing pin ID")?.into(), path);
        }
        Ok(Self { lock, paths })
    }
    fn file(&self, id: &str) -> Result<&Path> {
        self.paths
            .get(id)
            .map(PathBuf::as_path)
            .ok_or_else(|| format!("Missing pin {id}"))
    }
    fn suffix(&self, suffix: &str) -> Result<&Path> {
        self.paths
            .values()
            .find(|p| p.to_string_lossy().replace('\\', "/").ends_with(suffix))
            .map(PathBuf::as_path)
            .ok_or_else(|| format!("Missing pinned {suffix}"))
    }
}
fn preflight(dir: &Path, name: &str) -> Result<Pins> {
    // Same strict runtime-directory, hash, firmware and capability checks as doctor.
    let mut c = Command::new(tool("pwsh")?);
    c.args(["-NoProfile", "-File"])
        .arg(root().join("tools/Test-M1Prerequisites.ps1"))
        .arg("-Json");
    let text = capture(&mut c, dir, name, 120)?;
    let report: Value =
        serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
    if report["status"] != "passed" {
        return Err("M1 prerequisites blocked".into());
    }
    Pins::load()
}
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
enum Flavor {
    Normal,
    Smoke,
    NoEntry,
    Inject,
}
impl Flavor {
    fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Smoke => "iso-smoke",
            Self::NoEntry => "fault-before-entry",
            Self::Inject => "boot-inject",
        }
    }
    fn features(self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Smoke => "emulator-test",
            Self::NoEntry => "fault-before-entry",
            Self::Inject => "boot-inject",
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
fn audit_disassembly(text: &str) -> Result<()> {
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
fn build(dir: &Path, flavor: Flavor) -> Result<Value> {
    check_pin()?;
    let mut sources = BTreeMap::new();
    for p in ["kernel", "tools/xtask"] {
        collect(&root().join(p), &mut sources)?;
    }
    for p in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        "config/m1-inputs.lock.json",
        "tools/Test-M1Prerequisites.ps1",
        "tools/Test-P1-03.ps1",
    ] {
        sources.insert(p.into(), digest(&root().join(p))?);
    }
    let compiler = super::rust_version()?;
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
    let identity = json!({"schema_version":1,"sources":sources,"compiler":compiler,"rustc_sha256":compiler_hash,"profile":"kernel-dev","target":TARGET,"flavor":flavor.name(),"flags":flags});
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
        let text = capture(cmd.args(args).arg(dir.join("kernel.elf")), dir, name, 30)?;
        if name == "llvm-readobj" {
            inspection["kernel_image_bounds"] = audit_image_bounds(&text, &inspection)?;
        }
        if name == "llvm-nm" && !text.trim().is_empty() {
            return Err("LLVM found undefined symbols".into());
        }
        if name == "llvm-objdump" {
            audit_disassembly(&text)?;
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
    let manifest = json!({"schema_version":1,"build_id":id,"flavor":flavor.name(),"source_commit":commit,"source_dirty":!dirty.is_empty(),"source_status":dirty,"source_identity":"build-inputs.json contains hashes of all build source inputs","kernel_sha256":hash(&bytes),"inspection":inspection,"loader":"12.9.3","binding":"limine =0.5.0; defaults off","protocol_commit":"3a0526b700e356f0eac1b71a77697b3fd1c707a3","base_revision":3,"entry_boundary":"SysV without FP/SIMD; loader stack >=64 KiB; no return; no memory reclaimed","assembly":"cli, hlt, in/out only; see kernel/src/main.rs","boot_tested":false});
    write_json(&dir.join("build-manifest.json"), &manifest)?;
    Ok(manifest)
}
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
fn package(
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
    fs::write(
        stage.join("boot/limine/limine.conf"),
        config(missing, &session, fixture, module),
    )
    .map_err(|e| e.to_string())?;
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
    let report = json!({"schema_version":2,"build_id":manifest["build_id"],"kernel_sha256":manifest["kernel_sha256"],"host_session_id":session,"session_origin":"host; correlation only, not a security credential or reset detector","fixture":fixture,"module_fixture":module,"build_flavor":manifest["flavor"],"configuration_sha256":payload["boot/limine/limine.conf"],"iso_sha256":digest(&iso)?,"payload":payload,"uefi_optical_catalog":true,"payload_extraction_verified":true,"fault":if missing {"missing-configured-kernel"} else {fixture},"input_lock_sha256":digest(&root().join("config/m1-inputs.lock.json"))?,"firmware_code_sha256":digest(pins.file("firmware_code")?)?,"firmware_vars_template_sha256":digest(pins.file("firmware_vars")?)?,"bios_tested":false,"boot_tested":false});
    write_json(&dir.join("iso-manifest.json"), &report)?;
    Ok(report)
}

fn observe(
    c: &mut Command,
    dir: &Path,
    expected: Expected<'_>,
    test: bool,
    deadline: Duration,
) -> Result<Value> {
    let serial = dir.join("serial.log");
    // Reserve a brand-new path. QEMU opens/truncates this file for this process.
    fs::File::create_new(&serial).map_err(|e| e.to_string())?;
    let (out, err) = logs(c, dir, "qemu")?;
    let mut child = OwnedChild(c.spawn().map_err(|e| e.to_string())?);
    let pid = child.0.id();
    let start = Instant::now();
    let mut seen_at = None;
    let mut timeout = false;
    let mut error = None;
    let mut stopped = false;
    let code = loop {
        if oversize(&[&serial, &out, &err]) {
            error = Some("VM log limit exceeded".into());
            break None;
        }
        let bytes = match read_log(&serial) {
            Ok(bytes) => bytes,
            Err(e) => {
                error = Some(e);
                break None;
            }
        };
        match records::check(&bytes, expected, false) {
            Ok(progress) if progress.ready() => {
                if seen_at.is_none() {
                    seen_at = Some(Instant::now());
                }
            }
            Ok(_) => {}
            Err(e) => {
                error = Some(e);
                break None;
            }
        }
        if let Some(status) = child.0.try_wait().map_err(|e| e.to_string())? {
            break status.code();
        }
        if !test && seen_at.is_some_and(|t| t.elapsed() >= Duration::from_secs(1)) {
            stopped = true;
            break None;
        }
        if start.elapsed() >= deadline {
            timeout = true;
            break None;
        }
        thread::sleep(Duration::from_millis(20));
    };
    if child.0.try_wait().map_err(|e| e.to_string())?.is_none() {
        child.0.kill().map_err(|e| e.to_string())?;
    }
    let final_status = child.0.wait().map_err(|e| e.to_string())?;
    let bytes = match read_log(&serial) {
        Ok(bytes) => bytes,
        Err(e) => {
            error = Some(e);
            Vec::new()
        }
    };
    let checked = if let Some(e) = error {
        Err(e)
    } else if test {
        records::assertions(&bytes, expected, code, timeout)
    } else if !stopped || timeout {
        Err("Normal observation exited unexpectedly or timed out".into())
    } else {
        records::check(&bytes, expected, true).and_then(|progress| {
            if progress.ready() {
                Ok(())
            } else {
                Err("Checkpoint missing".into())
            }
        })
    };
    let report = json!({"schema_version":1,"status":if checked.is_ok(){"passed"}else{"failed"},"error":checked.err(),"build_id":expected.build,"host_session_id":expected.session,"fixture":expected.fixture,"pid":pid,"deadline_ms":deadline.as_millis(),"timed_out":timeout,"kernel_entry_observed":bytes.split(|b| *b == b'\n').any(|l| l.starts_with(b"m1.kernel_entry ")),"emulator_test_exit":if test {code}else{None},"final_process_exit":final_status.code(),"host_stopped_after_checkpoint":stopped,"cleanup_reaped":true,"elapsed_host_ms":start.elapsed().as_millis(),"guest_shutdown_claimed":false});
    if let Ok(progress) = records::check(&bytes, expected, true) {
        write_json(
            &dir.join("guest-records.json"),
            &json!({"schema_version":2,"validated":progress.validated,"complete":progress.complete,"rejected":progress.rejected,"records":progress.records}),
        )?;
    }
    write_json(&dir.join("run-result.json"), &report)?;
    Ok(report)
}
fn launch(dir: &Path, iso_dir: &Path, image: &Value, test: bool) -> Result<Value> {
    let pins = preflight(dir, "run-preflight")?;
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
    write_json(
        &dir.join("launch-manifest.json"),
        &json!({"schema_version":2,"build_id":id,"iso_sha256":image["iso_sha256"],"configuration_sha256":image["configuration_sha256"],"fixture":fixture,"host_session_id":session,"session_origin":"host; correlation only, not a security credential or reset detector","firmware_code_sha256":digest(pins.file("firmware_code")?)?,"vars_template_sha256":before,"vars_copy_before_sha256":before,"vm":pins.lock["vm"],"test_device":test,"arguments":args.iter().map(|a|sanitize(a)).collect::<Vec<_>>()}),
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
            rejection: rejection(fixture),
        },
        test,
        DEADLINE,
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
fn accepted(r: &Value) -> Result<()> {
    if r["status"] == "passed" {
        Ok(())
    } else {
        Err(format!("VM failed: {}", r["error"]))
    }
}
fn subdir(dir: &Path, name: &str) -> Result<PathBuf> {
    let p = dir.join(name);
    fs::create_dir(&p).map_err(|e| e.to_string())?;
    Ok(p)
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
#[path = "m1_cases.rs"]
mod cases;
use cases::rejection;
pub fn execute(action: &str) -> Result<()> {
    cases::execute(action)
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
    fn rejects_missing_and_changed_inputs() {
        let dir = fresh("input-tests").unwrap();
        let p = dir.join("kernel with spaces.elf");
        let pin = json!({"bytes":3,"sha256":hash(b"abc")});
        assert!(validate_file(&p, &pin).is_err());
        fs::write(&p, b"abc").unwrap();
        assert!(validate_file(&p, &pin).is_ok());
        fs::write(&p, b"abd").unwrap();
        assert!(validate_file(&p, &pin).is_err());
        fs::write(&p, b"abcd").unwrap();
        assert!(validate_file(&p, &pin).is_err());
    }
    #[test]
    fn locked_kernel_loader_and_firmware_inputs_fail_closed() {
        let lock = json_read(&root().join("config/m1-inputs.lock.json")).unwrap();
        for suffix in ["limine-binary/BOOTX64.EFI", "edk2-x86_64-code.fd"] {
            let mut missing = lock.clone();
            let f = missing["files"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|f| f["path"].as_str().unwrap().ends_with(suffix))
                .unwrap();
            f["path"] = json!("work/nonexistent-p1-02-input");
            f["override"] = Value::Null;
            assert!(Pins::from_lock(missing).is_err());
        }
        let mut altered = lock;
        altered["files"][0]["sha256"] = json!("0".repeat(64));
        altered["files"][0]["override"] = Value::Null;
        assert!(Pins::from_lock(altered).is_err());
    }
    #[test]
    fn windows_paths_are_translated_without_shell_quoting() {
        assert_eq!(
            iso_path(Path::new("C:\\space directory\\image.iso")).unwrap(),
            "/cygdrive/c/space directory/image.iso"
        );
        assert!(iso_path(Path::new("relative.iso")).is_err());
        assert!(iso_path(Path::new("\\\\server\\share")).is_err());
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
    #[test]
    fn process_success_failure_timeout_and_log_limit_are_reaped() {
        for (name, script, expected) in [
            ("exit-without-marker", "exit 33", "Missing complete"),
            (
                "marker-wrong-exit",
                "[IO.File]::WriteAllText($args[0], 'm1.kernel_entry version=2 build=abc session=def session_origin=host seq=0 stage=entry outcome=ok fixture=normal' + [char]10); exit 0",
                "Missing complete",
            ),
            ("timeout", "Start-Sleep -Seconds 10", "deadline"),
            (
                "log-limit",
                "[Console]::Write(('x' * 3000000)); Start-Sleep -Seconds 10",
                "log limit",
            ),
        ] {
            let dir = fresh(name).unwrap();
            let fixture = dir.join("fixture.ps1");
            fs::write(&fixture, script).unwrap();
            let mut c = Command::new(tool("pwsh").unwrap());
            c.args(["-NoProfile", "-File"])
                .arg(&fixture)
                .arg(dir.join("serial.log"));
            let r = observe(
                &mut c,
                &dir,
                Expected {
                    build: "abc",
                    session: "def",
                    fixture: "normal",
                    rejection: None,
                },
                true,
                Duration::from_secs(3),
            )
            .unwrap();
            assert_eq!(r["status"], "failed");
            assert_eq!(r["cleanup_reaped"], true);
            assert!(r["error"].as_str().unwrap().contains(expected), "{r}");
        }
    }
}
