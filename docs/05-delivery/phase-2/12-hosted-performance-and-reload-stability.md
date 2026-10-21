# P2-12 — Measure hosted idle, damage and repeated reload stability

Issue type: Story. Epic: P2. Status: Not started. Owner: Unassigned.
Depends on: P2-11 Done. Suggested labels: `phase-2`, `performance`, `reload-stability`.
Backlog: M2 subset of B32, B10 stability. Requirements: R09/R18; hosted R04/R08/R15.
Test families: hosted T05/T07/T09/T10; no guest efficiency acceptance.

## User story

As a performance reviewer, I need repeatable optimized hosted measurements and leak
checks so unnecessary work and reload resource growth are caught before guest porting.

## Implementation scope

1. Run the selected final UI with P2-01's fixed optimized profiles, sample counts,
   warm/cold policy and scripts. Recheck the comparison baseline after P2-10/P2-11
   changes; explain differences rather than quoting obsolete candidate measurements.
2. Capture input/dispatch/state/layout/raster/compose/submission phases, p50/p95/p99,
   missed deadlines, allocations, cache bytes, resident/committed memory and peaks.
   Measure static idle CPU/wakeups/frame count for a fixed interval. Name caret,
   clock and accessibility timers; static content must not perpetually repaint.
3. Validate incremental damage against full redraw, including old/new bounds,
   translucent overlap, clipping/filter expansion, scroll, scale and changing
   backgrounds under effects. Bound fragmentation and verify conservative fallback.
4. Run at least 100 consecutive compatible token/layout reloads under the fixed
   script. Assert continuity after every commit and compare resources after defined
   settling intervals. Fix object/cache/generation growth tolerance before the run;
   retained old generations or unexplained monotonic growth fail the gate.
5. Record old/new resources and staging peaks, refused low-headroom updates and
   completed-build-to-visible latency separately from compilation. Compare with
   requirements targets as hosted observations only, preserving the 256 MiB guest
   cold-idle goal and all unimplemented guest subsystem costs.

## Acceptance criteria

- [ ] Final optimized workload results include raw samples, distributions, sample
  counts, build/profile identities, trace overhead and complete memory attribution.
- [ ] Static content has no perpetual repaint/busy loop; declared clock/caret timers
  and their measured idle costs remain in the report.
- [ ] Incremental and full redraw agree within predeclared tolerances across damage
  fixtures; effect/occlusion/cache invalidation never exposes stale pixels.
- [ ] At least 100 consecutive compatible reloads preserve required state with no
  unexplained monotonic resource growth after quiescence under the fixed tolerance.
- [ ] Headroom rejection preserves the active generation, and reload peaks/latency
  are measured honestly; target misses have causes and bounded follow-up issues.
- [ ] Hosted limits and estimated guest costs are separate; neither VM RAM capacity
  nor host process footprint is reported as meeting the guest desktop budget.

## Tests and evidence

Use idle, scroll/type, large-list, drag/resize, overlap/effect and repeated-update
scripts at both resolutions, with scales as declared. Include hidden/minimized
surface pacing, reader-release delay and constrained staging budget. Retain compact
metric summaries with necessary raw samples and per-cycle state/resource outcomes.
Large traces/images remain local. Do not change tolerances after observing a leak;
any revised experiment has a new identity and preserves the failed run.

## Exclusions and handoff

No guest benchmark, GPU benefit, photon latency or whole-system memory guarantee.
M2 requires a measured hosted baseline, not fabricated passing guest target numbers.
P2-13 receives final workload/damage/idle results and the 100-cycle review. Sources:
[requirements targets](../../00-overview/requirements.md#performance-targets-and-accounting),
[performance](../../02-architecture/observability-and-performance.md),
[validation](../validation.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
