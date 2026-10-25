# P3-08 — Complete waits, timers, cancellation and backpressure

Issue type: Story. Epic: P3. Status: Not started. Owner: Unassigned.
Depends on: P3-07 Done. Suggested labels: `phase-3`, `waits`, `timers`.
Backlog: B12 waiting/deadlock subset, B06 deadlines. Requirements: R10/R14/R18;
preparatory R09/R17. Test families: T03/T06; timer subset of T04 and runtime T09.

## User story

As a client, I need race-safe waits and bounded deadlines so pressure, cancellation
or peer failure cannot silently lose a message, strand a waiter or stall the kernel.

## Implementation scope

1. Add events, bounded wait sets, timer objects and waitable channel/process state.
   Define atomic readiness observation/registration, level/edge behavior, multiple
   waiters and wake selection; integrate blocked/runnable transitions with P3-05.
2. Implement a tested elapsed monotonic deadline service using P3-01's selected
   source. Preserve boot uptime identity and state its delivered-tick limitations
   separately. Specify resolution, overflow/saturation, timer overrun/coalescing and
   long IRQ-off behavior; wall-clock types cannot be accepted as deadlines.
3. Add deadline/cancel outcomes to bounded send/receive/wait operations. Specify the
   winner when ready, timeout, cancel, close and termination coincide. Distinguish
   cancellation request from confirmed cancellation and message acceptance from
   service effects; timeout never silently changes transfer ownership.
4. Prevent kernel deadlocks with lock-order assertions and no global lock held while
   sleeping. Bound waits/queues and ensure a hung client/service can be cancelled
   or killed. Document application wait cycles as explicit timeout/error cases;
   do not promise automatic detection of every possible user deadlock.
5. Measure idle halt/wake and wake latency with fixed clocks and observer cost.
   Supply public clock/wait hooks for M4 pacing, without claiming input, frame
   scheduling or hard real-time guarantees.

## Acceptance criteria

- [ ] Signal/enqueue/close at every registration boundary cannot lose a wakeup;
  wait-set bounds, fairness policy and object destruction outcomes are deterministic.
- [ ] Deadlines use elapsed monotonic semantics with tested resolution/overrun behavior;
  delayed timer delivery does not silently extend a supposedly elapsed timeout.
- [ ] Timeout/cancel/peer-close races return exact outcomes with the specified queued
  bytes/handles and no double completion or abandoned waiter references.
- [ ] Queue pressure and deliberate request cycles remain bounded; cancellation,
  timeout and process termination allow healthy guest tasks to continue.
- [ ] Idle tasks block/halt rather than poll indefinitely; wake/cleanup evidence and
  existing timer/uptime regressions pass without a desktop latency claim.

## Tests and evidence

Exercise already-ready, zero deadline, future deadline, timer cancel/rearm, overrun,
rollover boundary, closed handle and wait-set exhaustion. Force wake-before-register,
wake-during-register, cancel-at-delivery and kill-during-wait interleavings. Build
two- and three-party wait cycles and verify bounded outcomes and control-task progress.
Retain clock comparisons within the guest, no cross-machine subtraction, plus
wait/transfer inventories and selected idle/wake raw observations.

## Exclusions and handoff

No wall-clock/time-zone service, real-time contract or public async executor.
P3-09 receives reliable wait/join/timer primitives for supervision; M4 receives
explicit clock/wait hooks whose display/input integration still needs evidence.
Sources: [objects contract](../../06-contracts/ipc-and-objects.md),
[kernel design](../../02-architecture/kernel-and-execution.md),
[timekeeping runbook](../phase-1/interrupts-timekeeping-runbook.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
