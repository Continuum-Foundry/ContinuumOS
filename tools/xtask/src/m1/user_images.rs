//! Separate Rust user links, exact trusted descriptors, no host dependency.
use super::{
    process::capture,
    support::{digest, read},
};
use crate::{Result, elf, output, root, tool, write_json};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
pub(super) fn build(dir: &Path) -> Result<PathBuf> {
    let mut descriptors = String::from("pub const IMAGES: [([u8;32],usize);3] = [\n");
    let mut images = Vec::new();
    let sysroot = output(Command::new(tool("rustc")?).args(["--print", "sysroot"]))?;
    let llvm = Path::new(&sysroot).join("lib/rustlib/x86_64-pc-windows-msvc/bin");
    let mut audits = std::collections::BTreeMap::new();
    for name in ["llvm-readobj", "llvm-nm", "llvm-objdump"] {
        audits.insert(name, digest(&llvm.join(format!("{name}.exe")))?);
    }
    for name in ["user_a", "user_b", "service"] {
        let image = dir.join(format!("{name}.elf"));
        let mut c = Command::new(tool("rustc")?);
        c.current_dir(root())
            .arg(root().join("experiments/user-images/fixture.rs"))
            .args([
                "--edition=2024",
                "--target=x86_64-unknown-none",
                "--crate-type=bin",
                "--cfg",
                name,
                "-C",
                "panic=abort",
                "-C",
                "opt-level=1",
                "-C",
                "code-model=small",
                "-C",
                "relocation-model=static",
                "-C",
                "no-redzone=yes",
                "-C",
                "link-arg=--build-id=none",
                "-C",
            ])
            .arg(format!(
                "link-arg=-T{}",
                root().join("experiments/user-images/link.ld").display()
            ))
            .arg(format!(
                "--remap-path-prefix={}=/continuum",
                root().display()
            ))
            .arg("-o")
            .arg(&image);
        capture(&mut c, dir, &format!("{name}-build"), 60)?;
        let bytes = read(&image)?;
        let inspection = elf::inspect(&bytes, "userland-link")?;
        for (toolname, args) in [
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
            let text = capture(
                Command::new(llvm.join(format!("{toolname}.exe")))
                    .args(args)
                    .arg(&image),
                dir,
                &format!("{name}-{toolname}"),
                30,
            )?;
            if toolname == "llvm-nm" && !text.trim().is_empty() {
                return Err("User image has undefined symbols".into());
            }
        }
        let sha = digest(&image)?;
        let raw = (0..64)
            .step_by(2)
            .map(|i| u8::from_str_radix(&sha[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        descriptors.push_str(&format!("({raw:?},{}),\n", bytes.len()));
        images.push(json!({"name":name,"sha256":sha,"bytes":bytes.len(),"inspection":inspection}));
    }
    descriptors.push_str("];\n");
    let path = dir.join("trusted-images.rs");
    fs::write(&path, descriptors).map_err(|e| e.to_string())?;
    write_json(
        &dir.join("user-images.json"),
        &json!({"schema_version":1,"target":"x86_64-unknown-none","code_model":"small","relocation":"static","red_zone":false,"panic":"abort","opt_level":1,"entry":"fixture_entry","compiler_sha256":digest(&Path::new(&sysroot).join("bin/rustc.exe"))?,"audit_tools_sha256":audits,"source_sha256":digest(&root().join("experiments/user-images/fixture.rs"))?,"linker_sha256":digest(&root().join("experiments/user-images/link.ld"))?,"images":images,"trusted_descriptors_sha256":digest(&path)?,"user_execution":false}),
    )?;
    Ok(path)
}
