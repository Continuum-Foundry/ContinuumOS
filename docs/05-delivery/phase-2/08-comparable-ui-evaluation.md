# P2-08 — Compare visual quality, reload paths and hosted resource cost

Issue type: Story. Epic: P2. Status: Done. Owner: Codex (local implementation/self-review).
Depends on: P2-05, P2-06 and P2-07 Done.
Suggested labels: `phase-2`, `comparison`, `performance`, `visual-review`.
Backlog: B36, M2 subset of B32. Requirements: R04/R08/R09/R15/R18; R19 design.
Test families: hosted T05/T07/T09/T10/T14 subset.

## User story

As a project reviewer, I need comparable product and engineering evidence so the
provisional toolkit choice is based on quality, continuity and cost together.

## Implementation scope

1. Run both optimized candidates with identical inputs, fonts, sizes/scales, software
   paths, warm-up and sample counts from P2-01/P2-02. Record failed/unsupported cases;
   do not drop an expensive effect or complex-script scene from one candidate only.
2. Review typography, clipping, scaling, hierarchy, focus, keyboard-only completion,
   high contrast/reduced motion, semantic names/actions and text corpus results.
   Record image differences and functional assertions separately.
3. Measure frame/layout/raster/submission p50/p95/p99, deadline misses, input-to-host
   submission, damaged area, idle wakeups/CPU, allocations and resident/committed/peak
   bytes. Account for surfaces, font caches, runtime and adapter copies; isolate
   compiler/interpreter overhead and trace overhead rather than hiding them.
4. Exercise independent token, layout and compiled behavior changes. Record changed
   output and state continuity or failures. For a hosted native replacement experiment,
   keep the state owner/controller alive and use bounded helper processes with explicit
   snapshot/restore; record child restarts and preserve state on candidate failure.
   Never unload reachable native code or present a process restart as L0/L1 continuity.
5. Compare authoring/adapter effort, diagnostics, missing hooks, CPU fallback, future
   GPU path assumptions, state export and distribution terms. Combine P2-07's actual
   target results with hosted evidence in every axis of the existing comparison matrix.

## Acceptance criteria

- [x] Each matrix axis has exact comparable evidence or a precise failure/unsupported
  outcome for both candidates, including required text, effects and semantics.
- [x] Quality review and keyboard/action results are recorded independently of image
  snapshots; reduced-effects mode preserves the tested workflows.
- [x] Raw timing/memory/idle/damage samples, distributions, configuration and accounting
  method are retained; host cost is never called guest performance or photon latency.
- [x] Token, layout and compiled native behavior change results distinguish resource
  continuity, helper restart and unsupported paths with exact state/build identities.
- [x] A reviewable recommendation lists material risks, maintenance/rights questions
  and bounded fixes; no guest reload or complete R04/R05/R19 claim follows.

## Tests and evidence

Use fixed idle, typing/scroll, large-list, drag/resize, overlap/effect and reload
scripts. Include invalid input/resources, a failed behavior build/start/restore,
removed focus target and composition/gesture cases. Record pauses and state deltas.
Stop a runaway candidate at the predeclared deadline and retain its failure, rather
than increasing the timeout after the fact. Full images/traces remain local; select
compact comparison tables, representative images and necessary raw samples.

## Exclusions and handoff

Local comparison acceptance: [2026-10-22 reviewed bundle](../../../evidence/phase-2/P2-08/2026-10-22/README.md).
Use the [comparison runbook](comparable-ui-runbook.md). The complete 20-block
experiment retains 6,000 frames, 12,000 process samples, 100 alternating token/layout
cycles per candidate and exact snapshot-restoring helper variants/failures.
Custom frame-budget misses, Slint shown-component retention, fidelity/terms gaps
and unavailable native clock/caret/drag/assistive paths are findings, not waivers.
The recommendation is conditional custom consideration at P2-09; no toolkit is
selected, actual hosted resource loader completed or guest UI/reload demonstrated.

No toolkit selection from a single screenshot or mean FPS, and no requirement to
implement the M5 transaction coordinator for this experiment. P2-09 receives the
complete comparison and unresolved issues. Sources:
[comparison matrix](../../01-research/ui-toolkit-comparison.md#comparison-matrix),
[performance specification](../../02-architecture/observability-and-performance.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
