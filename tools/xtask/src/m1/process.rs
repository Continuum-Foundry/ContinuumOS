//! Bounded host commands and owning child-process cleanup.
use crate::{Result, sanitize, write_json};
use serde_json::{Value, json};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub(super) const LIMIT: u64 = 2 * 1024 * 1024;

pub(super) fn read_log(path: &Path) -> Result<Vec<u8>> {
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
pub(super) struct OwnedChild(pub(super) Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}
pub(super) fn logs(c: &mut Command, dir: &Path, name: &str) -> Result<(PathBuf, PathBuf)> {
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
pub(super) fn oversize(paths: &[&Path]) -> bool {
    paths
        .iter()
        .any(|p| fs::metadata(p).is_ok_and(|m| m.len() > LIMIT))
}
pub(super) fn capture(c: &mut Command, dir: &Path, name: &str, seconds: u64) -> Result<String> {
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
