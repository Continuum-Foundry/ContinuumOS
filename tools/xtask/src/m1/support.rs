//! Artifact paths, file reads and digests shared by the M1 stages.
use crate::{Result, hash, root};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}
pub(super) fn digest(path: &Path) -> Result<String> {
    Ok(hash(&read(path)?))
}
pub(super) fn fresh(label: &str) -> Result<PathBuf> {
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
pub(super) fn json_read(p: &Path) -> Result<Value> {
    serde_json::from_slice(&read(p)?).map_err(|e| e.to_string())
}
pub(super) fn subdir(dir: &Path, name: &str) -> Result<PathBuf> {
    let p = dir.join(name);
    fs::create_dir(&p).map_err(|e| e.to_string())?;
    Ok(p)
}
