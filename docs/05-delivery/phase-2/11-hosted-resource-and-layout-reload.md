# P2-11 — Publish hosted token and layout reloads without losing state

Issue type: Story. Epic: P2. Status: Done. Owner: Unassigned.
Depends on: P2-10 Done. Suggested labels: `phase-2`, `resource-reload`, `continuity`.
Backlog: B10, hosted R04. Requirements: R04/R08/R15/R18; early R07/R17/R19.
Test families: hosted L0/L1 portions of T05/T07/T10/T14.

P2-09 handoff: use the [provisional custom core and required completion list](ui-selection-review.md#required-completion-and-reconsideration)
with the [exact selection inventory](../../../legal/ui-selection.json). Slint is
frozen comparison material. No required M2 behavior moves to a later guest gate.

## User story

As an OS UI developer, I need valid theme/layout edits to appear in the running
hosted preview while my unsaved document, selection, focus and window survive.

## Implementation scope

1. Implement a bounded host resource builder/validator and update trigger with
   debounce/coalescing. Hash complete immutable token/layout/asset generations;
   validate versions, base generation, references, finite geometry and complexity
   before publication. Failed or partial writes/builds never become visible.
2. Stage resources within an explicit headroom limit and publish the whole supported
   L0/L1 generation at an event/scene boundary. Keep old resources alive until all
   readers/buffers release them; avoid mixed layout/paint/semantic generations.
3. Reconcile stable IDs without recreating the shared document owner. Preserve
   unsaved text/revision, selection/caret, undo/redo, dirty state, scroll, search,
   compatible focus and hosted window geometry. Deliberately move focus if its
   control disappears; never silently drop accepted edits.
4. Apply P2-10's finish/transfer/cancel rules for gestures/composition and ordered
   bounded input during reload. Surface explicit rejection/cancellation reasons.
   Settings and semantic/action bindings remain consistent with the committed view.
5. Record build/request/generation IDs and receive/validate/stage/publish/first-frame
   timestamps. Separate resource-build cost from completed-build-to-visible latency;
   query the active generation after interruption/retry instead of assuming success.
6. Keep the same host preview process/session alive through successful L0/L1 reload.
   If the toolkit requires compiled replacement for a layout edit, classify it as
   unsupported in this L1 data subset and supply a real reloadable layout-data path
   for M2; do not disguise a restart as resource reload.

## Acceptance criteria

- [x] Visible token and supported structural layout changes occur in the same running
  host process, with monotonic host time and independent process-lifecycle evidence.
- [x] Before/after state assertions prove unsaved text, revision, selection, undo,
  scroll, dirty state, compatible focus and window geometry remain correct.
- [x] Invalid/unknown/stale/oversized content, missing assets and failed builds leave
  the previous complete generation visible and editable with a precise error.
- [x] Reload during typing, drag/capture and composition follows the declared policy;
  removed IDs and queue overflow cannot leave stuck modifiers/capture or lost edits.
- [x] Frame/semantic publication and resource retirement obey ownership and headroom
  rules; no in-use resource is freed and no unbounded old-generation list remains.
- [x] Trace and UI status identify actual generations/outcomes and ephemeral hosted
  scope; kernel boot/uptime, guest transport and native code replacement are not claimed.

## Tests and evidence

Demonstrate edit unsaved text → select range → reposition window → change palette,
spacing and panel structure → continue typing. Capture state and image/semantic
evidence separately. Inject truncated generation, wrong version/base, duplicate IDs,
cycles, missing fonts/images, stage exhaustion, rapid saves, failed validation and
withheld reader release. Test update mid-gesture/composition and removed focused
control. Preserve original failures and measure build/validation/publish/visible costs.

## Exclusions and handoff

No general native behavior hot swap, guest dev endpoint, process migration, durable
activation or universal layout language. P2-12 receives repeatable supported L0/L1
updates, negative fixtures, resource accounting and continuity assertions. Sources:
[UI tokens](../../02-architecture/ui-framework-and-design.md#token-system),
[live-update classes](../../03-live-update/model-and-guarantees.md#supported-classes),
[surface resources](../../06-contracts/ui-and-scene-protocol.md#resources).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation: see the [hosted resource reload runbook](resource-reload-runbook.md).
All criteria pass [local acceptance/self-review](../../../evidence/phase-2/P2-11/2026-10-23/README.md): ten prerequisites, nine guarded commands, 21 publications, 20 negative cases and real same-window generation 7-to-8 continuity. P2-12 performance/stability and P2-13 remain pending.

## Final M2 status — 2026-10-24

The pending P2-13 work described above now passes [final local hosted
acceptance](../../../evidence/phase-2/P2-13/2026-10-24/README.md), including the clean selected retained/cache link.
The earlier story review remains scoped to its original run. Guest execution,
efficiency targets and M3/M4/M5 obligations remain open.
