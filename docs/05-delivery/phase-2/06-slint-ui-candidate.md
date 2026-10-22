# P2-06 — Build the bounded Slint UI candidate

Issue type: Story. Epic: P2. Status: Done. Owner: Unassigned.
Depends on: P2-04 Done. Suggested labels: `phase-2`, `slint`, `comparison`.
Backlog: B07/B08 Slint candidate, B36. Requirements: R08/R09/R15/R18;
hosted R04 feasibility. Test families: hosted T05/T09/T10.

## User story

As a toolkit reviewer, I need the same workflows implemented with Slint so a
reusable toolkit can compete on evidence with the custom candidate.

## Implementation scope

Common locked input: [P2-02 corpus](../../../fixtures/m2/ui-v1/README.md) and
[contract](../../06-contracts/hosted-ui-fixtures.md). Use the exact same corpus/
profile/font locks, 30 configurations, five capture checkpoints and independent
state/semantic rules as P2-05. Use P2-04's [locked fonts/text corpus](../../../fixtures/m2/text-v1/README.md)
and [text contract](../../06-contracts/hosted-text.md), including every case at all
three scales. Report shaping, bidi, fallback, composition and raster outcomes
separately. Audit Slint's own pipeline and every adapter difference; no unspecified
host fallback may silently supply coverage absent from the locked fonts.

1. Inspect the locally pinned Slint candidate and current authoritative API/license
   material before relying on it. Record exact version, checksums, feature closure,
   host build/compiler dependencies and shipped runtime dependencies separately.
   Any dependency acquisition follows the repository's existing handling rules.
2. Build the P2-02 scenes using the same P2-03 state/action model and P2-04 fonts and
   script corpus. Use software rendering and a small platform/window adapter for
   the shared surface/input boundary. Do not depend on guest winit/LinuxKMS/browser
   services or require Slint to export the custom SceneCommit tree.
3. Identify allocation, time/event-loop, font, input, clipboard and semantics hooks;
   map toolkit focus/semantic identities to logical IDs. Audit single-thread-only
   assumptions and keep rendering callbacks outside IRQ/kernel execution.
4. Test hosted token/layout pathways, including interpreter-backed preview only if
   supported by the exact configuration. Keep hosted interpreter, hosted compiled
   UI and freestanding AOT feature sets distinct. Include a compiled behavior variant
   for comparison; preview is not evidence of native behavior replacement.
5. Add the same counters and reference scenarios as the custom candidate. Inventory
   effect/script/semantic limitations, adapter effort and a concrete completion path.
   Record intended OS/API distribution and applicable license options/open questions;
   do not infer license suitability merely from use on a custom OS.

## Acceptance criteria

- [x] Slint renders and operates the shared corpus through the common model/surface
  boundary using pinned software-renderer inputs and explicit host-only adapters.
- [x] Text, focus, semantics, scale, rich-effect and fallback outcomes use the same
  rules as the custom candidate; unsupported cases remain in the comparison table.
- [x] Hosted interpreter/AOT and proposed guest feature configurations are separately
  identified with no transitive host dependency mislabeled as guest-compatible.
- [x] Tokens/layout and compiled-behavior pathways are identified with real API tests
  or recorded failures; no guest interpreter or live replacement result is implied.
- [x] Metrics, platform hooks, maintenance cost and distribution-term questions are
  reproducible inputs to P2-07/P2-08 and the provisional selection gate.

## Tests and evidence

Run identical scene/input/text/semantics fixtures, missing resources, invalid layout,
focus removal and bounded list/caches. Retain feature graphs, platform hook mappings,
selected images, raw counters and adapter failure traces. State when semantics are
toolkit-native, adapter-derived or unavailable; screenshots cannot fill that gap.

## Exclusions and handoff

The [local review](../../../evidence/phase-2/P2-06/2026-10-21/README.md) accepts
bounded comparison evidence with explicit failing native features. Slint 1.18.1
AOT/software primitives use the shared P2-04 text Image adapter. Native text setup,
rich blur, rounded ancestor clips, launcher opacity and fractional blend arithmetic
remain comparison failures; focus/semantics are shared adapter-derived. The hosted
interpreter is unavailable offline. Use the [runbook](slint-ui-runbook.md).
No product requirement is waived, distribution term selected, guest link/reload
result claimed or toolkit chosen.

No full Slint port into the kernel, guest runtime claim or automatic license choice.
If the candidate is infeasible, preserve a bounded reproducible rejection for P2-09;
do not silently weaken requirements to obtain a comparison pass. Sources:
[UI comparison](../../01-research/ui-toolkit-comparison.md),
[ADR-0011](../../04-decisions/0011-ui-authoring.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
