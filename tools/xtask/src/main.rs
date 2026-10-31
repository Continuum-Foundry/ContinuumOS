mod accounting;
#[cfg(feature = "slint-candidate")]
mod comparison;
mod custom;
mod editor;
mod elf;
mod m1;
mod m2;
mod performance;
#[cfg(test)]
#[path = "../../../experiments/ui-link/src/arena.rs"]
mod probe_arena;
mod reload;
mod selected;
#[cfg(feature = "slint-candidate")]
mod slint_candidate;
mod text;
mod ui_fixtures;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

const GUEST: &str = "x86_64-unknown-none";
const COMPONENTS: [&str; 2] = ["kernel-link", "userland-link"];
type Result<T> = std::result::Result<T, String>;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(
        path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(value).map_err(|e| e.to_string())?
        ),
    )
    .map_err(|e| e.to_string())
}
fn tool(name: &str) -> Result<PathBuf> {
    let key = format!("CONTINUUM_{}", name.to_uppercase().replace('-', "_"));
    if let Some(path) = env::var_os(&key) {
        if !Path::new(&path).is_file() {
            return Err(format!(
                "{key} is not an existing executable; no fallback used"
            ));
        }
        return Ok(path.into());
    }
    Ok(if name == "cargo" {
        env::var_os("CARGO").unwrap_or_else(|| name.into()).into()
    } else {
        name.into()
    })
}
fn output(command: &mut Command) -> Result<String> {
    let result = command
        .output()
        .map_err(|e| format!("Could not execute {command:?}: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "{command:?} failed: {}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&result.stdout).trim().to_owned())
}
fn sanitize(text: &str) -> String {
    let mut text = text.replace(&root().to_string_lossy().to_string(), "${WORKSPACE}");
    if let Ok(home) = env::var("USERPROFILE") {
        text = text.replace(&home, "${USERPROFILE}");
    }
    text
}
fn rust_version() -> Result<String> {
    output(Command::new(tool("rustc")?).arg("-vV"))
}
fn check_pin() -> Result<()> {
    let config =
        fs::read_to_string(root().join("rust-toolchain.toml")).map_err(|e| e.to_string())?;
    let actual = rust_version()?;
    let pin = config
        .lines()
        .find_map(|l| l.trim().strip_prefix("channel = "))
        .ok_or("Missing exact toolchain pin")?
        .trim_matches('"');
    if !actual.lines().any(|l| l == format!("release: {pin}")) {
        return Err(format!("Rust version differs from pin {pin}: {actual}"));
    }
    if !actual.contains("host: x86_64-pc-windows-msvc") {
        return Err("Phase 0 currently validates Windows MSVC hosts only".into());
    }
    Ok(())
}
fn doctor(json_output: bool, m1: bool) -> Result<()> {
    let mut command = Command::new(tool("pwsh")?);
    command
        .args(["-NoProfile", "-File"])
        .arg(root().join("tools/Inventory.ps1"));
    if json_output {
        command.arg("-Json");
    }
    if m1 {
        command.arg("-M1");
    }
    let status = command.status().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("Required prerequisites are missing/blocked; see doctor output".into())
    }
}
fn build_into(directory: &Path) -> Result<Value> {
    check_pin()?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let mut artifacts = Vec::new();
    for (component, model) in [("kernel-link", "kernel"), ("userland-link", "small")] {
        let script = root().join(format!("experiments/{component}/link.ld"));
        let mut command = Command::new(tool("cargo")?);
        command
            .current_dir(root())
            .args([
                "rustc",
                "--locked",
                "--package",
                component,
                "--target",
                GUEST,
                "--profile",
                "feasibility",
                "--target-dir",
            ])
            .arg(directory.join("cargo"))
            .arg("--")
            .args([
                "-C",
                &format!("code-model={model}"),
                "-C",
                "relocation-model=static",
                "-C",
                "link-arg=--build-id=none",
                "-C",
            ])
            .arg(format!("link-arg=-T{}", script.display()))
            .arg(format!(
                "--remap-path-prefix={}=/continuum",
                root().display()
            ))
            .env("RUSTC", tool("rustc")?)
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS");
        let result = command.output().map_err(|e| e.to_string())?;
        let log = format!(
            "command: {}\n{}{}",
            sanitize(&format!("{command:?}")),
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        fs::write(directory.join(format!("{component}.log")), sanitize(&log))
            .map_err(|e| e.to_string())?;
        if !result.status.success() {
            return Err(format!(
                "{component} failed; log retained at {}",
                directory.display()
            ));
        }
        let bytes = fs::read(directory.join(format!("cargo/{GUEST}/feasibility/{component}")))
            .map_err(|e| e.to_string())?;
        let inspection = elf::inspect(&bytes, component)?;
        fs::write(directory.join(format!("{component}.elf")), &bytes).map_err(|e| e.to_string())?;
        artifacts.push(json!({"component":component,"sha256":hash(&bytes),"bytes":bytes.len(),"code_model":model,"inspection":inspection}));
    }
    let sources = [
        "Cargo.lock",
        "Cargo.toml",
        ".cargo/config.toml",
        "rust-toolchain.toml",
        "tools/xtask/Cargo.toml",
        "tools/xtask/src/main.rs",
        "tools/xtask/src/elf.rs",
        "tools/xtask/src/m2.rs",
        "config/m2-profiles.json",
        "fixtures/m2/runner-input.txt",
        "tools/xtask/src/m1.rs",
        "tools/xtask/src/m1/build.rs",
        "tools/xtask/src/m1/cases.rs",
        "tools/xtask/src/m1/inputs.rs",
        "tools/xtask/src/m1/launch.rs",
        "tools/xtask/src/m1/observe.rs",
        "tools/xtask/src/m1/package.rs",
        "tools/xtask/src/m1/process.rs",
        "tools/xtask/src/m1/records.rs",
        "tools/xtask/src/m1/support.rs",
        "kernel/Cargo.toml",
        "experiments/kernel-link/Cargo.toml",
        "experiments/kernel-link/src/main.rs",
        "experiments/kernel-link/link.ld",
        "experiments/userland-link/Cargo.toml",
        "experiments/userland-link/src/main.rs",
        "experiments/userland-link/link.ld",
    ];
    let inputs=sources.into_iter().map(|p|Ok(json!({"path":p,"sha256":hash(&fs::read(root().join(p)).map_err(|e|e.to_string())?)}))).collect::<Result<Vec<_>>>()?;
    let report = json!({"schema_version":1,"rustc":rust_version()?,"target":GUEST,"inputs":inputs,"artifacts":artifacts,"boot_tested":false,"runtime_tested":false});
    write_json(&directory.join("build-manifest.json"), &report)?;
    Ok(report)
}
fn fresh_directory(label: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let path = root().join("build/m0").join(format!("{label}-{stamp}"));
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path)
}
fn test_m0() -> Result<()> {
    let run = fresh_directory("verification")?;
    let a = build_into(&run.join("clean-a"))?;
    let b = build_into(&run.join("clean b with spaces"))?;
    for index in 0..2 {
        if a["artifacts"][index]["sha256"] != b["artifacts"][index]["sha256"] {
            return Err(format!(
                "Clean outputs differ; both retained at {}",
                run.display()
            ));
        }
    }
    // LLVM is independent corroboration of the narrow in-process ELF validator.
    let sysroot = output(Command::new(tool("rustc")?).args(["--print", "sysroot"]))?;
    let llvm = Path::new(&sysroot).join("lib/rustlib/x86_64-pc-windows-msvc/bin");
    for component in COMPONENTS {
        let image = run.join("clean-a").join(format!("{component}.elf"));
        for (name, flags) in [
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
        ] {
            let key = format!("CONTINUUM_{}", name.to_uppercase().replace('-', "_"));
            let executable = if env::var_os(&key).is_some() {
                tool(name)?
            } else {
                llvm.join(format!("{name}.exe"))
            };
            let text = output(Command::new(executable).args(flags).arg(&image))?;
            fs::write(run.join(format!("{component}-{name}.txt")), sanitize(&text))
                .map_err(|e| e.to_string())?;
            if name == "llvm-nm" && !text.is_empty() {
                return Err("LLVM reports undefined symbols".into());
            }
        }
    }
    let report = json!({"schema_version":1,"status":"passed","two_clean_builds_identical":true,"output_path_with_spaces":true,"independent_llvm_inspection":true,"build":a,"boot_tested":false});
    write_json(&run.join("m0-results.json"), &report)?;
    println!("M0 compile/link checks passed. Evidence: {}", run.display());
    Ok(())
}
fn run() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        #[cfg(feature = "slint-candidate")]
        ["ui-compare", candidate, trace, repetition, directory] => comparison::measure(candidate, trace, repetition, Path::new(directory)),
        #[cfg(feature = "slint-candidate")]
        ["ui-reload", candidate, replacement, directory] => comparison::reload(candidate, Path::new(replacement), Path::new(directory)),
        #[cfg(feature = "slint-candidate")]
        ["ui-behavior-child", candidate, snapshot, mode, directory] => comparison::behavior_child(candidate, Path::new(snapshot), mode, Path::new(directory)),
        ["test", "--suite", "ui-fixtures"] => ui_fixtures::execute(),
        ["test", "--suite", "editor-model"] => editor::execute(),
        ["test", "--suite", "text"] => text::execute(),
        ["test", "--suite", "custom-ui"] => custom::execute(),
        ["test", "--suite", "selected-ui"] => selected::execute(),
        ["test", "--suite", "resource-reload"] => reload::execute(),
        ["test", "--suite", "ui-damage"] => performance::damage(),
        ["test", "--suite", "ui-stability"] => performance::stability(),
        ["ui-performance", trace, repetition, directory] => performance::measure(trace, repetition, Path::new(directory)),
        ["ui-focused", mode, directory] => performance::focused(mode, Path::new(directory)),
        ["resource-template", path] => reload::template(Path::new(path)),
        ["resource-build", input, output] => reload::build(Path::new(input), Path::new(output)),
        ["custom-host"] => custom::host(),
        #[cfg(feature = "slint-candidate")]
        ["slint-host"] => custom::host_with(custom::Candidate::Slint),
        #[cfg(feature = "slint-candidate")]
        ["test", "--suite", "slint-ui"] => slint_candidate::execute(),
        #[cfg(feature = "slint-candidate")]
        ["slint-native-text-probe"] => slint_candidate::native_text_probe(),
        #[cfg(feature = "slint-candidate")]
        ["slint-behavior"] => slint_candidate::behavior(),
        ["hosted", "--config", config, "--profile", profile, "--scale", scale, "--case", case] => m2::execute(config, profile, scale, case),
        ["hosted-probe", case, input] => m2::probe(case, input),
        ["doctor"] => doctor(false, false),
        ["doctor", "--json"] => doctor(true, false),
        ["doctor", "--profile", "m1"] => doctor(false, true),
        ["doctor", "--profile", "m1", "--json"] => doctor(true, true),
        ["build", "--profile", "feasibility"] => {
            let directory = fresh_directory("build")?;
            build_into(&directory)?;
            println!("Linked probes (not boot-tested): {}", directory.display());
            Ok(())
        }
        ["test", "--suite", "m0"] => test_m0(),
        ["test", "--suite", "m1"] => m1::execute("suite"),
        ["build", "--profile", "dev"] => m1::execute("build"),
        ["iso", "--profile", "m1"] => m1::execute("iso"),
        ["run", "--profile", "vm-baseline"] => m1::execute("run"),
        ["test", "--suite", "m1", "--case", "iso-smoke"] => m1::execute("iso-smoke"),
        ["test", "--suite", "m1", "--case", "iso-negative"] => m1::execute("iso-negative"),
        ["test", "--suite", "m1", "--case", "boot-metadata"] => m1::execute("boot-metadata"),
        ["test", "--suite", "m1", "--case", "boot-reject"] => m1::execute("boot-reject"),
        ["test", "--suite", "m1", "--case", case @ ("exception-invalid-opcode" | "exception-page-fault" | "panic" | "exception-emergency")] => m1::execute(case),
        ["test", "--suite", "m1", "--case", case @ ("memory-ownership" | "memory-exhaustion" | "guard-fault" | "write-protect-fault" | "execute-disable-fault" | "emergency-stack" | "owned-memory" | "static-loader" | "user-boundary" | "scheduler" | "capabilities" | "channels" | "waits" | "supervisor" | "policy")] => m1::execute(case),
        ["test", "--suite", "m1", "--case", case @ ("timer-progress" | "timer-masked" | "clock-read" | "idle-wakeup" | "timer-exception" | "acpi-corrupt" | "timer-unsupported" | "timer-calibration")] => m1::execute(case),
        _ => Err(
            "Usage: cargo xtask doctor [--profile m1] [--json] | build --profile feasibility|dev | iso --profile m1 | run --profile vm-baseline | test --suite m0 | test --suite m1 --case iso-smoke|iso-negative|boot-metadata|boot-reject|exception-invalid-opcode|exception-page-fault|panic|exception-emergency|memory-ownership|memory-exhaustion|guard-fault|write-protect-fault|execute-disable-fault|emergency-stack"
                .into(),
        ),
    }
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}
