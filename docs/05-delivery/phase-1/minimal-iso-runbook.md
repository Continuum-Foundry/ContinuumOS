# Minimal original-kernel ISO (P1-02)

This slice provides serial entry from a standalone x86-64 UEFI optical image.
It does not complete M1: checked BootInfo, exception handling, owned frames/page
tables and interrupt/timekeeping evidence remain P1-03 through P1-07.

## Commands

Run from the repository root using PowerShell 7 and the workspace-local tools:

```powershell
. ./tools/Enter-Dev.ps1
cargo xtask doctor --profile m1
cargo xtask build --profile dev
cargo xtask iso --profile m1
cargo xtask run --profile vm-baseline
cargo xtask test --suite m1 --case iso-smoke
cargo xtask test --suite m1 --case iso-negative
pwsh -NoProfile -File tools/Test-P1-02.ps1
```

The dev CLI maps to Cargo's separate `kernel-dev` profile (panic abort,
optimization level 1, debug symbols, no LTO). It does not change the host target.
Kernel builds use `--locked`, remove inherited Rust flag/wrapper overrides, and
produce an immutable build identity before compilation. The canonical input hash
includes first-party build sources, kernel sources, lock/config hashes, compiler
identity, target, flags and test flavor. `build-inputs.json` is the input inventory;
`build-manifest.json` records the ELF digest and source commit/dirty state.

Each command prints its new artifact directory under ignored `build/m1/`.
`iso` builds the normal kernel; `run` also builds/packages a fresh normal ISO.
Smoke builds enable `emulator-test`; negative builds enable `fault-before-entry`.
These are different recorded build identities. No latest-log or latest-ISO cache
is used. The complete future `test --suite m1` remains unavailable until P1-07.

## Packaging and cold boots

Every package and launch passes the same strict M1 prerequisite checks as doctor,
including input sizes/digests and locked runtime directories. Nothing is downloaded.
The xorriso MSYS2 runtime needs execution outside the sandbox's named-object
restriction. Request that narrowly scoped execution context; do not modify host
settings, replace the tools or silently install WSL. The native argument boundary
uses `/cygdrive/<drive>/...` for this pinned xorriso runtime. UNC is unsupported.

The UEFI-only ISO stages the original ELF, `BOOTX64.EFI`,
`limine-uefi-cd.bin`, `limine.conf` and Limine loader/Rust-binding notices. The xorriso recipe uses the
UEFI optical portion of upstream 12.9.3's USAGE.md, without BIOS installation.
Configuration selects the Limine protocol and `boot():/boot/kernel.elf`, immediate
automatic boot, serial/verbose output, and no remembered selection. Packaging
inspects the El Torito catalog for a bootable UEFI entry and independently extracts
and hashes every staged payload before the ISO digest is recorded.

The runner uses the pinned `pc-q35-11.1`/TCG/`qemu64-v1,+apic` profile, one vCPU,
2048 MiB RAM, standard VGA with headless display, and OVMF without Secure Boot.
Only the exact ISO and code/fresh variable pflash pair are attached. No networking,
host shares, data disks, direct-kernel boot, snapshot restore, QMP/GDB or reboot
loop is enabled. Arguments are arrays; QEMU backend filenames containing commas
are explicitly rejected. Paths containing spaces are exercised by ISO staging and
the per-run variable-store copy.

Each launch creates a new variable copy and serial file and records a host session
ID. That ID correlates host artifacts; it is neither guest identity nor proof
against reset. Template/copy hashes, process ID, exact arguments, deadline, raw
logs, final ISO digest and process exit/cleanup are retained. The 30-second deadline
is host time, not guest uptime. Logs exceeding 2 MiB fail the run and trigger cleanup.

`run` observes one matching checkpoint for one second, then stops and reaps QEMU;
this is reported as host cleanup, not guest shutdown. `iso-smoke` enables only the
emulator exit device at 0xf4 and requires **both** one complete matching
`m1.kernel_entry version=1 build=<id>` line and exit 33. It launches two fresh
processes against the same hashed ISO with independent vars. Duplicate/mismatched
entry, panic/rejection, missing marker, unexpected exit and timeout all fail.

`iso-negative` retains two deliberately unsuccessful boot attempts: a separately
identified kernel that halts before entry diagnostics, and a new ISO containing a
smoke-capable payload whose config references a missing kernel. The second case
also requires Limine's actual missing-executable panic in the console transcript.
The suite passes only when those attempts fail as expected and their VM processes
are reaped. Neither is positive boot evidence.

## Review and handoff

`Test-P1-02.ps1` runs M0 acceptance (including host rejection/cleanup tests and the
source-path-with-spaces check), M1 doctor and P1-01 input rejection tests, normal
entry observation, two smoke cold boots and both guest negative cases. Raw ignored
artifacts include compile/package/launch commands, LLVM inspection/disassembly,
ISO payload inventory, optical catalog, serial, stderr/stdout and JSON assertions.
Retain a sanitized compact snapshot under `evidence/phase-1/P1-02/<run-id>/`.
Keep failed runs. An early emulator exit is never sufficient for acceptance.

Before marking Done, review the story's checklist against those records. Keep ADRs
Proposed. P1-03 inherits the loader/serial/ISO harness; it still must validate and
normalize boot descriptors and introduce bounded guest session records.
