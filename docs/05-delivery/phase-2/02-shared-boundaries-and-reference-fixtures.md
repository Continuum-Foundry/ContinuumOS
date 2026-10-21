# P2-02 — Define shared UI boundaries and reference fixtures

Issue type: Story. Epic: P2. Status: Done. Owner: Codex (local implementation/self-review).
Depends on: P2-01 Done. Suggested labels: `phase-2`, `ui-contracts`, `fixtures`.
Backlog: B07/B08 design, B36 fixtures. Requirements: R04/R08/R15/R18; early R14.
Test families: hosted T05/T10/T13; contract preparation only.

## User story

As a UI engineer, I need one bounded scene and interaction specification so both
candidates solve the same product problem and can later connect to our guest.

## Implementation scope

1. Specify portable surface geometry, RGBA8/premultiplied alpha, stride, color/blend
   policy, acquire/commit/release ownership, resource generations and bounded damage.
   Keep host window/presentation calls behind an adapter. Slint may submit buffers;
   exporting the custom retained scene is not mandatory.
2. Specify physical key/text/composition separation, pointer coordinates/revisions,
   focus/capture, stable logical widget/semantic IDs, timestamps and event ordering.
   Define bounded overflow/cancellation and finite geometry/count limits.
3. Define versioned token and reloadable layout-data drafts: semantic colors, fonts,
   spacing, radii, effects and motion. Describe allowed data operations and stable
   action references; executable callbacks/native pointers are never asset fields.
4. Fix an editor/sidebar/search scene with unsaved text, a 10,000-item virtual file
   list, overlapping surfaces and launcher/settings panel. Use synthetic data,
   deterministic seeds, the same fonts/text and a scripted keyboard/pointer workflow.
   Add clipped/translucent content, one rich effect and reduced-effects parity.
5. Define screenshot geometry/paint tolerances and independent semantic/state
   assertions at all epic scales, high contrast and reduced motion. Include missing
   assets, long labels, complex scripts and invalid data. Put authored fixtures in
   source-controlled test directories, not generated evidence or ignored work paths.
6. Reconcile drafts with existing surface/action/token specifications and schemas
   if affected. Document limits and schema versions without freezing process ABI IDs.

## Acceptance criteria

- [x] Both candidate stories have identical scene/data/input/profile fixtures and
  predeclared visual/semantic acceptance rules, including the rich-effect case.
- [x] Buffer ownership, color policy, bounds, stable IDs and input ordering have
  explicit contracts and fixtures independent of toolkit internals.
- [x] Layout data contains only validated values/IDs with an explicit supported
  subset; unknown versions, cycles and excessive complexity have rejection rules.
- [x] Golden snapshots and semantic assertions are distinct; a screenshot cannot
  satisfy a focus/action/state requirement.
- [x] Contract/spec/schema changes are consistent and existing documentation/schema
  checks pass; no guest IPC implementation or permanent ABI is claimed.

## Tests and evidence

Validate fixtures for overflow/invalid stride, nonfinite geometry, duplicate IDs,
missing resources, stale scene revisions and excessive node depth/count. Demonstrate
deterministic fixture generation and exact hashes. Record the corpus/profile
manifest and unresolved adapter questions for both candidates.

## Exclusions and handoff

No complete compositor, new UI language, production widget catalogue or guest
transport. P2-03 receives logical state/action boundaries; P2-05/P2-06 receive the
same product corpus. Sources: [UI design](../../02-architecture/ui-framework-and-design.md),
[surface contract](../../06-contracts/ui-and-scene-protocol.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).

Implementation: [shared draft contract](../../06-contracts/hosted-ui-fixtures.md),
[source fixtures/lock](../../../fixtures/m2/ui-v1/README.md),
[reproduction runbook](shared-fixtures-runbook.md), and
[accepted local review](../../../evidence/phase-2/P2-02/2026-10-21/README.md).
Seven positive inputs, 56 negative mutations, two checked-overflow cases, three
nonfinite parse cases, independent schema checks and two exact deterministic
generations pass. This is a data-contract/corpus acceptance; it supplies future
golden rules, not rendered screenshots or executed UI/action/reload results.
