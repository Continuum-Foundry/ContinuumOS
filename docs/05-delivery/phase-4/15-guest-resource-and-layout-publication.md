# P4-15 — Publish coherent guest L0/L1 generations

Issue type: Story. Epic: P4. Status: Not started. Owner: Updates/UI/Graphics.
Depends on: P4-11,P4-14. Backlog: B17.
Requirements: R04/R08/R10/R13/R14/R15/R17. Test families: T05/T06/T07/T10/T11.
Labels: `phase-4`, `resource-reload`, `publication`.

## User story

As a developer, I need a host-built theme/layout change to become visible in the
running guest while my unsaved document and window remain owned by the same app.

## Implementation scope

1. Build versioned L0 token/asset and L1 layout generations on the host, transfer
   and prepare through P4-14, and publish one complete immutable root at a scene
   boundary after current base/session/policy revalidation. No mixed generations.
2. Preserve app process, model/history/document revision, selection/grapheme affinity,
   window ID/geometry/stacking, scroll, focus and compatible stable widget IDs.
   Resource generation, scene revision and model revision remain distinct.
3. Reconcile input/semantic state coherently: complete/cancel active gestures,
   apply declared composition policy, bound queued input and avoid stale hit-test
   activation. Removed focus/capture/semantic IDs have explicit fallback/cancellation.
4. Retire old data only after layout/raster/compositor/presentation readers release.
   Bound retained generations/leases; stalled readers backpressure or abort further
   staging rather than free live resources. Publish truthful ephemeral status/ACKs.

## Acceptance criteria

- [ ] Host source edits produce both visible L0 and L1 changes in the actual guest
  through the authenticated channel, preserving the same app/window/document.
- [ ] Independent complete logical checkpoints, history, selection, focus and window
  assertions pass alongside actual display/semantic observations.
- [ ] Boot identity/continuous monotonic uptime and host VM lifecycle prove no reboot,
  process replacement, snapshot restoration or hidden helper activation.
- [ ] Corrupt/incompatible/unauthorized/stale candidates leave the previous complete
  generation usable; no partial paint/input/semantic publication or false success.
- [ ] Old/new reader/lease inventory and delayed-release tests prove safe bounded
  data retirement without making an M5 code-retirement claim.

## Tests and evidence

Replay typing/search/selection, active drag/key-up, compatible and removed focus,
composition fixtures, delayed render readers and changed scale/clip/layout. Retain
host build/transfer/prepare/commit/display observations and exact artifact identities.
Timing estimates await P4-17's locked measurement method.

## Exclusions and handoff

No app behavior/state migration, restart-based continuity, persistent activation or
universal rollback. P4-16 owns transition-fault/retry correctness. Read
[live-update guarantees](../../03-live-update/model-and-guarantees.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
