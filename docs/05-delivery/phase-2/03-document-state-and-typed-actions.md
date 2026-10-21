# P2-03 — Build the shared document state and typed-action model

Issue type: Story. Epic: P2. Status: Not started. Owner: Unassigned.
Depends on: P2-02 Done. Suggested labels: `phase-2`, `editor-state`, `typed-actions`.
Backlog: B09 state subset, M2 design of B37. Requirements: R19 design, R04/R15/R18;
preparatory R07/R14. Test families: hosted T05/T14 and T07 state fixtures.

## User story

As a developer editing unsaved work, I need a toolkit-independent state owner and
typed operations so UI rebuilding cannot recreate or discard my document.

## Implementation scope

1. Build a portable Rust model for document ID/revision, UTF-8 contents, logical
   selection/caret anchors, undo/redo history, scroll, search and dirty state. Keep
   window geometry/focus, preferences and transient gestures explicitly separate
   from persistent document state and toolkit object addresses.
2. Define a minimal edit/settings action set sufficient for the scenes: insert,
   delete, select, undo/redo, search, and theme/text-size/reduced-motion changes.
   Route button, shortcut, semantic action and scripted automation to one executor.
3. Specify action versions, schemas, expected revisions, request IDs, effect class,
   validation, cancellation/deadline outcomes and a bounded duplicate-result policy.
   Add scoped-context descriptors with provenance, source revision, expiry and size.
4. Use a hosted authority/provider model to exercise invalid/stale input and avoid
   accidental UI/automation bypass. Document that modeled IDs are not kernel-backed
   capabilities and authority/replacement enforcement requires M3/M5.
5. Add an explicit versioned logical state snapshot/restore fixture for comparison
   and future handoff. Serialize fields, lengths and IDs explicitly, with bounds;
   never serialize native layouts, raw toolkit objects, callbacks or pointers.
6. Update the action/state drafts and fixtures together. Keep interface and state
   schema versions independent and record unsupported operations honestly.

## Acceptance criteria

- [ ] All scene mutations use the same versioned executor from UI/shortcut/automation
  adapters and produce equal contents, settings, revisions and declared outcomes.
- [ ] Model state survives view destruction/recreation and compatible snapshot
  roundtrip, including unsaved text, selection, undo history and dirty status.
- [ ] Invalid input, stale revision, expired context, duplicate/unsupported requests
  and denied modeled rights have deterministic outcomes without unauthorized edits.
- [ ] Checkpoint bounds/version failures leave the active model intact; no native
  memory layout or toolkit identity becomes a wire/state contract.
- [ ] R19 is explicitly an M2 interface/model result; provider replacement, secure
  handles, IPC fencing and full B37 completion remain later gates.

## Tests and evidence

Replay the same action sequence through button, shortcut and scripted entry points.
Cover invalid UTF-8 boundaries, undo after selection replacement, revision conflict,
duplicate retry, unknown versions, fabricated context, expiry and oversize/truncated
snapshots. Retain before/after state hashes and structured expected outcomes, using
synthetic document contents. Text shaping/bidi conformance belongs to P2-04.

## Exclusions and handoff

No general workflow engine, AI, real files, durability, cross-process authority or
app replacement coordinator. P2-04 and both candidate adapters receive the shared
state/action API and conformance fixtures. Sources:
[actions contract](../../06-contracts/actions-and-context.md),
[ADR-0015](../../04-decisions/0015-typed-actions.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
