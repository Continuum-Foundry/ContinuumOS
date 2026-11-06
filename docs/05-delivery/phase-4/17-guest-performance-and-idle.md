# P4-17 — Measure and improve guest damage, pacing and latency

Issue type: Story. Epic: P4. Status: Not started. Owner: Graphics/Performance.
Depends on: P4-16. Backlog: M4 B32, B16 efficiency.
Requirements: R04/R08/R09/R15/R17. Test families: T05/T07/T09/T10.
Labels: `phase-4`, `performance`, `idle`.

## User story

As a desktop user, I need responsive guest rendering and a quiet static desktop,
with measurements that include real presentation, input and resource reload costs.

## Implementation scope

1. Measure the locked 720p/1080p scale/effect/text workload cells: typing/search,
   scrolling, moving/resizing overlapping windows, motion/caret/clock, hidden windows
   and static idle. Retain warmups, trace-on/off pairs, sample counts and distributions.
2. Separate app layout/raster, compositor CPU, copy/present submission, scheduler
   delay and input-to-submission on a shared guest monotonic timeline. CPU accounting
   must not masquerade as elapsed time; avoid host/guest clock subtraction. Measure
   host completed-build-to-observed-visible-commit plus guest stages with a declared
   observation bound; ACK alone is not proof of visible display.
3. Port/validate damage, opaque occlusion, bounded caches/scratch and pacing against
   full-redraw output. Repaint old/new/blur-source bounds correctly, cap fragmentation
   and fall back to full redraw. Hidden apps sleep; static UI does no perpetual work.
4. Compare original targets without changing them: 1080p/60 CPU p95 <8 ms, p99 <16 ms;
   guest input-to-submit p95 <25 ms; resource update <500 ms. Explain measured target
   misses and implement bounded remediation with a before/after correctness oracle.

## Acceptance criteria

- [ ] Real guest stage/latency/frame/drop/idle distributions are reproducible on
  exact profiles with clear clock, sampling and observer-cost limitations.
- [ ] Optimized output matches full redraw across overlap/translucency/filter/scale/
  caret/scroll/reload cases; no stale pixels, invisible hit regions or cache UAF.
- [ ] Static/hidden desktop behavior satisfies locked idle-work/wakeup bounds;
  clock/caret events remain counted and input/control progress stays bounded.
- [ ] Memory traffic, caches and old/new publication costs are reported with CPU
  and elapsed times separated; no host preview or RAM-only result is a guest pass.
- [ ] Each target is labeled met/missed/unmeasurable with evidence and next bounded
  work for misses; unresolved correctness/boundedness failures block completion.

## Tests and evidence

Retain p50/p95/p99/max, missed periods, seeds/sample counts, full-redraw comparisons
and baseline/remediation measurements. Include debug instrumentation cost and the
actual backend's scanout/VSync limits. Do not drop slow samples or silently reduce
rich quality to manufacture a pass. Whole-guest memory/stability belongs to P4-18.

## Exclusions and handoff

Valid measurements do not automatically accept all R09 targets. P4-19 must retain
misses; later desktop work inherits explicit findings. Read
[performance accounting](../../02-architecture/observability-and-performance.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
