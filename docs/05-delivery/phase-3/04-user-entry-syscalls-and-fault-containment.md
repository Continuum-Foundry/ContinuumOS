# P3-04 — Enter user mode and enforce the syscall/fault boundary

Issue type: Story. Epic: P3. Status: Done. Owner: Unassigned.
Depends on: P3-03 Done. Suggested labels: `phase-3`, `syscalls`, `fault-containment`.
Backlog: B11 entry/syscalls, B13 failure groundwork.
Requirements: R03/R10/R14/R18; preparatory R17. Test families: T01/T02/T03/T11.

## User story

As a user-process developer, I need an explicit entry/syscall/runtime boundary so
normal execution works and a bad pointer, panic or user fault leaves the kernel alive.

## Implementation scope

1. Implement reviewed x86-64 user entry and syscall return with owned per-thread
   kernel stack/TSS state, exact register/clobber/stack alignment rules, IF/DF and
   privilege checks. Validate canonical return IP/SP and user-controlled flags;
   use a return path that handles invalid states without a kernel privilege fault.
2. Supply minimal discovery, bounded diagnostic, allocation/mapping, yield and exit
   operations needed by fixtures. Derive current process/context from trusted entry
   state, not user arguments. Unsupported syscall/version returns a typed error.
3. Implement checked copy-in/out across page boundaries and overflow-safe lengths,
   user/supervisor permissions and required access rights. Tie validation to mapping
   lifetime; process termination or later preemption must not create a check/use gap.
   Define partial output and fault behavior before clients rely on it.
4. Add a minimal no-std Rust user runtime using reclaiming allocation and bounded
   startup/panic/OOM/exit paths. An ordinary panic or allocation failure reports and
   terminates the process; a terminal spin or VM debug-exit is not a user supervisor.
5. Split user exception containment from kernel fatal diagnostics. Run a replacement
   healthy fixture after the offender dies in the same guest. Until P3-05 passes,
   use audited fixtures that do not require user FP/SIMD; do not execute UI math.

## Acceptance criteria

- [x] Real ring-3 fixtures use discovery/allocation/log/exit and return through the
  documented syscall boundary; privilege, register and interrupt-state checks pass.
- [x] Invalid, RO, kernel, other-process and boundary-crossing pointers fail safely;
  no user numeric address or fabricated identity becomes trusted access.
- [x] User page/guard/illegal-instruction/privileged-instruction faults, panic and OOM
  have bounded process-local outcomes, with subsequent healthy guest execution.
- [x] Kernel/IST stacks and M1 kernel-fatal paths remain valid; bad user return state
  cannot turn a process-local fault into a successful kernel entry or fatal cascade.
- [x] Runtime allocation/free and process exit restore the declared inventories;
  assembly and unsafe-copy invariants receive an explicit recorded review.

## Tests and evidence

Exercise register/DF/flags sentinels, noncanonical IP/SP, forged syscall number,
bad version, null/end/overflow buffers, mapped/unmapped mixed pages and denied
copies. Retain actual guest traces proving process-fatal versus kernel-fatal
distinction, boot ID continuity and post-fault progress. Mapping races gain real
preemption/handle coverage at P3-05/P3-06; do not credit hosted models as that result.

## Exclusions and handoff

No fairness, general blocking IPC, live replacement or UI execution. P3-05 turns the
single-process execution boundary into preemptive tasks; P3-06 replaces provisional
memory authority with rights-bearing objects. Sources:
[kernel design](../../02-architecture/kernel-and-execution.md),
[exception runbook](../phase-1/exception-diagnostics-runbook.md),
[objects contract](../../06-contracts/ipc-and-objects.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation commands and review obligations: [syscall boundary runbook](syscall-boundary-runbook.md). The [2026-10-26 local acceptance review](../../../evidence/phase-3/P3-04/2026-10-26/README.md) passes every criterion. This is local Codex self-review, not external approval.
