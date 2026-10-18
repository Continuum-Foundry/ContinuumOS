# P1-01 host prerequisite evidence

Recorded 2026-10-18, America/New_York. **Host prerequisites pass; no VM launch,
ISO build, firmware boot or guest execution occurred.** Source base and dirty
state, first-party source hashes and command identities are in `source-manifest.json`.
The pre-existing Phase 0 evidence snapshot is unchanged.

## Acceptance review

| Criterion | Evidence and reviewed result |
| --- | --- |
| Phase 0 recheck | `m0/acceptance-results.json`, host/compile/override/path-space logs and `m0/m0-results.json`: pass after implementation |
| Native executables and runtime | `doctor.json` reports QEMU 11.1.0 and xorriso 1.5.8.pl02; all locked DLLs hash correctly; `package-metadata/` records runtime package dependencies/licenses |
| Firmware pairing | `m1-inputs.lock.json` identifies raw 3,653,632-byte code and 540,672-byte vars, distinct hashes, non-Secure-Boot descriptor; firmware boot untested |
| VM capabilities | `doctor.json` retains machine, CPU, accelerator and HPET queries: pc-q35-11.1 / qemu64-v1 / TCG; no VM launched |
| Real input pins | `m1-inputs.lock.json` has downloaded archive and extracted file sizes/hashes. QEMU SHA-512 matches publisher; Limine ZIP matches reviewed upstream SHA-256; crate 0.5.0 matches registry checksum |
| Rejection behavior | `tests/results.json`: baseline and valid override pass; missing explicit executable, corrupted firmware, missing runtime and missing lock block before queries |
| Stable bindings | `bindings.log`: locked no-std base-revision-3 framebuffer/memory-map/RSDP probe compiles and request sizes/alignment pass. Request IDs compared with pinned C header; loader source handles base revision 3 |

Reviewed by Codex against P1-01's checklist. Result: **Done for host prerequisites**,
with the sandbox execution constraint below. No separate human approval or external
review is implied. ADR-0002 remains Proposed. B38 and the M1 boot gate remain open.

## Retained limits and failed experiments

`doctor-sandbox.json` retains the MSYS2 `NtCreateDirectoryObject` access denial.
Identical inputs pass outside the sandbox; no machine setting was changed.
`rejected-binding-0.6.5.log` retains the stable-channel feature error. A direct
rustc attempt for 0.5.0 without its bitflags dependency also failed and is retained;
the final Cargo-locked probe supplies that dependency and passes.

`dependency-recheck/` repeats all five Phase 0 dependency experiments: library
compilation passes; architecture links; the other four binary links still stop
at the documented missing-allocator gate. These remain expected limitations,
not successful guest integrations. An initial invocation omitted Enter-Dev and
could not locate Cargo; its log is retained separately. The corrected invocation
uses the workspace-local toolchain and passes the declared experiment checks.

`supply.log` is a successful cached archive restore, not a second independent
download. MSYS2/7-Zip acquisition hashes are measured pins; no independent package
signature authentication is claimed. Original download/extraction results and
QEMU publisher checksum comparison are recorded in the manifest. Transient GitHub
404s when locating the protocol header were resolved to `include/limine.h`; they
were discovery failures, not accepted build inputs.

Tool and upstream notices remain in ignored tool trees and downloaded archives.
The [runbook](../../../../docs/05-delivery/phase-1/prerequisites-runbook.md) describes
exact pins, notices, execution constraints and the proposed (unexecuted) VM argument
array. A new host must supply and validate its own files. Source hashes identify
this dirty workspace; they do not claim a clean committed release.

`SHA256SUMS.json` covers this retained snapshot, excluding the checksum file itself.
