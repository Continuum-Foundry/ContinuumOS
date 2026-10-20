# Minimal original-kernel ISO (P1-02)

P1-02 established serial entry from a standalone x86-64 UEFI optical image.
The current harness also implements P1-03's metadata and P1-04's version-3 fatal
diagnostics; see the [metadata runbook](boot-metadata-runbook.md) and
[exception runbook](exception-diagnostics-runbook.md). Historical evidence remains
unchanged. P1-05 adds [owned frames/page tables, stack guards and double-fault
integration](memory-ownership-runbook.md) with version-4 diagnostics; check that
story's acceptance status. P1-06 timekeeping passes; M1 still needs P1-07 acceptance,
currently [blocked by host execution policy](../../../evidence/phase-1/P1-07/2026-10-20/README.md).

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
is used. The complete `test --suite m1` now exists; see the
[acceptance runbook](acceptance-runbook.md) and its current blocker.

## Host harness source layout

`tools/xtask/src/m1.rs` is the CLI entry point. Its `m1/` modules separate
kernel compilation and binary audits (`build`), fresh configuration and verified
ISO construction (`package`), pinned QEMU preparation (`launch`), and guest
process supervision and reporting (`observe`). `observe` keeps spawning and
owning the child together so early errors still kill and reap the process.

`cases` orchestrates the named scenarios and supplies expected rejection outcomes;
`records` validates the current version-5 serial contract, including P1-07's final
`m1.ready` after boot completion. `suite` aggregates the complete gate. Shared `inputs`, `process` and
`support` modules handle prerequisite/pin checks, bounded host commands and
artifact file operations. The command names, manifest schemas and evidence paths
are preserved. Source moves change the build identity because all xtask sources
are hashed; session identity remains independent of the kernel build.
The [module-refactor regression review](../../../evidence/phase-1/m1-module-refactor/2026-10-18/README.md)
records the post-split acceptance run and build identities.

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

Each current launch packages a new configuration with a host session ID, copied
once into the guest. The kernel bytes/build ID remain independent of that session;
each configuration and final ISO have their own hash. The ID is correlation, not
a security credential or proof against reset. Each launch also creates a new
variable copy and serial file. Template/copy hashes, process ID, arguments, deadline, raw
logs, final ISO digest and process exit/cleanup are retained. The 30-second deadline
is host time, not guest uptime. Logs exceeding 2 MiB fail the run and trigger cleanup.

`run` observes completed metadata diagnostics for one second, then reaps QEMU;
this is reported as host cleanup, not guest shutdown. `iso-smoke` enables only the
emulator exit device at 0xf4 and requires matching version-3 entry, validation,
resource summaries, completion and exit 33. It launches two fresh processes with
identical kernel bytes/build IDs and distinct sessions/configured ISO hashes.
Duplicate/mismatched entry, panic/rejection, sequence errors, incomplete summaries,
unexpected exit and timeout all fail. P1-02's historical version-1/same-ISO
comparison remains in its retained evidence; current assertions use P1-03's contract.

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

Review each story's checklist against its own evidence and keep ADRs Proposed.
P1-03 extends this harness with checked owned descriptors and guest session records;
its [acceptance script](../../../tools/Test-P1-03.ps1) adds both new named cases.
