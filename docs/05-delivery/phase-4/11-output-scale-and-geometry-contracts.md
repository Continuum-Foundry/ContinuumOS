# P4-11 — Validate single-output display contract foundations

Issue type: Story. Epic: P4. Status: Not started. Owner: Graphics/SDK.
Depends on: P4-10. Backlog: B61.
Requirements: R08/R09/R10/R14/R15/R18. Test families: T02/T05/T06/T10/T11/T13.
Labels: `phase-4`, `display-contracts`, `scaling`.

## User story

As a future display client, I need single-output identity and coordinate contracts
that can grow to M22 without conflating logical windows, scanout buffers and authority.

## Implementation scope

1. Encode output identity/generation, pixel extent, logical origin, rational scale,
   format/stride, mode/topology revision and surface association explicitly. Return
   unsupported for unimplemented enumeration/change operations; freeze only tested
   fields after independent clients/fixtures, keeping future display extensions open.
2. Check signed coordinates, rational scaling/rounding, clipping and size/stride/
   allocation arithmetic. Separate logical document/window identity from physical
   buffer generation; never treat saved output IDs as current grants.
3. Exercise one output at 720p/1080p and declared 100/150/200% logical scale through
   separately configured sessions if live scale change is unavailable. Compare paint,
   pointer/capture/caret and semantic transforms using independently authored points.
4. Enforce output/surface authority and stale revision rejection across recreated
   test outputs. Check old-reader release and failed candidate admission preserve
   the last usable configuration; no claim of physical hot-plug follows.

## Acceptance criteria

- [ ] Independent producer/consumer fixtures validate explicit encodings, versions,
  checked arithmetic and unsupported mandatory extensions without native layouts.
- [ ] Actual single-output guest paint/hit-test/caret/semantic coordinates agree at
  both resolutions and all declared scales, including boundaries and clipping.
- [ ] Stale output/lease/revision, forged mode/capture authority, overflow and
  insufficient headroom deny while the previous valid output stays usable.
- [ ] Global accounting counts shared buffers once; logical surfaces and physical
  backing/output generations have documented distinct lifetimes.
- [ ] B61 handoff lists exact supported fields and M22 obligations without claiming
  4K, dual output, live mode switching, hot-plug or independent-refresh delivery.

## Tests and evidence

Use fractional edge/rounding, negative/offscreen coordinate fixtures, oversized
stride, removed/recreated output generations and independent codec mutations.
Retain guest captures and semantic/input observations separately from host models.

## Exclusions and handoff

No extra M22 release dependency is added to M4. P4-15 consumes stable surface/output
identities; later settings/SDK/backup/assistive clients consume explicit coordinates.
Read [display roadmap](../display-roadmap.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
