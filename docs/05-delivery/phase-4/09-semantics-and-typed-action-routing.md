# P4-09 — Export semantics and checked typed actions

Issue type: Story. Epic: P4. Status: Not started. Owner: UI/Services.
Depends on: P4-08. Backlog: B16, M4 B37 seed.
Requirements: R08/R10/R14/R15/R19 groundwork. Test families: T05/T06/T10/T11/T14 subset.
Labels: `phase-4`, `semantics`, `typed-actions`.

## User story

As a keyboard or automation client, I need an authorized semantic/action boundary
that describes the same guest UI without relying on visual pixels or privileged IDs.

## Implementation scope

1. Encode bounded semantic snapshots/deltas with stable node/parent IDs, roles,
   values/states, logical bounds, focus, supported actions and virtual-list position.
   Validate hierarchy, revisions, limits and explicit errors independently.
2. Export guest semantics through scoped service grants; exclude private surfaces
   and fields. Semantic bounds use the same layout/scroll/scale transforms as paint
   and hit testing, while remaining structurally independent from the scene tree.
3. Route buttons/shortcuts and an independently built test automation client into
   the same typed executor. Recheck current caller/session, grant scope, expected
   model revision, expiry and request identity at provider use time.
4. Test focus/action reconciliation when nodes move/retire. Reserve later M9
   grant-bearing offer/picker/notification interfaces with explicit unsupported
   responses; strings, paths and object IDs alone confer no authority.

## Acceptance criteria

- [ ] Real guest snapshots/deltas match independent role/hierarchy/value/focus/
  virtual-navigation expectations separately from screenshots.
- [ ] UI, shortcut and independent automation entries produce identical admitted
  model results; unauthorized, stale/expired and wrong-revision actions deny.
- [ ] Semantic movement/removal and scale/scroll changes update bounds and focus
  without stale-node actions or private cross-window/session observation.
- [ ] Malformed trees, over-budget deltas, duplicate IDs and invalid action data
  fail atomically; codec compatibility is tested by an independent consumer.
- [ ] The supported subset and remaining native-assistive limitations are recorded;
  no M5 replacement-parity or M13 speech/assistive claim is made.

## Tests and evidence

Keep device-driven keyboard workflows, semantic/action records and provider-side
denials. Test a second app of the same synthetic user and a different user fixture,
revocation between preview/use, duplicate requests and service restart/rebinding.

## Exclusions and handoff

No persistent account system, native screen reader or integrated later desktop
services. P4-10 closes protected-surface privacy and P4-11 checks coordinates.
Read [action contract](../../06-contracts/actions-and-context.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
