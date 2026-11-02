# P3-13 — Stress isolation, teardown and runtime resource bounds

Issue type: Story. Epic: P3. Status: Done. Owner: Codex (local implementation/self-review).
Depends on: P3-12 Done. Suggested labels: `phase-3`, `fault-injection`, `accounting`.
Backlog: B06/B11–B13 robustness, M3 B32 subset.
Requirements: R10/R14/R18; preparatory R06/R09/R17.
Test families: T03/T06/T11/T09; synthetic route-fault subset of T08.

## User story

As a runtime reviewer, I need sustained adversarial guest evidence and resource
accounting so isolated happy-path examples cannot hide leaks, starvation or stale authority.

## Implementation scope

1. Build one bounded adversarial suite over final public SDK fixtures, with scenario
   IDs, seeds and fixed limits. Cover allocation/launch failures, memory/copy faults,
   handle reuse/transfer, wait/cancel races, queue pressure, supervision, role/session
   invalidation and route fencing. Include combined faults across these boundaries.
2. Run at least 100 complete launch/work/terminate/reap cycles and 100 synthetic
   route/fence cycles in a continuous guest, plus P3-01's fixed stress duration/counts.
   Keep a healthy control service progressing during CPU hog, OOM, crash loop,
   cancellation storm and peer/broker death. These are runtime cycles, not reloads.
3. Account all guest kernel/runtime/fixture bytes: resident/committed/private/shared,
   page tables, stacks, allocator metadata, object/handle/channel/wait/timer tables,
   caches and pending work. Count shared backing once globally and show attribution.
   Capture staging/death/queue peaks as well as settled baselines and genuinely free
   RAM. Explain every retained bounded cache and step change before passing.
4. Measure scheduling fairness/dispatch/wake and IPC latency/throughput distributions,
   idle behavior, maximum interrupt-off intervals and diagnostics/trace overhead on
   declared one-vCPU profiles. Use guest clocks for guest intervals and separate
   host VM/process observations; emulator elapsed results are profile-specific.
5. Reject unexplained settled monotonic growth, resource/invariant violations and
   fairness/deadline misses against fixed runtime bounds. Fix causes in owning
   stories and rerun affected cases; do not widen tolerances after seeing results.
   Record desktop memory/frame/input targets as still unaccepted by this workload.

## Acceptance criteria

- [x] All fixed scenario/fault classes and both 100-cycle runs complete with exact
  expected outcomes, healthy control progress and no hidden reset/snapshot restore.
- [x] Quiescent object/reference/frame inventories return to baseline or a justified
  fixed cache bound; no unexplained growth, stale writer or surviving waiter occurs.
- [x] Guest memory totals reconcile with backing/ownership and reserved/free bytes;
  transient old/new/queued peaks and admission refusal remain within declared limits.
- [x] Fairness, dispatch/wake/deadline and IPC measurements retain sample counts,
  p50/p95/p99/max, failures and observer overhead with no host/guest clock conflation.
- [x] Combined resource/authority faults remain contained; failed attempts and any
  remediation retain exact source IDs and regression results before final review.

## Tests and evidence

Use deterministic interleaving hooks plus real timer-preempted stress; neither is
a substitute for the other. Include kill during copy/map/transfer/wait, receiver
exhaustion after send acceptance, policy expiry with queued work, supervisor/broker
restart with retained endpoints and cancellation after synthetic mutation commit.
Keep compact selected raw traces, resource time series, independent inventory/epoch
assertions and failed-run summaries. Full traces and artifacts stay local.

## Exclusions and handoff

No broad hostile-code/security certification, desktop efficiency acceptance,
state-preserving reload, durability or real-device result. P3-14 receives a stable
final runtime, reproducible stress commands and explicit measured limitations.
Sources: [validation](../validation.md),
[performance accounting](../../02-architecture/observability-and-performance.md),
[identity plan](../../02-architecture/identity-and-authorization.md),
[retention policy](../evidence-retention.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).


## Local criterion review

P3-13 passes local Codex self-review for bounded SDK stress and resource accounting. Ten continuous guests complete 100 waves each: 2000 worker lifecycles, 1000 route/fence cycles, 100000 measured IPC round trips and 10000 warmups, with healthy control progress and exact settled inventories. A separate public supervision guest preserves 29 admissions across four epochs. The final-source regressions pass 95 kernel host tests, ten prerequisites, complete M1 (33 rows/42 launches), prior runtime/SDK/portable checks and 14 separate metadata flavors. The optimized native observer retains the original 30000-ms deadline and all validation; regression reuse requires identical measured source and compiler hashes. Only the existing exact SDK contract subset remains frozen. P3-14 is next; M3 exit, guest UI/reload and desktop efficiency targets remain open.

See the [compact acceptance evidence](../../../evidence/phase-3/P3-13/2026-11-01/README.md), [criterion/unsafe review](../../../evidence/phase-3/P3-13/2026-11-01/review.json), [stress runbook](stress-runbook.md) and [ADR-0029](../../04-decisions/0029-bounded-runtime-stress-observation.md). Earlier rejected guests and the orchestration error remain retained as failures; bounds and tolerances were unchanged.
