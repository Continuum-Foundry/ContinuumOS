# P4-05 — Broker surface leases and compose guest windows

Issue type: Story. Epic: P4. Status: Not started. Owner: Graphics/Services.
Depends on: P4-04. Backlog: B16 compositor.
Requirements: R08/R09/R10/R14/R17. Test families: T03/T05/T06/T09/T11.
Labels: `phase-4`, `surfaces`, `compositor`.

## User story

As an app, I need scoped surface ownership and bounded composition so I can submit
pixels without accessing another window or modifying buffers still in use.

## Implementation scope

1. Encode surface creation, acquire/write/commit/in-use/release, geometry, damage,
   resource generation and errors explicitly. Bind surface/lease generations to the
   trusted app/session and output revision. Independently test codecs and stale IDs.
2. Enforce writer/read exclusion through supported mappings/ownership. A checked
   copy into compositor-owned storage is an acceptable baseline; a submitted mutable
   user buffer cannot race the reader. Handle duplicates, outstanding leases and death.
3. Implement bounded user-mode software composition, z-order, clips, alpha/format
   conversion and window old/new exposure. Clear newly exposed pixels. Initially
   allow full redraw; damage optimization belongs to P4-17 with a full-redraw oracle.
4. Admit all surface/scratch/scanout bytes before commit, preserve the prior frame
   on rejection, and separate frame opportunities from release fences. Define bounded
   compositor failure behavior and serial recovery without claiming crash continuity.

## Acceptance criteria

- [ ] Two independently built guest clients compose overlapping opaque/translucent
  surfaces with reference-correct pixels and actual display captures.
- [ ] Unauthorized/wrong-session/stale-generation commits, malformed stride/damage
  and over-budget acquisitions deny without changing the last valid frame.
- [ ] In-use buffer write/reuse, duplicate release and owner death have enforced,
  bounded outcomes; exact reference/page/lease inventories settle after teardown.
- [ ] Moving, hiding and removing windows repairs exposed regions without stale
  private pixels or alpha corruption; submission and release ordering are explicit.
- [ ] Independent codec/authority oracles and affected memory/IPC/runtime regressions
  pass; no native Rust struct crosses the boundary.

## Tests and evidence

Stress queue/lease pressure, checked size arithmetic, forged owners, partial client
copies, death during commit, stalled presentation and allocation failures. Compare
composed output with an independent reference and retain actual display proof.

## Exclusions and handoff

No mandatory SceneCommit implementation, zero-copy promise or safe compositor code
replacement. P4-06 receives surface transforms and P4-07 receives a buffer API.
Read [surface contract](../../06-contracts/ui-and-scene-protocol.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
