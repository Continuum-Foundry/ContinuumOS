//! Spawn, supervise and reap one guest; assert its correlated serial records.
use super::{
    process::{OwnedChild, logs, oversize, read_log},
    records::{self, Expected},
};
use crate::{Result, write_json};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

fn qmp_line(value: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}
fn qmp_send(input: &mut impl Write, journal: &mut fs::File, value: &Value) -> Result<()> {
    // Serialize a complete request before touching the pipe, and retain exact
    // bytes for protocol diagnosis. Formatting a Value directly into ChildStdin
    // did not produce accepted nested requests in the retained Windows run.
    let bytes = qmp_line(value)?;
    journal.write_all(&bytes).map_err(|e| e.to_string())?;
    input.write_all(&bytes).map_err(|e| e.to_string())
}
fn qmp_request(
    input: &mut impl Write,
    journal: &mut fs::File,
    output: &Path,
    value: &Value,
    until: Instant,
) -> Result<Value> {
    qmp_send(input, journal, value)?;
    loop {
        let bytes = read_log(output)?;
        for line in bytes
            .split_inclusive(|b| *b == b'\n')
            .filter(|l| l.ends_with(b"\n"))
        {
            let response: Value = serde_json::from_slice(line).map_err(|e| e.to_string())?;
            if response.get("id") == value.get("id") {
                return Ok(response);
            }
            if response.get("error").is_some() && response.get("id").is_none() {
                return Err(format!("Uncorrelated QMP error: {response}"));
            }
        }
        if Instant::now() >= until {
            return Err("QMP response missed the existing guest/diagnostic deadline".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

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
    Read::by_ref(reader)
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
    if c.get_args().any(|a| a == "-qmp") {
        c.stdin(std::process::Stdio::piped());
    }
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
    let mut captured = false;
    let mut blank_captured = false;
    let mut capture_tail = Vec::new();
    let mut qmp_journal =
        fs::File::create_new(dir.join("qmp-requests.jsonl")).map_err(|e| e.to_string())?;
    if let Some(input) = child.0.stdin.as_mut() {
        let response = qmp_request(
            input,
            &mut qmp_journal,
            &out,
            &json!({"execute":"qmp_capabilities","id":0}),
            start + deadline,
        )?;
        if response.get("error").is_some() {
            return Err(format!("QMP negotiation failed: {response}"));
        }
    }
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
        if child.0.stdin.is_some() && !captured {
            capture_tail.extend_from_slice(&appended);
        }
        if !captured
            && !blank_captured
            && capture_tail
                .windows(b"m1.display_output".len())
                .any(|w| w == b"m1.display_output")
        {
            let filename = dir.join("display-blank.ppm");
            if let Some(input) = child.0.stdin.as_mut() {
                for command in [
                    json!({"execute":"stop","id":4}),
                    json!({"execute":"screendump","arguments":{"filename":filename},"id":5}),
                    json!({"execute":"cont","id":6}),
                ] {
                    let response =
                        qmp_request(input, &mut qmp_journal, &out, &command, start + deadline)?;
                    if response.get("error").is_some() {
                        return Err(format!("QMP capture failed: {response}"));
                    }
                }
                blank_captured = true;
            }
        }
        if !captured
            && capture_tail
                .windows(b"m1.display_pattern".len())
                .any(|w| w == b"m1.display_pattern")
        {
            let filename = dir.join("display.ppm");
            if let Some(input) = child.0.stdin.as_mut() {
                for command in [
                    json!({"execute":"stop","id":1}),
                    json!({"execute":"screendump","arguments":{"filename":filename},"id":2}),
                    json!({"execute":"cont","id":3}),
                ] {
                    let response =
                        qmp_request(input, &mut qmp_journal, &out, &command, start + deadline)?;
                    if response.get("error").is_some() {
                        return Err(format!("QMP capture failed: {response}"));
                    }
                }
                captured = true;
            }
        }
        if capture_tail.len() > 32 {
            capture_tail.drain(..capture_tail.len() - 32);
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
    #[test]
    fn qmp_nested_paths_are_complete_json_lines() {
        for filename in [r"C:\a path\capture.ppm", "unicode-é-\"quoted\".ppm"] {
            let request = json!({"execute":"screendump","arguments":{"filename":filename},"id":2});
            let line = qmp_line(&request).unwrap();
            assert_eq!(line.iter().filter(|b| **b == b'\n').count(), 1);
            assert_eq!(serde_json::from_slice::<Value>(&line).unwrap(), request);
        }
    }
    #[test]
    #[ignore = "explicit one-VM no-ISO QMP protocol diagnostic"]
    fn qmp_no_iso_protocol_diagnostic() {
        let dir = fresh("qmp-protocol").unwrap();
        println!("QMP diagnostic artifacts: {}", dir.display());
        let pins = super::super::inputs::preflight(&dir, "qmp-preflight").unwrap();
        let mut c = Command::new(pins.file("qemu").unwrap());
        c.args([
            "-machine",
            "none",
            "-nodefaults",
            "-display",
            "none",
            "-qmp",
            "stdio",
            "-S",
        ]);
        let (out, _) = logs(&mut c, &dir, "qemu").unwrap();
        c.stdin(std::process::Stdio::piped());
        let mut child = OwnedChild(c.spawn().unwrap());
        let start = Instant::now();
        let mut journal = fs::File::create_new(dir.join("qmp-requests.jsonl")).unwrap();
        let input = child.0.stdin.as_mut().unwrap();
        for value in [
            json!({"execute":"qmp_capabilities","id":0}),
            json!({"execute":"screendump","arguments":{"filename":dir.join("a path with spaces and a long nested filename - capture.ppm")},"id":2}),
            json!({"execute":"quit","id":3}),
        ] {
            qmp_request(
                input,
                &mut journal,
                &out,
                &value,
                start + Duration::from_secs(30),
            )
            .unwrap();
        }
        while child.0.try_wait().unwrap().is_none() && start.elapsed() < Duration::from_secs(30) {
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            child.0.try_wait().unwrap().is_some(),
            "bounded QMP child did not exit"
        );
        child.0.wait().unwrap();
        let text = String::from_utf8(read_log(&out).unwrap()).unwrap();
        let replies: Vec<Value> = text
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert!(
            replies
                .iter()
                .any(|r| r["id"] == 0 && r.get("return").is_some())
        );
        assert!(
            replies.iter().any(|r| r["id"] == 2
                && r["error"]["desc"] == "There is no console to take a screendump from"),
            "nested QMP request was not accepted: {text}"
        );
        assert!(
            replies
                .iter()
                .any(|r| r["id"] == 3 && r.get("return").is_some())
        );
    }
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
    #[ignore = "native child entry exercised by the observer lifecycle parent test"]
    fn native_observer_fixture_child() {
        let Ok(mode) = std::env::var("CONTINUUM_M1_OBSERVER_CHILD_MODE") else {
            return;
        };
        match mode.as_str() {
            "exit-without-marker" => std::process::exit(33),
            "marker-wrong-exit" => {
                fs::write(std::env::var_os("CONTINUUM_M1_OBSERVER_CHILD_SERIAL").unwrap(),b"m1.kernel_entry version=5 build=abc session=def session_origin=host seq=0 stage=entry outcome=ok emergency=false fixture=normal\n").unwrap();
                std::process::exit(0);
            }
            "timeout" => thread::sleep(Duration::from_secs(10)),
            "log-limit" => {
                std::io::stdout().write_all(&vec![b'x'; 3_000_000]).unwrap();
                thread::sleep(Duration::from_secs(10));
            }
            _ => panic!("unknown native fixture mode"),
        }
    }
    #[test]
    fn process_success_failure_timeout_and_log_limit_are_reaped() {
        for (name, expected) in [
            ("exit-without-marker", "Missing complete"),
            ("marker-wrong-exit", "Missing complete"),
            ("timeout", "deadline"),
            ("log-limit", "log limit"),
        ] {
            let dir = fresh(name).unwrap();
            // Re-enter this already signed native test binary. Shell startup is
            // not the child lifecycle behavior under test; keep the 3-s bound.
            let mut c = Command::new(std::env::current_exe().unwrap());
            c.args([
                "--exact",
                "m1::observe::tests::native_observer_fixture_child",
                "--ignored",
                "--nocapture",
            ])
            .env("CONTINUUM_M1_OBSERVER_CHILD_MODE", name)
            .env("CONTINUUM_M1_OBSERVER_CHILD_SERIAL", dir.join("serial.log"));
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
            if name == "exit-without-marker" || name == "marker-wrong-exit" {
                assert_eq!(r["timed_out"], false, "{r}");
                assert_eq!(
                    r["emulator_test_exit"],
                    if name == "exit-without-marker" { 33 } else { 0 }
                );
            }
            assert!(r["error"].as_str().unwrap().contains(expected), "{r}");
        }
    }
}
