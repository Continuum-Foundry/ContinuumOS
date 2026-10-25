# P3-01 — Recheck prerequisites and define runtime contracts/profiles

Issue type: Story. Epic: P3. Status: Not started. Owner: Unassigned.
Depends on: P1-07 and P2-13 Done; current Phase 0 acceptance recheck.
Suggested labels: `phase-3`, `prerequisites`, `runtime-contracts`.
Backlog: B01/B02 follow-up, B06/B11–B13 design, B32 setup.
Requirements: R03/R14/R18; preparatory R09/R10/R16. Test families: T02/T13; T09 setup.

## User story

As a kernel developer, I need a current reproducible baseline and a bounded runtime
contract so user-mode work preserves accepted boot behavior and tests real boundaries.

## Implementation scope

1. Inspect source/dirty inputs, M1/P1-07, M2/P2-13 and the performance follow-up
   reviews. Recheck Phase 0 commands, offline dependency availability, compiler and
   existing development signing, then M1 doctor and current baseline boot/regressions.
   Record historical local-only evidence availability; do not overwrite prior runs.
2. Inventory actual user-link targets/code models and M2's runtime hook obligations.
   Define minimal kernel allocation, address-space/process/thread ownership and
   panic/OOM policy. Decide bounded tables versus fallible heap use explicitly;
   no non-reclaiming link-probe arena becomes the runtime allocator.
3. Write implementation-level draft startup/syscall/object/IPC contracts: registers,
   return/error/clobber rules, user ranges, stacks, discovery, fixed-width codecs,
   version policy and trusted context. Establish one contract source for generated
   SDK bindings; defer permanent ABI assignment until independent P3-12 tests.
4. Specify the one-CPU scheduling/lock/IRQ model, initial CPU feature policy and
   FP/SIMD gate, monotonic deadline source, rollover and timer-overrun semantics.
   P1-06 delivered ticks alone cannot certify elapsed deadlines; choose and test
   an elapsed monotonic source or explicitly bounded conversion at P3-08.
5. Lock runtime test profiles, process/mapping/heap/handle/channel/wait/message
   limits, stress counts, fairness bounds and measurement clocks before runs.
   The contract's proposed 4 KiB/8 handles/64 messages remains a measured default,
   not an implicit mandatory capacity. Document actual commands and bounded runner
   setup, serial versions, expected outcomes, timeout and process cleanup.

## Acceptance criteria

- [ ] Current Phase 0, tool/signing/doctor and baseline VM results are recorded;
  a failed prerequisite blocks extension instead of borrowing a historical pass.
- [ ] Draft runtime/SDK contracts cover startup, privilege transitions, pointers,
  allocation, identity, errors, lifetime and version discovery without native-layout
  serialization or premature ABI freeze.
- [ ] Scheduling, lock order, IRQ restrictions, FP/SIMD enablement gate and deadline
  clock obligations have named owning stories and testable policies.
- [ ] Resource limits, profile/CPU features, seeds, stress counts, tolerances and
  measurement rules are fixed; runtime memory is distinct from VM capacity.
- [ ] Bounded runner configuration rejects missing/invalid inputs and records
  nonzero exit, timeout and cleanup distinctly; actual documented commands work.

## Tests and evidence

Retain compact current prerequisite results, contract/profile hashes and the hook
gap map. Exercise invalid CPU/profile/limits, unavailable image/tool, missing final
record, early exit and timeout, with all owned host processes reaped. Run baseline
M1 acceptance before changing the kernel; planning alone does not require it.

## Exclusions and handoff

No runtime implementation or new security/performance claim follows from the draft.
P3-02 receives validated inputs and memory ownership/admission rules; later stories
receive explicit draft interfaces and negative-case budgets. Sources:
[Phase 0](../phase-0.md), [handoff](../agent-handoff.md),
[kernel design](../../02-architecture/kernel-and-execution.md),
[runtime hooks](../../../experiments/ui-link/hooks.json),
[objects contract](../../06-contracts/ipc-and-objects.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
