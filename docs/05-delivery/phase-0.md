# Phase 0 — Windows feasibility implementation

Status: compile/tooling slice implemented; see [retained evidence](../../evidence/phase-0/README.md)
for the recorded run. No ISO, VM boot, rendered UI, process isolation or live update
has been demonstrated. This is the user-approved M0 scope; actual ISO boot starts M1.

## Run locally

Use PowerShell 7 from the repository root. An existing Windows MSVC C++ toolset
and Windows SDK/UCRT are required for the host tool. Do not install them silently.

```powershell
# Explicit setup only when Rust is missing. Downloads to work/; no persistent PATH change.
pwsh -NoProfile -File tools/Install-Rust.ps1

# Process-local environment, also usable with an existing matching Rust installation.
. ./tools/Enter-Dev.ps1
cargo xtask doctor
cargo xtask doctor --json
cargo xtask build --profile feasibility
cargo xtask test --suite m0
pwsh -NoProfile -File tools/Test-M0.ps1
pwsh -NoProfile -File tools/Probe-Dependencies.ps1
```

Setup downloads official rustup 1.29.1, checks its recorded SHA-256, and installs
Rust 1.99.1, rustfmt, LLVM tools and the bare-metal target in `work/toolchains/`.
It changes no Windows feature, SDK, global package or shell profile. The initial
sandbox attempt could not create rustup executable links; a workspace-only setup
with the required filesystem access succeeded. Future setup should report such
errors rather than relocating installation to a global directory.

After entering the environment, rustup automatic installation is disabled. Doctor
does not install tools. The first `cargo xtask` invocation builds the host tool and
may download Cargo-locked host dependencies; subsequent doctor execution is read-only.

`doctor` checks PATH and standard installations, records unavailable CIM inspection
as blocked/unknown, and returns nonzero for missing compile prerequisites. Missing
QEMU/firmware/ISO tools remain visible but do not fail a compile-only Phase 0 gate.
No authentication tokens, computer name, full environment dump or account email is collected.

Explicit executable paths use `CONTINUUM_RUSTC`, `CONTINUUM_CARGO`,
`CONTINUUM_PWSH`, `CONTINUUM_LLVM_READOBJ`, `CONTINUUM_LLVM_NM`,
`CONTINUUM_QEMU_SYSTEM_X86_64` and `CONTINUUM_XORRISO`. Set
`CONTINUUM_OVMF_CODE` to the firmware image. An invalid override is rejected without
falling back to another executable. `CONTINUUM_LINK` is a doctor discovery override;
Cargo's Windows host linker discovery is still provided by Rust/MSVC.

## Build boundaries and interfaces

The workspace defaults to the host xtask, without a global guest target.
Kernel and userland probes are separate binaries using `x86_64-unknown-none`.
The [Rust target documentation](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html)
describes ELF output, no standard library/default allocator, a kernel code model,
and SysV C calling conventions without a red zone.

| Property | Kernel probe | Userland probe |
| --- | --- | --- |
| Entry | `kernel_probe_entry`, `0xffffffff80000000` | `user_probe_entry`, `0x400000` |
| Code model | kernel | small |
| Link policy | static ET_EXEC, separate RX/R/RW segments | independent linker script, static ET_EXEC |
| Runtime | no heap, no std, panic abort, spin-only entry | no heap, no std, panic abort, integer transform |

Userland's experimental entry accepts a `u64` in RDI and returns a `u64` in RAX.
Before CALL the caller aligns RSP to 16 bytes; entry RSP modulo 16 is 8 with a valid
return address. DF is clear; the normal SysV preserved-register rules apply.
No syscall numbers or cross-process Rust memory layouts are introduced.
The probe is not executed, and no process loader exists. The kernel probe has no
Limine requests, boot stack setup or serial output and is not a bootable kernel.

No first-party inline assembly is introduced. `core::hint::spin_loop` uses the
target's architecture-specific hint; the library architecture experiment exercises
address/page-table APIs without executing privileged instructions.

Builds produce JSON manifests, hashes, ELF files and sanitized command logs under
`build/m0/`. A narrow, bounds-checked ELF inspector checks class, machine, entry,
load ranges, permissions, symbol closure, relocations, and absence of dynamic/TLS
segments. This host validator is not the future security-critical guest ELF loader.
LLVM readobj/nm independently corroborate the generated artifacts.

The M0 test builds in two fresh directories, compares bytes, and retains both
outputs. The PowerShell acceptance harness additionally copies source into a path
containing spaces, builds there, and checks doctor overrides. Failure logs are kept.
These checks establish repeatable probe builds in this environment, not full release
reproducibility or Windows/Linux parity.

## Dependency feasibility

An isolated, Cargo-locked experiment prevents UI candidates from becoming kernel
dependencies. Each function invokes real candidate APIs with external inputs rather
than relying on an unused import. No host fonts or model assets are embedded.

| Candidate | Exact version/features | Recorded result |
| --- | --- | --- |
| x86_64 | 0.15.5; defaults off, instructions | guest library compile and link pass |
| tiny-skia | 0.12.0; defaults off, no-std-float | library compiles; binary requires allocator |
| rustybuzz | 0.20.1; defaults off | library compiles; binary requires allocator |
| fontdue | 0.9.4; defaults off, hashbrown | library compiles; binary requires allocator |
| Slint | 1.18.1; defaults off, compat-1-18, unsafe-single-threaded, libm, renderer-software | software-renderer APIs compile; binary requires allocator |

Final symbol closure for allocation-dependent probes remains unverified because
the compiler stops before linking. No dummy allocator or platform-success stub is
supplied. Allocation, FP/SIMD context, font parsing limits, clock/input/event-loop
hooks, clipboard, semantics and rendering correctness remain later integration work.
Slint's single-thread feature requires single-thread-only use, outside interrupt
handlers; it is a feasibility configuration, not an accepted multithreaded backend.
Hosted preview/interpreter support was not tested and is not implied by guest compilation.

The generated dependency report includes exact transitive versions, checksums,
features, license expressions, declared MSRVs, build scripts and proc macros. It
includes host compiler/build dependencies and is not a list of shipped guest code.
Build scripts execute on the host; a compile pass does not certify unsafe code or
parser behavior. Slint's GPL/commercial/royalty-free choices require a distribution
decision before adoption. No project license is selected. Full comparable scenes,
license suitability, semantics and measurements remain B36/M2, confirmed in M4.

## Phase 1 packaging and boot runbook

This section records Phase 0's starting point and proposed recipe. P1-01 later
supplied the tools; P1-02 now implements and demonstrates the minimal ISO route.
Use the current [minimal ISO runbook](phase-1/minimal-iso-runbook.md) for execution.

The baseline remains Q35, one vCPU, 2 GiB RAM, TCG, software display, UEFI,
network disabled and no host shares. WHPX is untested. QEMU, OVMF and xorriso are
missing in the recorded environment, so their versions/hashes are intentionally
unset. No VM was launched or optional Windows feature enabled.

1. Supply native Windows QEMU, xorriso and OVMF; record tested versions, hashes,
   licenses and the QEMU versioned Q35 machine identifier before boot testing.
   Inspect QEMU's reported accelerators; do not enable WHPX implicitly.
2. Limine 12.9.3 is the reviewed candidate. Its binary ZIP's upstream-reported
   SHA-256 is in the toolchain manifest; it has not been downloaded or executed.
   Pin matching protocol bindings and verify the archive before using it.
3. Add a minimal original Rust kernel implementing the selected boot handoff and
   serial readiness marker. The current link probe cannot substitute for it.
4. Stage the matching Limine files, configuration and kernel in an ISO root. Follow
   [Limine's pinned ISO recipe](https://github.com/Limine-Bootloader/Limine/blob/v12.9.3/USAGE.md).
   It uses xorriso and real optical boot files; a renamed disk image is insufficient.
5. Hash the packaged ISO and boot that exact file through OVMF's optical path.
   Copy any writable firmware variable template per run. Disable network/shares,
   use file-backed serial logs, and bound execution with a timeout.
6. Retain the QEMU command/configuration, firmware and ISO hashes, serial marker,
   failure transcripts and shutdown outcome before declaring B03/B38 complete.

Rust OSDev bootloader 0.11.17 remains an alternative: its documented builder
produces BIOS/UEFI disk images and requires nightly Rust. Inspection of its
published `build.rs` found nested `cargo install` and `-Zbuild-std` commands;
it was not admitted to the stable workspace. No equivalent ISO path or Windows build has been
tested here. Limine is preferred provisionally for its documented ISO recipe;
neither route is boot-proven. If native packaging is blocked, propose a separate
Linux CI packaging job without silently installing WSL or changing the guest OS.

## Remaining gates

B01/B02 have compile/tooling evidence. B38 and the full B36 comparison remain open.
All architectural ADRs remain Proposed. M1 must prove actual ISO boot; M2/M4 must
establish UI behavior; M3 must prove isolation; M5 must demonstrate live state
transfer. The 256 MiB cold-idle target is unchanged and unmeasured.
