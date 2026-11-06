# P4-08 — Integrate the minimal shell and editor workflow

Issue type: Story. Epic: P4. Status: Not started. Owner: Desktop/UI.
Depends on: P4-07. Backlog: B08/B16 shell.
Requirements: R04 preparation/R08/R10/R15/R17. Test families: T01/T05/T10/T11.
Labels: `phase-4`, `shell`, `editor`.

## User story

As a developer using the guest, I need a keyboard-complete editor and minimal
window shell so live resource changes can be evaluated against real unsaved work.

## Implementation scope

1. Compose a minimal launcher, editor, synthetic file-list/settings/reference
   surfaces and bounded diagnostics using the selected UI. Declare ephemeral files
   honestly; no Save action may imply durable storage that does not exist.
2. Implement window creation/focus/z-order, move/resize, hide/minimize/restore and
   close policy. Keep shell geometry/window IDs owned independently from app model
   and resource generations; constrain unreachable windows and modal focus traps.
3. Wire editor insert/delete/selection/undo/search and bounded list navigation
   through typed model operations. Preserve unsaved text, history, scroll and selection
   as observable state; do not reconstruct them from screenshots.
4. Supply visible focus, keyboard equivalents for required actions, high contrast,
   reduced motion and supported scales. Keep a stable serial diagnostic path when
   shell/app crashes; explicitly report unrecoverable unsaved work after app death.

## Acceptance criteria

- [ ] A freshly booted exact ISO supports launcher-to-editor, unsaved typing,
  selection/search/undo, keyboard window navigation and pointer move/resize.
- [ ] Focus/capture, overlap/clipping, hidden-window input and close-with-unsaved-work
  policies match independent workflow expectations with no unauthorized routing.
- [ ] Required workflows remain keyboard-complete with visible focus in contrast/
  reduced-motion/scale profiles; UI text describes ephemeral persistence accurately.
- [ ] Shell/app error is distinct from kernel failure and reload success; recovery
  controls/serial remain available with documented work-loss limits.
- [ ] Exact logical checkpoints, window identities/geometry and captures provide
  a reproducible pre-reload baseline for P4-15/P4-16.

## Tests and evidence

Run device-driven workflows including active drag, held modifier, text composition
fixture, focus removal, minimize/restore and app crash. Retain expected versus actual
model/history/geometry values separately from visual and semantic artifacts.

## Exclusions and handoff

No real file browser/storage, app installation, integrated clipboard/notifications
or desktop completeness claim. P4-09 receives stable window/widget/model identities.
Read [UI design](../../02-architecture/ui-framework-and-design.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
