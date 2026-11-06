//! Spawn, supervise and reap one guest; assert its correlated serial records.
use super::{
    process::{OwnedChild, logs, oversize, read_log},
    records::{self, Expected},
};
use crate::{Result, write_json};
use serde_json::{Value, json};
use std::{
    fs,
    io::Read,
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

fn read_appended(
    reader: &mut fs::File,
    read_bytes: &mut u64,
    appended: &mut Vec<u8>,
) -> Result<()> {
    let length = reader.metadata().map_err(|e| e.to_string())?.len();
    if length < *read_bytes {
        return Err("Serial log truncated".into());
    }
    if length > super::process::LIMIT {
        return Err("Serial output limit exceeded".into());
    }
    reader
        .by_ref()
        .take(super::process::LIMIT + 1 - *read_bytes)
        .read_to_end(appended)
        .map_err(|e| e.to_string())?;
    *read_bytes += appended.len() as u64;
    Ok(())
}

pub(super) fn observe(
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
    let mut reader = fs::File::open(&serial).map_err(|e| e.to_string())?;
    let mut parser = records::StreamParser::default();
    let mut read_bytes = 0u64;
    let mut read_ns = 0u128;
    let mut parse_ns = 0u128;
    let mut appended = Vec::new();
    let mut seen_at = None;
    let mut timeout = false;
    let mut error = None;
    let mut stopped = false;
    let code = loop {
        if oversize(&[&serial, &out, &err]) {
            error = Some("VM log limit exceeded".into());
            break None;
        }
        let read_start = Instant::now();
        appended.clear();
        let read = read_appended(&mut reader, &mut read_bytes, &mut appended);
        read_ns += read_start.elapsed().as_nanos();
        match read {
            Ok(()) => {}
            Err(e) => {
                error = Some(e);
                break None;
            }
        }
        let parse_start = Instant::now();
        let progress = parser.feed(&appended, expected);
        parse_ns += parse_start.elapsed().as_nanos();
        match progress {
            Ok(true) => {
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
    let report = json!({"observer_read_bytes":read_bytes,"observer_read_ns":read_ns,"observer_parse_ns":parse_ns,"schema_version":1,"status":if checked.is_ok(){"passed"}else{"failed"},"error":checked.err(),"build_id":expected.build,"host_session_id":expected.session,"fixture":expected.fixture,"pid":pid,"deadline_ms":deadline.as_millis(),"timed_out":timeout,"kernel_entry_observed":bytes.split(|b| *b == b'\n').any(|l| l.starts_with(b"m1.kernel_entry ")),"emulator_test_exit":if test {code}else{None},"final_process_exit":final_status.code(),"host_stopped_after_checkpoint":stopped,"cleanup_reaped":true,"elapsed_host_ms":start.elapsed().as_millis(),"guest_shutdown_claimed":false});
    if let Ok(progress) = records::check(&bytes, expected, true) {
        write_json(
            &dir.join("guest-records.json"),
            &json!({"schema_version":5,"timer_ready":progress.timer,"memory_ready":progress.memory,"exceptions_ready":progress.exceptions,"fatal":progress.fatal,"validated":progress.validated,"complete":progress.complete,"rejected":progress.rejected,"records":progress.records}),
        )?;
    }
    write_json(&dir.join("run-result.json"), &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::super::support::fresh;
    use super::*;
    use crate::tool;
    #[test]
    fn append_reader_reads_only_new_bytes_and_rejects_truncation_and_overflow() {
        use std::io::Write;
        let dir = fresh("append-reader").unwrap();
        let path = dir.join("serial.log");
        let mut writer = fs::File::create(&path).unwrap();
        let mut reader = fs::File::open(&path).unwrap();
        let mut count = 0;
        let mut bytes = Vec::new();
        writer.write_all(b"abc").unwrap();
        read_appended(&mut reader, &mut count, &mut bytes).unwrap();
        assert_eq!(bytes, b"abc");
        bytes.clear();
        read_appended(&mut reader, &mut count, &mut bytes).unwrap();
        assert!(bytes.is_empty());
        writer.write_all(b"def").unwrap();
        read_appended(&mut reader, &mut count, &mut bytes).unwrap();
        assert_eq!(bytes, b"def");
        assert_eq!(count, 6);
        writer.set_len(1).unwrap();
        assert!(
            read_appended(&mut reader, &mut count, &mut bytes)
                .unwrap_err()
                .contains("truncated")
        );
        writer.set_len(super::super::process::LIMIT + 1).unwrap();
        assert!(
            read_appended(&mut reader, &mut count, &mut bytes)
                .unwrap_err()
                .contains("limit")
        );
    }
    #[test]
    fn process_success_failure_timeout_and_log_limit_are_reaped() {
        for (name, script, expected) in [
            ("exit-without-marker", "exit 33", "Missing complete"),
            (
                "marker-wrong-exit",
                "[IO.File]::WriteAllText($args[0], 'm1.kernel_entry version=5 build=abc session=def session_origin=host seq=0 stage=entry outcome=ok emergency=false fixture=normal' + [char]10); exit 0",
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
                    fault_rip: None,
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
