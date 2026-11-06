# P4-06 — Deliver physical keyboard and pointer input

Issue type: Story. Epic: P4. Status: Not started. Owner: Platform/Input.
Depends on: P4-03,P4-05. Backlog: B16 input.
Requirements: R08/R10/R14/R15/R17. Test families: T04/T05/T06/T11.
Labels: `phase-4`, `input`, `focus`.

## User story

As a desktop user, I need real keyboard and pointer events routed to the intended
window with no stuck keys or accidental activation after reset or layout changes.

## Implementation scope

1. Implement the selected VM input device via scoped resources and checked queues.
   Negotiate exact capabilities, map physical keys/buttons/motion and keep text,
   modifiers and physical key state distinct. Declare the initial keymap explicitly.
2. Deliver versioned, timestamped, sequenced events to a stable input broker and
   authorized focused surface. Apply current compositor transforms/hit-test revision;
   payload-selected owner/widget IDs cannot redirect input authority.
3. Implement focus and pointer capture grants/lifetimes, cancellation on death,
   hidden/removed windows and stale geometry. Overflow/reset/disconnect produce
   explicit cancellation/all-keys-up and bounded resynchronization, never silent loss.
4. Preserve responsive input/control progress under rendering/IPC pressure. Supply
   deterministic device injection through the real VM input path, plus manual replay
   instructions; direct Controller calls remain separate unit tests.

## Acceptance criteria

- [ ] Real VM key/pointer input reaches the authorized guest window with ordered
  press/release/modifier/text behavior and correct scaled hit testing.
- [ ] Focus/capture changes, device reset and queue overflow leave no stuck state
  and cannot activate a moved/removed control on stale release.
- [ ] Forged input injection, another surface's capture and wrong-session delivery
  deny through trusted caller checks.
- [ ] Device negotiation/descriptor/IRQ failures remain bounded; healthy control
  and serial diagnostics progress without a busy-poll rendering loop.
- [ ] Guest device/routing evidence and independent event/state assertions pass
  separately from synthetic model tests and affected prior regressions.

## Tests and evidence

Inject key repeat, simultaneous modifiers, press-drag-release across windows, stale
scene revisions, owner death, malformed events and missing releases. Verify actual
queue reset/cancel outcomes and bounded event drops with synthetic nonsensitive input.

## Exclusions and handoff

No native IME, persistent regional settings or protected login authority yet.
P4-07/P4-08 integrate real editing/window actions; P4-10 owns sensitive focus/privacy.
Read [input contract](../../06-contracts/ui-and-scene-protocol.md#inputevents).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
