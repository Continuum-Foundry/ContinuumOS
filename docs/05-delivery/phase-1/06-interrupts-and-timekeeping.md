# P1-06 — Establish interrupts and monotonic kernel uptime

Issue type: Story. Epic: P1. Status: Done. Owner: Codex (local implementation/self-review).
Depends on: P1-05 Done. Suggested labels: `phase-1`, `kernel`, `interrupts`, `clock`.
Backlog: M1 interrupt/timer subset of B06. Requirements: R01/R03/R17; early R09/R10 support.
Test families: T01/T02/T04, single-core baseline only.

## User story

As an OS developer, I need bounded timer interrupts and a kernel-owned monotonic
clock so later scheduling and no-reboot demonstrations have a trustworthy baseline.

## Implementation scope

1. Validate the minimal ACPI table chain needed to discover the bootstrap local
   interrupt controller and timer resources. Check table lengths/checksums and mapped
   ranges before access; do not assume discovery from a pointer's existence alone.
2. Use a local APIC timer and HPET-derived timing as the proposed Q35 baseline.
   P1-01 must expose these capabilities in its pinned profile. Confirm the exact
   device/register/clock contracts before coding; absence or incompatibility blocks
   this route and requires a documented alternative, not an invented frequency.
3. Map MMIO with the required attributes; establish vector ownership, masked startup,
   legacy interrupt handling policy, spurious handling and explicit acknowledgement.
   Unmask only the timer after tables, stacks and state are ready.
4. Implement a bounded interrupt handler that records timer progress and defers work.
   No heap allocation, blocking logs, arbitrary callbacks or IPC in interrupt context.
   Define register preservation and a single-core interrupt-safe counter/read protocol.
5. Initialize uptime once per kernel boot and keep it monotonic, using checked tick
   conversion and a documented resolution/overflow policy. Keep wall time, host time
   and kernel uptime distinct. Use no unvalidated TSC-frequency assumption.
6. Add an idle halt path that resumes for timer work, plus `timer-progress`,
   `timer-masked`, `clock-read` and `idle-wakeup` cases. Never introduce a scheduler
   or promise precision/performance from these functional tests.

## Acceptance criteria

- [x] Normal boot reaches `m1.timer_ready`; progress is established by at least
  100 delivered timer interrupts in a bounded guest run.
- [x] At least 100 kernel uptime reads across delivered interrupts never decrease;
  progress and effective resolution are reported without wall-clock equivalence.
- [x] Serial/session identity remains unchanged during the run; a fresh VM launch
  starts a new boot session and a newly initialized uptime epoch.
- [x] Masking the timer suppresses its delivered-event count while the polling test
  remains responsive; restoring it resumes delivery. Do not require interrupt-backed
  uptime progress while the interrupt is intentionally masked.
- [x] Repeated halt/wakeup cycles return to the same kernel continuation. Exceptions
  remain diagnosable after interrupt delivery is enabled.
- [x] Malformed ACPI fixtures, unsupported discovery and calibration failure produce
  an explicit rejection instead of guessed MMIO addresses or timer values.

## Tests and evidence

Use host fixtures for table length/checksum/range rejection, clock conversion overflow
and synthetic counter-wrap behavior. Use real guest interrupts for delivery,
acknowledgement, masking, register sentinels and idle resumption; host simulation
cannot pass those cases. Preserve calibration inputs, vector/device configuration,
sampled counts/times and fault logs. A host timeout or VM restart fails the case.

## Exclusions and handoff

No threads, task scheduling, preemption, syscall entry or FP/SIMD context switching.
Leave that remainder of B06 open for M3. No SMP, suspend/resume clock, deadline
guarantee or physical-display latency claim. P1-07 receives a stable boot/session
identity and monotonic kernel uptime for final acceptance.
Sources: [kernel contract](../../02-architecture/kernel-and-execution.md),
[platform discovery](../../02-architecture/boot-and-platform.md),
[CPU source](../../01-research/sources.md#s30).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation revision: allocation-free ACPI/xAPIC/HPET adapter and version-5
guest/host diagnostics. See the [runbook](interrupts-timekeeping-runbook.md).
Evidence: [2026-10-20 acceptance](../../../evidence/phase-1/P1-06/2026-10-20/README.md). Review: local Codex self-review passes all six criteria; not independent approval. M1 was incomplete at this review; [P1-07's later complete gate](../../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md) now accepts the declared experimental baseline.
