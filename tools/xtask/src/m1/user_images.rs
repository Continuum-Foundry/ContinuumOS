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
pub(super) fn build(
    dir: &Path,
    execution: bool,
    scheduler: bool,
    capabilities: bool,
    channels: bool,
    waits: bool,
    supervisor: bool,
    policy: bool,
    routes: bool,
) -> Result<PathBuf> {
    let contract: serde_json::Value =
        serde_json::from_slice(&read(&root().join("contracts/runtime-v0.json"))?)
            .map_err(|e| e.to_string())?;
    let mut ids = String::new();
    for group in ["dispatch_ids", "error_ids"] {
        for (name, value) in contract["syscall"][group]
            .as_object()
            .ok_or("Missing runtime IDs")?
        {
            ids.push_str(&format!(
                "pub const {}: u64 = {};\n",
                name.to_uppercase(),
                value.as_u64().ok_or("Bad runtime ID")?
            ));
        }
    }
    // xor edi,edi; xor r8d,r8d; xor r9d,r9d; mov eax,exit; syscall; ud2.
    let mut trampoline = vec![0x31u8, 0xff, 0x45, 0x31, 0xc0, 0x45, 0x31, 0xc9, 0xb8];
    trampoline.extend_from_slice(
        &(contract["syscall"]["dispatch_ids"]["process_exit"]
            .as_u64()
            .ok_or("Missing exit ID")? as u32)
            .to_le_bytes(),
    );
    trampoline.extend_from_slice(&[0x0f, 0x05, 0x0f, 0x0b]);
    ids.push_str(&format!(
        "pub const EXIT_TRAMPOLINE: &[u8] = &{trampoline:?};\n"
    ));
    let runtime_ids = dir.join("runtime-ids.rs");
    fs::write(&runtime_ids, ids).map_err(|e| e.to_string())?;
    let source = if routes {
        "experiments/user-images/routes.rs"
    } else if policy {
        "experiments/user-images/policy.rs"
    } else if supervisor {
        "experiments/user-images/supervisor.rs"
    } else if waits {
        "experiments/user-images/waits.rs"
    } else if channels {
        "experiments/user-images/channels.rs"
    } else if capabilities {
        "experiments/user-images/capabilities.rs"
    } else if scheduler {
        "experiments/user-images/scheduler.rs"
    } else if execution {
        "experiments/user-images/executable.rs"
    } else {
        "experiments/user-images/fixture.rs"
    };
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
            .arg(root().join(source))
            .env("CONTINUUM_RUNTIME_IDS", &runtime_ids)
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
            if toolname == "llvm-objdump" && execution && !scheduler {
                let permitted = text
                    .lines()
                    .filter(|line| line.contains("%xmm"))
                    .collect::<Vec<_>>();
                if permitted.len() != 1 || !permitted[0].contains("pxor\t%xmm0, %xmm0") {
                    return Err("Expected exactly the forbidden FP trap fixture".into());
                }
                let integer = text
                    .lines()
                    .filter(|line| !line.contains("%xmm"))
                    .collect::<Vec<_>>()
                    .join("\n");
                super::build::audit_disassembly(&integer)?;
            }
            if toolname == "llvm-objdump" && scheduler {
                // Every permitted non-integer instruction is explicit fixture
                // assembly. Reject unexpected compiler/library vector dispatch.
                let mut integer = String::new();
                for line in text.lines() {
                    let mnemonic = line
                        .split_once(':')
                        .map(|(_, v)| v.split_whitespace().next().unwrap_or(""))
                        .unwrap_or("");
                    if [
                        "fxsave64",
                        "fxrstor64",
                        "fninit",
                        "fldcw",
                        "fldz",
                        "fdivp",
                        "fdivrp",
                        "fwait",
                        "wait",
                        "ldmxcsr",
                        "pxor",
                        "divss",
                        "vxorps",
                    ]
                    .contains(&mnemonic)
                    {
                        continue;
                    }
                    integer.push_str(line);
                    integer.push('\n');
                }
                super::build::audit_disassembly(&integer)?;
            }
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
        &json!({"schema_version":1,"target":"x86_64-unknown-none","code_model":"small","relocation":"static","red_zone":false,"panic":"abort","opt_level":1,"entry":"fixture_entry","compiler_sha256":digest(&Path::new(&sysroot).join("bin/rustc.exe"))?,"audit_tools_sha256":audits,"source_sha256":digest(&root().join(source))?,"runtime_source_sha256":digest(&root().join("experiments/user-images/runtime.rs"))?,"runtime_ids_sha256":digest(&runtime_ids)?,"linker_sha256":digest(&root().join("experiments/user-images/link.ld"))?,"images":images,"trusted_descriptors_sha256":digest(&path)?,"user_execution":execution || scheduler,"scheduler":scheduler,"required_cpu_features":if scheduler {7}else{0}}),
    )?;
    Ok(path)
}
