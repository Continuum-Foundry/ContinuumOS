# P1 epic — Original kernel boots from an offline UEFI ISO

Issue type: Epic. Status: In progress. Owner: Unassigned.
Created: 2026-10-18. Milestone: M1. Suggested labels: `phase-1`, `milestone-m1`.
These are repository-local, Jira-ready stories; no Jira issues have been created.

## Outcome and starting point

Boot an original Rust kernel from the exact packaged ISO, validate boot metadata,
establish exception handling and memory ownership, and demonstrate a single-core
interrupt/timer baseline with honest diagnostics and repeatable test exits.

[Phase 0](../phase-0.md) provides compile/link probes and a local Rust toolchain.
Its [recorded evidence](../../../evidence/phase-0/README.md) does not establish a
bootable kernel, guest allocator, VM, clock or running UI. QEMU, OVMF and ISO tools
were missing in that snapshot; recheck before acquiring them. Review the existing
worktree changes before implementation; a clean Git status is not currently assumed.

## Ordered stories

| ID | Story | Depends on | Backlog coverage | Status |
| --- | --- | --- | --- | --- |
| P1-01 | [Supply and pin VM/ISO prerequisites](01-vm-and-packaging-prerequisites.md) | Phase 0 recheck | B01/B02 follow-up, B38 inputs | Done |
| P1-02 | [Cold-boot the minimal original kernel ISO](02-minimal-iso-boot.md) | P1-01 | B38, B03, minimal B04 | Done |
| P1-03 | [Validate boot metadata and identity](03-boot-metadata-and-diagnostics.md) | P1-02 | B04 | Not started |
| P1-04 | [Establish exception diagnostics](04-exception-handling.md) | P1-03 | B05 exception baseline | Not started |
| P1-05 | [Own physical frames and page tables](05-memory-ownership.md) | P1-04 | B05 memory/guards | Not started |
| P1-06 | [Establish interrupts and monotonic time](06-interrupts-and-timekeeping.md) | P1-05 | M1 subset of B06 | Not started |
| P1-07 | [Run and review Phase 1 acceptance](07-phase-1-acceptance.md) | P1-01 through P1-06 | M1, early B32 | Not started |

Work through this order, one accepted story at a time. Host-only preparation can
continue while a dependency is blocked, but cannot satisfy the dependent guest gate.
B06's preemption/context switching/FP context work remains M3; these stories do not
mark the entire B06 task complete. B32 grows later; Phase 1 does not finish it.

## Fixed scope and defaults

- Windows native host, existing stable Rust target, original first-party Rust kernel
  and a documented minimal assembly boundary. Keep the Phase 0 probes intact.
- QEMU Q35, one vCPU, 2 GiB RAM, TCG, OVMF UEFI, software framebuffer metadata.
  Pin the actual versioned machine identifier and CPU profile in P1-01.
- Limine is the provisional loader. Start from the reviewed 12.9.3 candidate and
  pin compatible protocol/bindings before use; a failed integration needs recorded
  evidence and an ADR update. Do not silently switch to another kernel or nightly.
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
P1-01 host prerequisite evidence and P1-02 minimal guest-entry evidence are retained.
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
named ISO tests exist today. See the [minimal ISO runbook](minimal-iso-runbook.md).
The full Phase 1 suite remains P1-07 work:

| Interface | Owner | Intended behavior |
| --- | --- | --- |
| `cargo xtask build --profile dev` | P1-02 | Build the original M1 kernel and input manifest |
| `cargo xtask iso --profile m1` | P1-02 | Package the exact kernel and pinned UEFI inputs |
| `cargo xtask run --profile vm-baseline` | P1-02 | Fresh, bounded ISO boot with per-run artifacts |
| `cargo xtask test --suite m1 --case <name>` | P1-02 onward | Run one named case with explicit assertions |
| `cargo xtask test --suite m1` | P1-07 | Run the complete declared Phase 1 matrix |

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

Continue with **P1-03** after P1-02 acceptance review and the
[minimal ISO runbook](minimal-iso-runbook.md). Minimal ISO entry is demonstrated;
the remaining M1 metadata, memory, exceptions and timer gates are unstarted.
