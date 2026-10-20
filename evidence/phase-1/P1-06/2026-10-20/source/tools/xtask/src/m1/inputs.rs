//! Pinned M1 input validation and the shared prerequisite gate.
use super::{
    process::capture,
    support::{digest, json_read},
};
use crate::{Result, root, tool};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

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
pub(super) struct Pins {
    pub(super) lock: Value,
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
    pub(super) fn file(&self, id: &str) -> Result<&Path> {
        self.paths
            .get(id)
            .map(PathBuf::as_path)
            .ok_or_else(|| format!("Missing pin {id}"))
    }
    pub(super) fn suffix(&self, suffix: &str) -> Result<&Path> {
        self.paths
            .values()
            .find(|p| p.to_string_lossy().replace('\\', "/").ends_with(suffix))
            .map(PathBuf::as_path)
            .ok_or_else(|| format!("Missing pinned {suffix}"))
    }
}
pub(super) fn preflight(dir: &Path, name: &str) -> Result<Pins> {
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

#[cfg(test)]
mod tests {
    use super::super::support::fresh;
    use super::*;
    use crate::hash;
    use serde_json::json;
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
}
