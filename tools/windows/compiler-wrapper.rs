//! Standalone, dependency-free Cargo compiler wrapper. Preserve OS argument vectors.
use std::{env, ffi::OsString, fs, path::PathBuf, process::Command};

fn main() {
    if let Err(error) = run() {
        eprintln!("ContinuumOS compiler signing: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let compiler = args.first().ok_or("missing compiler")?;
    let status = Command::new(compiler).args(&args[1..]).status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    let value = |flag: &str| {
        args.windows(2)
            .find(|pair| pair[0] == flag)
            .map(|pair| pair[1].clone())
    };
    if let Some(target) = value("--target") {
        if target != "x86_64-pc-windows-msvc" {
            return Ok(());
        }
    }
    let Some(directory) = value("--out-dir") else {
        return Ok(());
    };
    let Some(crate_name) = value("--crate-name") else {
        return Ok(());
    };
    let source = args
        .iter()
        .map(PathBuf::from)
        .find(|path| path.extension().is_some_and(|ext| ext == "rs") && path.is_file())
        .ok_or("missing Rust source path")?;
    let suffix = args
        .iter()
        .filter_map(|arg| arg.to_str())
        .find_map(|arg| arg.strip_prefix("extra-filename="))
        .unwrap_or("");
    let stem = format!("{}{suffix}", crate_name.to_string_lossy());
    let script = env::var_os("CONTINUUM_COMPILER_SIGNER").ok_or("missing signer script")?;
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.file_stem().is_some_and(|name| name == stem.as_str())
            && path.extension().is_some_and(|ext| ext == "exe" || ext == "dll")
        {
            let status = Command::new("pwsh")
                .args(["-NoProfile", "-File"])
                .arg(&script)
                .arg("-Executable")
                .arg(&path)
                .arg("-Source")
                .arg(&source)
                .status()?;
            if !status.success() {
                return Err("host compiler output could not be signed and verified".into());
            }
        }
    }
    Ok(())
}
