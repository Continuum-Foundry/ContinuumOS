# P3-05 — Preempt tasks and preserve integer/FP/SIMD context

Issue type: Story. Epic: P3. Status: Not started. Owner: Unassigned.
Depends on: P3-04 Done. Suggested labels: `phase-3`, `scheduler`, `cpu-context`.
Backlog: Remaining M3 B06 scheduling/context. Requirements: R03/R10/R18;
preparatory R09/R14. Test families: T02/T03, timer/context subset of T04, runtime T09.

## User story

As a service developer, I need preemptive progress and private CPU state so a busy
or faulting task cannot monopolize the CPU or expose another task's registers.

## Implementation scope

1. Extend the accepted interrupt path into single-CPU preemption, with bounded
   runnable queues, idle halt, yield and a documented priority/fairness policy.
   Use kernel-internal threads where needed for bring-up; keep work out of hard IRQ.
   Record preemption/IRQ nesting and lock order; never hold a global lock across a
   user wait or perform blocking allocation/IPC in an interrupt handler.
2. Save/restore complete documented user/kernel continuation state, address space,
   kernel stack and entry bookkeeping. Bound nonpreemptible regions and rescheduling
   points; test an interrupt during syscall/copy/exit without switching into freed state.
3. Review actual emitted user instructions and CPUID/XCR0 policy. Start with eager
   FP/SIMD initialization/save/restore for the supported feature set, including control
   and exception state; prevent state remnants across tasks and process reuse.
   Reject unsupported feature requirements before entry. Preserve kernel FP exclusion
   or document and test any narrowly necessary kernel context use explicitly.
4. Define thread creation/owning-thread rules and bounded termination. Keep the
   selected UI model single-owner; preemption is not permission for concurrent UI
   mutation or interrupt rendering. Define unsupported features/FP exception outcomes.
5. Instrument dispatch/runnable time, switches and interrupt-off intervals with a
   declared observer cost. Use P3-01 fairness limits; reserve final resource/latency
   review for P3-13 without importing desktop performance targets.

## Acceptance criteria

- [ ] Two non-yielding ring-3 workloads and a healthy control task progress under
  timer preemption within fixed fairness bounds; voluntary yield is not the proof.
- [ ] Integer/flags/stack/address-space and supported FP/SIMD/control-state sentinels
  survive repeated switches; fresh tasks observe initialized state, not prior data.
- [ ] Priority, idle, termination and nested IRQ/preemption rules are bounded and
  cannot be elevated or disabled by ordinary user code.
- [ ] Actual instruction/feature audit, unsupported-CPU rejection and FP fault tests
  pass before any renderer/text math is admitted to user tasks.
- [ ] User fault or kill during compute/syscall/copy leaves another task running;
  M1 acknowledgement, uptime and fatal-stack regressions remain intact.

## Tests and evidence

Alternate distinct integer, x87/SSE and any additionally enabled extended-state
patterns across tasks, including rounding/exception controls, fresh/reused process
slots and forced faults. Run equal/mixed-priority CPU hogs, yield loops, syscall
loops and repeated exit during timer delivery. Retain code audit excerpts, progress
and context traces, clock definitions, bounds and maximum interrupt-off observations.

## Exclusions and handoff

No SMP, hard real-time guarantee, GPU contexts or broad asynchronous UI runtime.
P3-06 receives a preemptible execution environment; P3-08 adds public blocking/wake
semantics. P3-12 checks real portable allocation/math hooks without guest rendering.
Sources: [kernel design](../../02-architecture/kernel-and-execution.md),
[timekeeping runbook](../phase-1/interrupts-timekeeping-runbook.md),
[runtime hooks](../../../experiments/ui-link/hooks.json).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
