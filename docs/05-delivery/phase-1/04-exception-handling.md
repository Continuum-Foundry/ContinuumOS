# P1-04 — Establish bounded kernel exception diagnostics

Issue type: Story. Epic: P1. Status: Done. Owner: Codex.
Depends on: P1-03 Done. Suggested labels: `phase-1`, `kernel`, `fault-handling`.
Backlog: exception subset of B05. Requirements: R01/R03/R17; early R10 groundwork.
Test families: T01/T02; guest faults are mandatory evidence.

## User story

As an OS developer, I need deterministic exception reporting before changing page
tables so a kernel fault produces usable evidence rather than an unexplained reset.

## Implementation scope

1. Establish kernel-owned descriptor/interrupt tables and task-state data needed
   for exception entry. Use statically reserved, aligned stacks under the validated
   bootstrap mapping; do not require the future allocator to diagnose its own startup.
2. Provide a separate emergency stack for double-fault handling and record its
   mapping/lifetime assumptions. Guard mappings are completed and tested in P1-05.
3. Implement the minimum stable-Rust/assembly entry wrappers, normalizing exception
   frames and error-code variants without assuming an unstable Rust interrupt ABI.
   Document stack alignment, register clobbers, privilege assumptions and unsafe sites.
4. Add no-allocation, bounded panic/exception reporting with vector, instruction
   pointer, relevant error code and fault address, plus build/session/stage identity.
   Reentrant logging must have an emergency path that does not deadlock.
5. Keep device interrupts masked. The default fault policy reports then halts or
   exits via the explicit test device; never returns to an unresolved fatal fault
   or automatically restarts the VM.
6. Add named `exception-invalid-opcode`, `exception-page-fault` and `panic` tests.

## Acceptance criteria

- [x] Normal boot reaches `m1.exceptions_ready` without disturbing existing markers.
- [x] Deliberate invalid-opcode and unmapped-access cases produce the expected
  distinguishable vector/context and terminate as expected faults, not normal boots.
- [x] Panic and emergency reporting operate without heap allocation or scheduler help.
- [x] No unexplained reset or triple fault is counted as a passing diagnostic test.
- [x] Assembly/unsafe boundaries have review notes against the actual CPU contract;
  no FP/SIMD reliance is introduced by entry or reporting code.

## Tests and evidence

Guest tests must actually trigger exceptions; printing a synthetic register record
does not pass. Host tests may supplement frame-format/error-code normalization.
Bound every test with a host timeout and retain all failing serial/host output.
Inspect generated entry code and compare diagnostic addresses to symbolized fault
sites. Record emergency-stack setup now; the guard/double-fault integration case
must pass after P1-05 establishes the necessary owned mappings.

## Exclusions and handoff

No user faults/process isolation, external IRQs, scheduler or FP context switching.
An exception handler is not an arbitrary-fault recovery mechanism. P1-05 can now
change mappings with basic diagnostics already established; P1-07 cannot pass until
the deferred emergency-stack integration test passes.
Sources: [kernel contract](../../02-architecture/kernel-and-execution.md),
[CPU source catalogue entry](../../01-research/sources.md#s30).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation: [runbook and CPU/unsafe review](exception-diagnostics-runbook.md).
Evidence: [2026-10-19 acceptance](../../../evidence/phase-1/P1-04/2026-10-19/README.md).
Review: Codex local self-review passed all criteria, 19 host tests and 21 VM launches.
The [final formatting check](../../../evidence/phase-1/P1-04/2026-10-19-formatting/README.md)
covers the final EOF-only assembly cleanup, unchanged entry wrappers and one additional real fault launch.
At this story's review, double-fault/guard integration remained P1-05 and M1 was
incomplete. See [P1-07's later complete review](../../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
for the current experimental-baseline acceptance.
