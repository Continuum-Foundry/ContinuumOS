# P2-05 — Build the bounded custom Rust UI candidate

Issue type: Story. Epic: P2. Status: Not started. Owner: Unassigned.
Depends on: P2-04 Done. Suggested labels: `phase-2`, `custom-ui`, `comparison`.
Backlog: B07/B08 custom candidate, B36. Requirements: R08/R09/R15/R18;
hosted R04 feasibility. Test families: hosted T05/T09/T10.

## User story

As a toolkit reviewer, I need a small original UI candidate that exercises the
reference workflows so its benefits and maintenance costs can be measured fairly.

## Implementation scope

1. Implement only the Rust builder/layout/paint primitives needed by P2-02's scenes:
   bounded row/column/overlay/scroll/list, text, action controls, clips, rounded
   geometry, image/glyph resources, alpha and the declared effect/fallback.
   Keep domain state and typed actions in the shared P2-03 model.
2. Use stable IDs, separate layout/paint/semantic invalidation and bounded virtual
   list realization. Submit software pixels through the common buffer lifecycle;
   preserve stride/color/alpha policy and release buffers before reuse.
3. Connect keyboard, pointer, focus/capture, text input and semantic nodes/actions
   to the common fixtures. Use P2-04's declared text pipeline and font assets.
4. Render all comparison scenes at the epic profiles. Include theme/spacing changes,
   a changed layout with stable IDs and a compiled action-behavior variant for the
   later reload-path experiment. Do not claim safe native code unloading.
5. Expose comparable layout/raster/submission, damage, allocation/cache and idle
   counters. Add an adapter/hook inventory and record first-party implementation
   size, dependency features, diagnostics and missing work.

## Acceptance criteria

- [ ] All reference scenes and scripted workflows run against the shared model,
  fonts, surfaces and action API; no easier substitute corpus is used.
- [ ] Required paint/layout/text/semantics results meet the fixed comparison rules
  or have explicit failing cases retained for the selection gate.
- [ ] List realization and caches remain bounded as logical item count grows;
  stable IDs, buffer ownership and malformed geometry/resource failures are checked.
- [ ] Rich-effect and reduced-effects paths expose equivalent controls and actions;
  visual limitations and actual implementation effort are recorded.
- [ ] Instrumentation, source/asset identities and the portable/host dependency split
  are sufficient for P2-07/P2-08 to reproduce and compare the candidate.

## Tests and evidence

Run deterministic paint fixtures, clipping/translucent overlap, old/new bounds,
list scroll/selection, keyboard focus and the text corpus. Exercise duplicate IDs,
missing images/fonts, exhausted buffer/cache budgets and a withheld buffer release.
Keep a scene/feature outcome table, selected images, action/state traces and raw
counters. Missing features remain visible; they do not become accepted product support.

## Exclusions and handoff

No full widget suite, custom programming language, real guest compositor or assumed
guest allocator. This is one bounded experiment, not a preselected production path.
P2-07 receives actual portable code/hook requirements; P2-08 receives the comparable
build, fixtures and unresolved findings. Sources:
[UI comparison](../../01-research/ui-toolkit-comparison.md),
[ADR-0008](../../04-decisions/0008-rendering-path.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
