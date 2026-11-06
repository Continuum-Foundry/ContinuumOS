# P4-07 — Run selected UI, fonts and text in guest processes

Issue type: Story. Epic: P4. Status: Not started. Owner: UI/Text/Runtime.
Depends on: P4-05,P4-06. Backlog: B07/B09/B16/B36 guest confirmation.
Requirements: R03/R08/R09/R10/R14/R15/R18. Test families: T02/T03/T05/T09/T10/T13.
Labels: `phase-4`, `toolkit`, `text`.

## User story

As a UI developer, I need the selected portable renderer and text engine to run
unchanged in purpose inside the guest, with real input and surface submission.

## Implementation scope

1. Build actual custom-ui/text-core/editor-model user images through the supported
   SDK/runtime. Replace probe startup/arena/hooks with P4-02 allocation, clocks,
   waits, checked input and surface leases; audit symbols, CPU instructions and ELF
   permissions. No host OS calls or kernel rendering shortcut.
2. Load exact pinned immutable fonts/assets through scoped bounded lifetimes.
   Verify digest/size before parsing; bound glyph/layout/raster caches and scratch.
   Keep parser failure/OOM process-local; never place arbitrary font parsing in IRQs.
3. Render the common hosted corpus in the guest, including clips, rounded edges,
   gradients, translucent overlap, scroll/list virtualization and rich/reduced effects.
   Exercise shaping/bidi/fallback/grapheme/caret/selection at declared scales.
4. Adapt the single-thread event loop to monotonic waits/frame opportunities and
   device input. Keep logical state, scene/resource identity and physical surface
   identity separate. Export a guest hook closure report for the toolkit decision.

## Acceptance criteria

- [ ] Actual packaged guest images render the selected corpus to displayed surfaces
  at 720p/1080p with fixed pixel tolerances and separate logical assertions.
- [ ] Real keyboard/pointer editing uses the portable text/model path; shaping,
  selection/caret, clipping and fallback match the declared corpus/support limits.
- [ ] Exact font/dependency/feature/license identities and runtime hook closure are
  reviewed; missing hooks fail explicitly instead of using a hosted fallback.
- [ ] Bad/oversize font/resource, cache pressure, allocation failure and UI panic
  preserve kernel/control progress with bounded reclamation and no stale lease use.
- [ ] A guest confirmation report compares the actual selected subset with M2's
  provisional obligations; unresolved correctness gaps block dependent stories.

## Tests and evidence

Keep image/ELF audits, actual guest captures, independent text/semantic expectations,
parser negatives, memory inventories and affected hosted corpus/link regressions.
Record real guest costs as observations for P4-17, not inherited host performance.

## Exclusions and handoff

No full widget rewrite, automatic Slint re-port, native assistive consumer, universal
Unicode/IME or toolkit approval from linking alone. P4-08 owns complete shell use;
P4-19 records the final bounded toolkit confirmation. Read
[selection review](../phase-2/ui-selection-review.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
