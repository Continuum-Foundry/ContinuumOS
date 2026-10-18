# P1-01 — Supply and pin Windows VM/ISO prerequisites

Issue type: Story. Epic: P1. Status: Done. Owner: Codex.
Depends on: Phase 0 acceptance recheck. Suggested labels: `phase-1`, `build`, `platform`.
Backlog: B01/B02 follow-up, B38 prerequisite work. Requirements: R02/R03/R16/R18.
Test families: T02/T12/T13 prerequisites; no kernel boot acceptance here.

## User story

As an OS developer, I need verified native Windows packaging and emulator inputs
so later boot results can be reproduced without relying on hidden host setup.

## Implementation scope

1. Inspect the current worktree and rerun the Phase 0 acceptance harness. Preserve
   the existing evidence snapshot; record a new run rather than overwriting history.
2. Recheck QEMU, OVMF and xorriso paths, including explicit overrides and standard
   installations. Reuse suitable existing tools. Report concrete URLs, sizes,
   destination paths and any approval-required changes before acquiring missing tools.
3. Prefer portable tools under ignored `work/`; if a native xorriso package requires
   companion runtime DLLs, pin and inventory that runtime too. Never copy an isolated
   executable and assume its dependencies exist. Do not install WSL or enable WHPX.
4. Pin the tested QEMU executable, xorriso/runtime, firmware code and variable-store
   template, Limine binaries and matching protocol/bindings. Start with the reviewed
   Limine candidate; verify actual downloads, notices and feature compatibility.
5. Record the exact versioned Q35 machine name supported by the supplied QEMU,
   an explicit CPU model supported under TCG, one vCPU, 2 GiB RAM, COM1 serial and
   optical boot policy. Keep all management/debug endpoints disabled by default.
   Explicitly retain the local APIC/HPET baseline proposed for P1-06 in the profile;
   register-level discovery remains unverified until the guest experiment runs.
6. Extend doctor with actionable prerequisite/version/hash checks. A missing M1
   prerequisite must block M1 commands while retaining the compile-only M0 behavior.

## Acceptance criteria

- [x] Phase 0 tests still pass; source state and test evidence are identified.
- [x] QEMU and xorriso actually execute their version/help queries successfully;
  required DLLs are present and accounted for.
- [x] Firmware code and variable-store inputs have compatible formats/sizes, distinct
  roles, provenance, hashes and recorded Secure Boot mode. Boot compatibility stays untested.
- [x] QEMU reports the selected machine, CPU and TCG support. No VM boot is claimed.
- [x] Tool/protocol/loader locks contain real versions and verified hashes; unavailable
  inputs are recorded as blockers rather than filled with speculative pins.
- [x] Missing files, invalid overrides and hash mismatches produce clear failures
  before VM launch; no install, network exposure or machine setting change is implicit.

## Tests and evidence

Exercise existing valid overrides, nonexistent paths, corrupted copied inputs and
missing runtime dependencies where applicable. Capture version/capability output,
checksum comparisons, sanitized doctor output and all acquisition failures. Record
the Windows SDK/MSVC/Rust identities reused from Phase 0.

If native ISO tooling cannot be supplied, preserve the error and propose the separate
Linux CI packaging alternative from [Phase 0](../phase-0.md). That is a blocked
Windows packaging route, not permission to change the build host automatically.

## Exclusions and handoff

No production kernel implementation or ISO boot in this story. Do not claim graphics
acceleration from a CPU accelerator listing. On acceptance, P1-02 receives exact
input locks and a complete proposed VM configuration, not a boot-success claim.
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation revision: 2026-10-18, input hashes and dirty source identity retained
in [P1-01 evidence](../../../evidence/phase-1/P1-01/2026-10-18/README.md).
Review: acceptance criteria checked against the retained host results; pass with
the documented MSYS2 sandbox execution limit. No boot or guest compatibility result.
See [the prerequisite runbook](prerequisites-runbook.md) for acquisition, commands,
license/provenance limits and the proposed launch profile. P1-02 is next.
