//! Hosted prerequisite fixture only. No window, renderer, guest code or descendants.
use crate::{Result, hash, root, sanitize, write_json};
use serde_json::{Value, json};
use std::{
    env, fs,
    io::Read,
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const LIMIT: u64 = 65536;

fn keys(value: &Value, expected: &[&str]) -> Result<()> {
    let object = value.as_object().ok_or("Expected JSON object")?;
    if object.len() != expected.len() || expected.iter().any(|key| !object.contains_key(*key)) {
        return Err(format!("Expected exactly these fields: {expected:?}"));
    }
    Ok(())
}

fn validate(config: &Value, profile: &str, scale: &str, case: &str) -> Result<()> {
    keys(
        config,
        &[
            "schema_version",
            "profiles",
            "scales",
            "refresh_hz",
            "timeout_ms",
            "input",
            "measurement",
        ],
    )?;
    if config["schema_version"] != 1
        || config["refresh_hz"] != 60
        || config["scales"] != json!([1.0, 1.5, 2.0])
    {
        return Err("Unsupported version, refresh or scale matrix".into());
    }
    if config["profiles"]
        != json!({"hd":{"width":1280,"height":720},"full-hd":{"width":1920,"height":1080}})
    {
        return Err("Profiles must be hd=1280x720 and full-hd=1920x1080".into());
    }
    if !matches!(profile, "hd" | "full-hd") || !matches!(scale, "1.0" | "1.5" | "2.0") {
        return Err("Unsupported profile or scale; use hd|full-hd and 1.0|1.5|2.0".into());
    }
    if !matches!(case, "success" | "nonzero" | "timeout") {
        return Err("Unknown lifecycle fixture".into());
    }
    if !config["timeout_ms"]
        .as_u64()
        .is_some_and(|n| (100..=10000).contains(&n))
    {
        return Err("timeout_ms must be an integer in 100..=10000".into());
    }
    keys(&config["input"], &["path", "sha256"])?;
    let digest = config["input"]["sha256"]
        .as_str()
        .ok_or("Missing input digest")?;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("Expected lowercase SHA-256 input digest".into());
    }
    // Versioned measurement protocol: changes require an explicit new implementation.
    if config["measurement"] != measurement() {
        return Err("Unsupported measurement protocol; see hosted runbook".into());
    }
    Ok(())
}

fn measurement() -> Value {
    json!({"clock":"std::time::Instant (Windows QPC); process-relative nanoseconds",
        "seed":20261020,"warmup_ms":5000,"sample_ms":100,"active_ms":30000,"idle_ms":30000,
        "repetitions":5,"reload_cycles":100,"instrumentation":"paired enabled/disabled runs; report delta, never subtract silently",
        "workloads":["static-clock-caret","editor-search-typing-complex-script","virtual-list-10000-scroll","overlap-drag-resize","launcher-settings-rich-and-reduced-effects","token-layout-reload"],
        "aggregation":"retain raw samples; nearest-rank p50/p95/p99; report missed periods and lost events",
        "scope":"future hosted measurements only; P2-01 measures no UI or guest performance"})
}

fn bounded_read(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("Input/log exceeds 64 KiB".into());
    }
    Ok(bytes)
}

struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

fn launch(
    directory: &Path,
    input: &Path,
    case: &str,
    timeout_ms: u64,
    report: &mut Value,
) -> Result<()> {
    let executable = env::current_exe().map_err(|e| e.to_string())?;
    report["executable_sha256"] = hash(&fs::read(&executable).map_err(|e| e.to_string())?).into();
    let mut command = Command::new(executable);
    command
        .args(["hosted-probe", case])
        .arg(input)
        .stdin(Stdio::null())
        .stdout(fs::File::create_new(directory.join("stdout.log")).map_err(|e| e.to_string())?)
        .stderr(fs::File::create_new(directory.join("stderr.log")).map_err(|e| e.to_string())?);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    report["arguments"] = json!(["hosted-probe", case, sanitize(&input.to_string_lossy())]);
    let start = Instant::now();
    let mut child = OwnedChild(command.spawn().map_err(|e| format!("spawn failed: {e}"))?);
    report["launched"] = true.into();
    report["cleanup"] = "unconfirmed".into();
    report["child_pid"] = child.0.id().into();
    let mut forced = None;
    let status = loop {
        if let Some(status) = child.0.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        if start.elapsed() >= Duration::from_millis(timeout_ms) {
            forced = Some("timeout");
        }
        if ["stdout.log", "stderr.log"]
            .iter()
            .any(|name| fs::metadata(directory.join(name)).is_ok_and(|m| m.len() > LIMIT))
        {
            forced = Some("output-limit");
        }
        if forced.is_some() {
            // Only this exact in-process fixture is launched; it never spawns children.
            child.0.kill().map_err(|e| format!("kill failed: {e}"))?;
            break child.0.wait().map_err(|e| format!("reap failed: {e}"))?;
        }
        thread::sleep(Duration::from_millis(5));
    };
    child
        .0
        .wait()
        .map_err(|e| format!("final reap failed: {e}"))?;
    report["cleanup"] = "reaped".into();
    report["exit_code"] = json!(status.code());
    report["elapsed_ns"] = json!(start.elapsed().as_nanos().to_string());
    report["outcome"] = forced
        .unwrap_or(if status.success() {
            "exited-zero"
        } else {
            "exited-nonzero"
        })
        .into();
    if forced.is_some() || !status.success() {
        return Err(format!("fixture outcome: {}", report["outcome"]));
    }
    let bytes = bounded_read(&directory.join("stdout.log"))?;
    let observed: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if observed != json!({"fixture_version":1,"input_sha256":report["input_sha256"]}) {
        return Err("Child output did not match the validated fixture input".into());
    }
    Ok(())
}

pub(super) fn execute(config_path: &str, profile: &str, scale: &str, case: &str) -> Result<()> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let directory = root().join(format!("build/m2/P2-01/run-{stamp}-{}", std::process::id()));
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let mut clock_pairs = (0..4096)
        .map(|_| {
            let start = Instant::now();
            start.elapsed().as_nanos() as u64
        })
        .collect::<Vec<_>>();
    write_json(&directory.join("clock-samples.json"), &json!(clock_pairs))?;
    clock_pairs.sort_unstable();
    let mut report = json!({"schema_version":1,"story":"P2-01","run_id":directory.file_name().unwrap().to_string_lossy(),
        "host_pid":std::process::id(),"os":env::consts::OS,"architecture":env::consts::ARCH,
        "profile":profile,"scale":scale,"case":case,"launched":false,"cleanup":"not-launched","exit_code":null,
        "outcome":"validation-rejected","rendered":false,"guest_metrics":false,
        "debug_assertions":cfg!(debug_assertions),"harness_source_sha256":hash(include_bytes!("m2.rs")),
        "cargo_manifest_sha256":hash(include_bytes!("../../../Cargo.toml")),"cargo_lock_sha256":hash(include_bytes!("../../../Cargo.lock"))});
    report["clock_probe"] = json!({"pairs":4096,"unit":"ns","minimum":clock_pairs[0],"p50":clock_pairs[2047],"p95":clock_pairs[3891],"scope":"timer-call baseline only; not UI instrumentation overhead"});
    let result = (|| {
        let bytes = bounded_read(Path::new(config_path))?;
        report["config_sha256"] = hash(&bytes).into();
        let config: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        validate(&config, profile, scale, case)?;
        report["configuration"] = config.clone();
        let relative = Path::new(
            config["input"]["path"]
                .as_str()
                .ok_or("Input path must be a string")?,
        );
        if relative.is_absolute() {
            return Err("Input path must be workspace relative".into());
        }
        let input = root()
            .join(relative)
            .canonicalize()
            .map_err(|e| format!("Input unavailable: {e}"))?;
        if !input.starts_with(root().canonicalize().map_err(|e| e.to_string())?) {
            return Err("Input must remain inside workspace".into());
        }
        let input_hash = hash(&bounded_read(&input)?);
        if config["input"]["sha256"] != input_hash {
            return Err("Input checksum mismatch".into());
        }
        report["input_sha256"] = input_hash.into();
        report["outcome"] = "launch-error".into();
        let deadline = if case == "timeout" {
            100
        } else {
            config["timeout_ms"].as_u64().unwrap()
        };
        report["deadline_ms"] = deadline.into();
        launch(&directory, &input, case, deadline, &mut report)
    })();
    report["status"] = if result.is_ok() { "passed" } else { "failed" }.into();
    if let Err(error) = &result {
        report["reason"] = sanitize(error).into();
    }
    write_json(&directory.join("result.json"), &report)?;
    println!("Hosted lifecycle evidence: {}", directory.display());
    result
}

pub(super) fn probe(case: &str, input: &str) -> Result<()> {
    match case {
        "timeout" => {
            thread::sleep(Duration::from_secs(5));
            Ok(())
        }
        "nonzero" => Err("Deliberate lifecycle failure".into()),
        "success" => {
            println!(
                "{}",
                json!({"fixture_version":1,"input_sha256":hash(&bounded_read(Path::new(input))?)})
            );
            Ok(())
        }
        _ => Err("Unknown child fixture".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Value {
        serde_json::from_str(include_str!("../../../config/m2-profiles.json")).unwrap()
    }
    #[test]
    fn fixed_matrix_and_rejections() {
        for profile in ["hd", "full-hd"] {
            for scale in ["1.0", "1.5", "2.0"] {
                assert!(validate(&config(), profile, scale, "success").is_ok());
            }
        }
        for (field, bad) in [
            ("profiles", json!({"hd":{"width":0,"height":720}})),
            ("scales", json!([1.0, 0.0])),
            ("refresh_hz", json!(120)),
            ("timeout_ms", json!(0)),
            ("measurement", json!({})),
        ] {
            let mut value = config();
            value[field] = bad;
            assert!(validate(&value, "hd", "1.0", "success").is_err(), "{field}");
        }
        assert!(validate(&config(), "4k", "1.0", "success").is_err());
        assert!(validate(&config(), "hd", "NaN", "success").is_err());
        let mut unknown = config();
        unknown["unknown"] = true.into();
        assert!(validate(&unknown, "hd", "1.0", "success").is_err());
    }
}
