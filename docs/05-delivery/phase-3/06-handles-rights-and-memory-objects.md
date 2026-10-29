# P3-06 — Enforce handle rights and memory-object lifetimes

Issue type: Story. Epic: P3. Status: Done. Owner: Unassigned.
Depends on: P3-05 Done. Suggested labels: `phase-3`, `capabilities`, `memory-objects`.
Backlog: B12 object/handle/mapping subset. Requirements: R10/R14/R18.
Test families: T02/T03/T06/T11.

## User story

As an isolated client, I need process-local rights-bearing handles so guessing an ID,
duplicating a handle or mapping a shared object cannot increase my authority.

## Implementation scope

1. Implement typed kernel objects and bounded process-local handle tables with
   anti-stale generations, explicit exhaustion/wrap policy, reference accounting,
   close, query and attenuating duplicate. Keep object identity distinct from the
   numeric handle slot and trusted principal/session/process identities.
2. Apply operation-specific rights/type checks to process/thread and memory objects
   already present. Replace provisional memory syscalls with granted-object access;
   define initial handle installation only through trusted launch/admission.
3. Expose checked memory create/map/unmap/protect. Validate object offset/length,
   target-space authority, mapping rights and backing permissions; deny cross-process
   mutation and writable/executable alias escalation. Share immutable backing with
   explicit read-only grants, not pointer or owner-ID knowledge.
4. Define close/duplicate/map lifetime: closing one handle need not unmap an existing
   mapping or revoke duplicates. Track backing and mapping references until the last
   legal user is gone; pin syscall copies under preemption and race-safe teardown.
5. Precharge handle/object/mapping quotas, unwind every failed installation and log
   bounded diagnostics. Document unimplemented device/IRQ/DMA families as unsupported,
   without a general device grant or mapped kernel debug interface.

## Acceptance criteria

- [x] Forged, stale, cross-process and wrong-type handles fail with exact errors;
  generation exhaustion cannot silently resurrect old authority.
- [x] Duplicate/query/close enforce rights and cannot amplify grants or reveal kernel
  pointers; duplicate close and process death have defined resource outcomes.
- [x] Real guest tasks cannot read/write kernel or another task's private memory;
  explicit shared read succeeds while write/protect/execute escalation is denied.
- [x] Mapping/close/copy/termination interleavings preserve valid backing lifetime
  and zero-on-reuse without deadlock, use-after-free or leaked references.
- [x] Direct-handle and shared-mapping revocation limits are explicit and tested;
  quotas and failures restore the declared object/handle/frame inventories.

## Tests and evidence

Exercise guessed/reused slots, forced generation boundary, wrong rights/type,
duplicate amplification, close-after-duplicate, full handle tables and mapping
overflow/alias changes. Run preempted copy versus unmap/exit, shared backing after
creator death and last-reference cleanup. Preserve actual denial/fault records,
positive shared-read evidence and before/after reference/memory inventories.

## Exclusions and handoff

No recursive universal revocation, public device resources or DMA isolation claim.
P3-07 adds channel objects and transfer acceptance using these lifetime/rights rules.
P3-10 adds scoped policy; it cannot replace mandatory kernel rights checks. Sources:
[capability design](../../02-architecture/capabilities-and-ipc.md),
[objects contract](../../06-contracts/ipc-and-objects.md),
[ADR-0010](../../04-decisions/0010-capability-security.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Local Codex self-review passes the bounded experimental subset. See the [reviewed evidence](../../../evidence/phase-3/P3-06/2026-10-29/README.md) and [runbook](handles-memory-runbook.md). This is not external approval, universal revocation, public IPC or the M3 exit.
