# P1 epic — Original kernel boots from an offline UEFI ISO

Issue type: Epic. Status: Done (declared experimental VM baseline; local review). Owner: Unassigned.
Created: 2026-10-18. Milestone: M1. Suggested labels: `phase-1`, `milestone-m1`.
These are repository-local, Jira-ready stories; no Jira issues have been created.

## Outcome and starting point

Boot an original Rust kernel from the exact packaged ISO, validate boot metadata,
establish exception handling and memory ownership, and demonstrate a single-core
interrupt/timer baseline with honest diagnostics and repeatable test exits.

[Phase 0](../phase-0.md) provides compile/link probes and a local Rust toolchain.
Its [recorded evidence](../../../evidence/phase-0/README.md) does not establish a
bootable kernel, guest allocator, VM, clock or running UI. P1-01 subsequently
supplied and pinned the local QEMU, OVMF and ISO tools. Re-run prerequisites and
review the current worktree before implementation; do not infer current state
from Phase 0's historical snapshot.

## Ordered stories

| ID | Story | Depends on | Backlog coverage | Status |
| --- | --- | --- | --- | --- |
| P1-01 | [Supply and pin VM/ISO prerequisites](01-vm-and-packaging-prerequisites.md) | Phase 0 recheck | B01/B02 follow-up, B38 inputs | Done |
| P1-02 | [Cold-boot the minimal original kernel ISO](02-minimal-iso-boot.md) | P1-01 | B38, B03, minimal B04 | Done |
| P1-03 | [Validate boot metadata and identity](03-boot-metadata-and-diagnostics.md) | P1-02 | B04 | Done |
| P1-04 | [Establish exception diagnostics](04-exception-handling.md) | P1-03 | B05 exception baseline | Done |
| P1-05 | [Own physical frames and page tables](05-memory-ownership.md) | P1-04 | B05 memory/guards | Done |
| P1-06 | [Establish interrupts and monotonic time](06-interrupts-and-timekeeping.md) | P1-05 | M1 subset of B06 | Done |
| P1-07 | [Run and review Phase 1 acceptance](07-phase-1-acceptance.md) | P1-01 through P1-06 | M1, early B32 | Done |

Work through this order, one accepted story at a time. Host-only preparation can
continue while a dependency is blocked, but cannot satisfy the dependent guest gate.
B06's preemption/context switching/FP context work remains M3; these stories do not
mark the entire B06 task complete. B32 grows later; Phase 1 does not finish it.

## Fixed scope and defaults

- Windows native host, existing stable Rust target, original first-party Rust kernel
  and a documented minimal assembly boundary. Keep the Phase 0 probes intact.
- QEMU Q35, one vCPU, 2 GiB RAM, TCG, OVMF UEFI, software framebuffer metadata.
  Pin the actual versioned machine identifier and CPU profile in P1-01.
- Limine 12.9.3 and stable-compatible Rust bindings 0.5.0 are pinned and exercised
  on the declared profile; the loader choice remains provisional beyond it.
  A changed or failed integration needs recorded evidence and an ADR update.
  Do not silently switch to another kernel or nightly.
- Serial-first diagnostics, fixed storage before any allocator, 4 KiB pages, no
  general heap or FP/SIMD use. Frame allocation is not a guest UI allocator.
- No networking, shared directories, writable guest data disk, snapshot restore,
  direct-kernel boot, physical-disk access or automatic VM restart in acceptance runs.
- No desktop, UI toolkit selection, processes/syscalls, scheduler, service replacement,
  persistence, kernel hot reload or inference. The 256 MiB desktop target is unchanged
  and cannot be assessed from an idle kernel.

## Evidence and story completion

Use `Not started`, `In progress`, `Blocked`, or `Done` in the story and index.
Update both together. A blocked story records the exact failure, evidence location
and next experiment; later stories cannot assume it passed. Checklists begin unchecked.

A story is Done only after all its acceptance criteria pass, its tests and limitations
are recorded, and an explicit review checks the evidence against the criteria. Do not
erase failed runs, silently skip cases, or mark expected failures as successful boots.
Keep the architectural ADRs Proposed until their documented review gates are met.

Store generated run artifacts in ignored `build/m1/<run-id>/`; retain a compact,
sanitized summary and logs in `evidence/phase-1/<story-id>/<run-id>/` when implemented.
Apply the [compact evidence policy](../evidence-retention.md): new bundles have
4 MiB/200-file/256-KiB-per-file defaults, and generated logs/source copies are
ignored until deliberately selected. Preserve required raw outcomes and failures;
deduplicate reports by build/input identity rather than copying entire run trees.
P1-01 host prerequisites, P1-02 minimal guest entry and P1-03 checked metadata and
identity evidence are retained. The
[host module extraction](../../../evidence/phase-1/m1-module-refactor/2026-10-18/README.md)
also passed the existing regression suite; it does not advance a guest story.
Each run must record:

- Source commit and dirty state; if dirty, a patch/source snapshot or hashes covering
  all build inputs. Record toolchain/lock/config hashes and exact upstream provenance.
- Kernel, ISO, firmware and loader hashes; exact VM arguments, CPU/machine/accelerator,
  test case, host process lifecycle, timeouts and any manual intervention.
- Raw serial output, host stdout/stderr, assertions, process exit reason and a concise
  limitations note. Screenshots supplement the serial record; they never replace it.
- Separate outcomes for compile, package, firmware boot, kernel readiness and expected
  fault behavior. An ISO build or earlier run's transcript cannot establish a new boot.

The kernel build identity is fixed before ISO packaging. The host maps that identity
to the final ISO hash in the manifest; do not try to embed an ISO's own final digest
inside itself. Each new VM launch receives a distinct host-generated boot-session
identifier, clearly labeled as host-provided and not a security credential.

## Tooling and remaining proposed commands

Phase 0's commands, P1-01's `doctor --profile m1` and P1-02's build/package/run and
named ISO tests exist today. P1-03 adds the `boot-metadata` and `boot-reject` cases
and the [current acceptance script](../../../tools/Test-P1-03.ps1). See the
[minimal ISO runbook](minimal-iso-runbook.md) and
[metadata contract](boot-metadata-runbook.md).
The full Phase 1 suite and its local review pass P1-07's declared matrix. The
earlier host execution blocker was resolved through approved development signing.
See the [acceptance runbook](acceptance-runbook.md):

| Interface | Owner | Intended behavior |
| --- | --- | --- |
| `cargo xtask build --profile dev` | P1-02 | Build the original M1 kernel and input manifest |
| `cargo xtask iso --profile m1` | P1-02 | Package the exact kernel and pinned UEFI inputs |
| `cargo xtask run --profile vm-baseline` | P1-02 | Fresh, bounded ISO boot with per-run artifacts |
| `cargo xtask test --suite m1 --case <name>` | P1-02 onward | Run one named case with explicit assertions |
| `cargo xtask test --suite m1` | P1-07 | Run the complete declared Phase 1 matrix |

The host entry point is [m1.rs](../../../tools/xtask/src/m1.rs). The
[module layout](minimal-iso-runbook.md#host-harness-source-layout) separates
`build`, `package`, `launch` and `observe`, with `cases` orchestrating scenarios
and `records` validating serial output. Shared helpers keep pin checks and bounded
process execution consistent. Cases supply expected rejection outcomes to launch;
observation owns spawning, deadlines and process cleanup.

Use argument arrays rather than shell-built commands. Validate all required inputs
before spawning a VM. Tests may use an explicitly configured emulator test-exit
device, which is separate from physical-machine shutdown and excluded from normal
runtime assumptions. Timeouts are failures; forced termination is cleanup, not a
guest shutdown result. Never leave a background VM running after a story test.
Use QEMU's no-reboot policy, a new serial file per run, and reject duplicate kernel
entry events. A host-provided session ID alone cannot detect a reset within one
emulator process; never use it as the sole evidence that the kernel did not reboot.

## Traceability

Epic requirements: R01/R02/R03; early support for R10/R16/R17/R18. This does not
complete process isolation, security or release reproducibility requirements.
Test families: the M1 subsets of T01/T02/T03/T04/T12/T13.
Read [milestones](../milestones.md), [backlog](../backlog.md),
[validation](../validation.md), [boot/platform](../../02-architecture/boot-and-platform.md),
[kernel/execution](../../02-architecture/kernel-and-execution.md),
[ADR-0001](../../04-decisions/0001-system-language.md),
[ADR-0002](../../04-decisions/0002-boot-and-iso.md) and
[ADR-0013](../../04-decisions/0013-build-and-distribution.md).

P1-04 builds on the
[P1-03 acceptance review](../../../evidence/phase-1/P1-03/2026-10-18/README.md) and
[metadata runbook](boot-metadata-runbook.md). Checked metadata and session records
are demonstrated. P1-05's [accepted memory ownership](../../../evidence/phase-1/P1-05/2026-10-19/README.md)
adds owned page tables, enforced permissions, stack guards and actual double-fault
entry. P1-06 [interrupts/timekeeping acceptance](../../../evidence/phase-1/P1-06/2026-10-20/README.md) now passes.
P1-07's [complete local acceptance](../../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
passes 33 rows and 42 launches. The next bounded workstream is M2's hosted UI
comparison; general heap, isolation, scheduling, persistence and live reload remain open.
