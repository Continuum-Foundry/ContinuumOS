# P2-10 — Complete selected hosted layout, input and semantic behavior

Issue type: Story. Epic: P2. Status: Done. Owner: Codex local implementation/self-review.
Depends on: P2-09 Done. Suggested labels: `phase-2`, `layout`, `input`, `semantics`.
Backlog: M2 portions of B07/B08/B09/B37. Requirements: R08/R15/R18; R19 design.
Test families: hosted T05/T10/T14 subset.

P2-09 handoff: use the [provisional custom core and required completion list](ui-selection-review.md#required-completion-and-reconsideration)
with the [exact selection inventory](../../../legal/ui-selection.json). Slint is
frozen comparison material. No required M2 behavior moves to a later guest gate.

Implementation and reproducible commands: [selected hosted core runbook](selected-ui-runbook.md).

## User story

As a keyboard and pointer user, I need predictable layout, focus and semantic
actions in the selected UI so later reload preserves a usable application.

## Implementation scope

1. Close the selected path's required M2 gaps from P2-09 while preserving the shared
   model/surface boundary. Stabilize logical IDs and bounded measure/arrange/paint/
   semantic invalidation; a paint-only change must not force unrelated full layout.
2. Complete reference scene resize/scroll, list virtualization, clipping and hit-test
   transforms. Keep window geometry as hosted shell/controller state; do not imply
   that a toolkit supplies OS window authority or a real guest compositor.
3. Finish key/text/composition separation, modifier release, keyboard navigation,
   visible focus, pointer capture and modal dismissal. Define behavior when controls
   disappear or move, input references stale geometry, or a queue saturates.
4. Expose semantic roles, names, values, state, actions, bounds and logical hierarchy
   independently of paint nodes. Include virtual list count/navigation, focus changes
   and notifications needed by the corpus; route actions through P2-03's executor.
5. Close declared text/edit/scale/high-contrast/reduced-motion cases. Use toolkit
   adapters or a hosted semantic test consumer as needed; record native assistive
   integration limits rather than equating an inspectable tree with a screen reader.

## Acceptance criteria

- [x] All mandatory selected-path reference scenes, text cases and keyboard workflows
  pass at the declared sizes/scales, with stable focus and visible non-color cues.
- [x] Invalidation distinguishes layout, paint and semantics; list realization,
  recursion, event queues and resource counts remain within declared bounds.
- [x] Hit testing, capture, modifier release, composition and focus retirement have
  deterministic policies that prevent stuck input or activation of a stale target.
- [x] Semantic roles/names/actions/state and virtual list navigation pass independent
  assertions; semantic actions and UI/shortcut actions have the same model outcome.
- [x] Missing resources, malformed geometry and pressure fail honestly without losing
  accepted document edits; no native assistive or guest isolation guarantee is claimed.

## Tests and evidence

Run keyboard-only editor/search/settings/list tasks, resize and fractional-scale
hit tests, removed focused/captured controls, pointer-up/key-up across scene changes,
composition commit/cancel and queue saturation. Check long/RTL labels, contrast,
reduced motion and invalidation counters. Retain state/semantic diffs and selected
paint comparisons with all remaining limitations explicitly recorded.

## Exclusions and handoff

No broad desktop feature catalogue, native input driver, IME service, screen reader
or real file browser. P2-11 receives a working selected hosted core and explicit
focus/capture/composition policies for reload reconciliation. Sources:
[UI design](../../02-architecture/ui-framework-and-design.md),
[text/accessibility](../../02-architecture/accessibility-and-internationalization.md),
[surface contract](../../06-contracts/ui-and-scene-protocol.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

## Local acceptance review

All five criteria pass the [2026-10-23 local review](../../../evidence/phase-2/P2-10/2026-10-23/README.md).
This is Codex self-review, not independent approval. The selected 30-configuration
matrix, full text/keyboard/semantic and failure cases, unchanged reference corpus,
independent edge oracle and native Windows replay pass. File reload, measured
optimization, native IME/assistive integration and guest execution remain separate.
