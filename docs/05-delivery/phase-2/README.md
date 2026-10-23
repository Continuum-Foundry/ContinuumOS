# P2 epic — Hosted UI core and state-preserving resource reload

Issue type: Epic. Status: In progress. Owner: Unassigned.
Created: 2026-10-20. Milestone: M2. Suggested labels: `phase-2`, `milestone-m2`.
These are repository-local, Jira-ready stories; no Jira issues have been created.
This directory follows the [Phase 1 story format](../phase-1/README.md).

## Outcome and starting point

Compare Slint and a bounded custom Rust UI using the same scenes, document/action
model, fonts, input scripts and software-surface boundary. Select provisionally
using hosted measurements and exact freestanding compile/link evidence. Finish
the selected hosted core and demonstrate token/layout reload while unsaved text,
selection, focus and window state remain intact in the same running host process.

Source baseline: `aef1354` on main, inspected 2026-10-20. M1's
[complete local acceptance](../../../evidence/phase-1/P1-07/2026-10-20-acceptance/README.md)
passes 33 rows and 42 launches. Original Rust ISO boot, metadata, exceptions,
owned mappings and delivered-tick uptime exist. There is no general guest heap,
scheduler, user isolation, UI, persistence or guest live reload. B06/B32 remain
partial. Phase 0's candidate library compilation does not prove final linking,
platform integration, text support or toolkit suitability. Recheck actual source,
local tools and evidence availability before implementation.

## Ordered stories

| ID | Story | Depends on | Backlog coverage | Status |
| --- | --- | --- | --- | --- |
| P2-01 | [Recheck prerequisites and define hosted profiles](01-hosted-prerequisites-and-profiles.md) | P1-07 Done; Phase 0 recheck | B01/B02 follow-up, B32 setup | Done |
| P2-02 | [Define shared boundaries and reference fixtures](02-shared-boundaries-and-reference-fixtures.md) | P2-01 | B07/B08 design, B36 fixtures | Done |
| P2-03 | [Build the document state and typed-action model](03-document-state-and-typed-actions.md) | P2-02 | B09 state subset, M2 design of B37 | Done |
| P2-04 | [Establish fonts and text conformance](04-fonts-and-text-conformance.md) | P2-03 | B09, B36 text comparison | Done |
| P2-05 | [Build the minimal custom UI candidate](05-minimal-custom-ui-candidate.md) | P2-04 | B07/B08 custom candidate, B36 | Done |
| P2-06 | [Build the Slint UI candidate](06-slint-ui-candidate.md) | P2-04 | B07/B08 Slint candidate, B36 | Done |
| P2-07 | [Close freestanding integration feasibility](07-freestanding-integration-feasibility.md) | P2-05, P2-06 | B02 follow-up, B07/B36 target hooks | Done |
| P2-08 | [Compare quality, reload paths and hosted cost](08-comparable-ui-evaluation.md) | P2-05, P2-06, P2-07 | B36, M2 subset of B32 | Done |
| P2-09 | [Record the provisional UI selection](09-provisional-ui-selection.md) | P2-08 | B36 decision, B07 direction | Done |
| P2-10 | [Complete selected layout, input and semantics](10-selected-ui-layout-input-and-semantics.md) | P2-09 | M2 portions of B07/B08/B09/B37 | Not started |
| P2-11 | [Publish hosted token/layout reloads](11-hosted-resource-and-layout-reload.md) | P2-10 | B10, hosted R04 | Not started |
| P2-12 | [Measure idle, damage and reload stability](12-hosted-performance-and-reload-stability.md) | P2-11 | M2 subset of B32, B10 stability | Not started |
| P2-13 | [Run and review Phase 2 acceptance](13-phase-2-acceptance.md) | P2-01 through P2-12 | M2 gate, early B32 | Not started |

Work in this order, accepting one bounded story at a time. P2-05/P2-06 are sibling
candidate slices with the same prerequisites; numbering gives a default execution
order without making one toolkit depend on the other's code. Do not expand either
into a full production framework before P2-09. Blocked dependencies remain visible.

## Fixed scope and defaults

- Native Windows hosted executable and headless deterministic fixtures; first-party
  portable UI/domain code in Rust, separate from the kernel. A host window adapter
  may use host facilities, with those dependencies kept outside the portable core.
- Software pixels through a common surface/input boundary. A toolkit's private
  tree may remain private; the custom SceneCommit format is optional. No browser
  engine, guest GPU API, guest interpreter or guest compiler is assumed.
- Same editor/sidebar/search, large virtual file list, overlapping surfaces and
  launcher/settings scenes for both candidates; synthetic files do not imply a
  guest filesystem. Include a complex-script case and a rich effect with a visible,
  functional reduced-effects path. OS window/compositor authority remains ours.
- Start at 1280×720 and 60 Hz, record 1920×1080 comparison separately, and test
  logical scales 1.0/1.5/2.0. Fix fonts, data, seeds and workload lengths before
  measurements. Hosted CPU/OS process memory is distinct from guest accounting.
- Typed edit/settings actions and scoped-context design begin here. Hosted test
  authority is a model, not hardware isolation or production capability security.
- M2 proves hosted L0/L1 continuity. Compiled native behavior replacement is a
  separately recorded feasibility experiment; guest L2 safety remains M5.
- Keep the 256 MiB cold-idle desktop target and the requirements' frame/input/reload
  targets. Hosted results inform estimates; they do not certify those guest targets.
- No guest heap, scheduler, user processes, transport, input drivers, native assistive
  consumer, durable file operations, service replacement, AI or kernel hot reload.
  M3 supplies isolation/context policy; M4 confirms the backend and guest assets;
  M5 proves state-preserving app behavior replacement.

## Evidence and story completion

Use `Not started`, `In progress`, `Blocked`, or `Done` in each story and this index;
update both together. All checklists begin unchecked. Done requires every criterion,
the required negative cases, reproducible evidence and an explicit recorded review.
A prototype failure records a bounded finding and next experiment; it is not a
waiver of the milestone. ADRs remain Proposed pending their documented gates.

Keep full generated runs under ignored `build/m2/<story-id>/<run-id>/` or `work/`.
When implemented, retain a compact reviewed bundle under
`evidence/phase-2/<story-id>/<run-id>/`, following the
[retention policy](../evidence-retention.md): 4 MiB/200 files/256 KiB per file defaults.
P2-01 now implements the local prerequisite/lifecycle paths in the
[hosted runbook](hosted-prerequisites-runbook.md). P2-02's
[shared fixture runbook](shared-fixtures-runbook.md) and
[local acceptance](../../../evidence/phase-2/P2-02/2026-10-21/README.md) supply the
locked comparison corpus and draft schemas. P2-03's
[document/action runbook](document-actions-runbook.md) and
[local acceptance](../../../evidence/phase-2/P2-03/2026-10-21/README.md) add the
portable logical model, shared executor, context model and bounded checkpoint codec.
P2-04's [text runbook](text-conformance-runbook.md) and
[local acceptance](../../../evidence/phase-2/P2-04/2026-10-21/README.md) add pinned
fonts, independent shaping references and bounded text/composition fixtures.
P2-05's [custom candidate acceptance](../../../evidence/phase-2/P2-05/2026-10-21/README.md)
now passes 150 captures, shared workflows, text, semantics and native presentation.
Use its [runbook](custom-ui-runbook.md); full redraw/copies, immediate transitions
and text/native-accessibility limits remain comparison findings. P2-06
[accepts bounded Slint evidence](../../../evidence/phase-2/P2-06/2026-10-21/README.md):
AOT software primitives, explicit shared text/input/semantic adapters and recorded
native feature failures. Use its [runbook](slint-ui-runbook.md).
P2-07's [local target feasibility acceptance](../../../evidence/phase-2/P2-07/2026-10-22/README.md)
closes both real-API final links with a bounded probe arena, independent symbol/
section/codegen audits and deliberate missing-hook/host-feature failures. Use the
[freestanding runbook](freestanding-ui-runbook.md); guest runtime obligations and
Slint feature/license gaps remain explicit. P2-08's
[accepted local comparison](../../../evidence/phase-2/P2-08/2026-10-22/README.md)
adds 20 paired measurement blocks, independent quality/state/semantic review,
100 direct resource publications per candidate and bounded compiled-helper cases.
Frame-budget misses and Slint component retention remain material findings.
Use the [comparison runbook](comparable-ui-runbook.md). P2-09 now records a
[provisional custom Rust direction](ui-selection-review.md), exact terms/source
ownership, frozen Slint comparison and required completion cases. P2-10–P2-13
remain Not started; M2, guest UI and guest reload remain unaccepted.
Each run records:

- Source commit/dirty-input hashes, harness revision, compiler/profile, dependency
  locks/features, asset/font provenance and configuration hashes.
- Candidate/build IDs, host OS/CPU, renderer, size/scale, fonts/Unicode versions,
  scenario/input seed, sample counts, clock definitions and warm/cold status.
- Separate compile, link, render, interaction, text/semantics, reload and measurement
  outcomes. Retain selected raw traces, exact fixtures, image references and failure
  reasons. Golden tolerances are fixed before runs; screenshots cannot prove state.
- Before/after document revisions/hashes, selection, scroll, focus, window geometry,
  process/session identity and monotonic samples for continuity. These are host
  identities, never mislabeled as a kernel boot ID or guest uptime.
- Raw metric samples and aggregation method, dropped-event counts, memory attribution,
  idle intervals, resource peaks and limitations. No cross-machine clock subtraction.

Implementation/harness commands are supplied by their owning story. P2-01's
`hosted` lifecycle command is documented in its runbook. P2-05/P2-06 provide
candidate render/acceptance commands; no M2 aggregate acceptance command exists yet. Use argument arrays, bounded runs and
explicit process cleanup. Reuse local signing without changing host policy implicitly.
These local stories do not authorize Jira creation or remote uploads.

## Traceability

Epic requirements: hosted portions of R04/R08/R09/R15, R19 interface design and R18.
R03/R14/R16/R17 receive preparatory evidence only. Test families: M2 subsets of
T02/T05/T07/T09/T10/T13/T14. Guest continuity, isolation, security, persistent state,
native screen-reader support and release reproducibility remain later gates.

Read [milestones](../milestones.md), [backlog](../backlog.md),
[validation](../validation.md), [reconciled direction](../../00-overview/reconciled-plan.md),
[UI comparison](../../01-research/ui-toolkit-comparison.md),
[UI design](../../02-architecture/ui-framework-and-design.md),
[text/accessibility](../../02-architecture/accessibility-and-internationalization.md),
[performance](../../02-architecture/observability-and-performance.md),
[surface contract](../../06-contracts/ui-and-scene-protocol.md),
[actions](../../06-contracts/actions-and-context.md),
[ADR-0008](../../04-decisions/0008-rendering-path.md),
[ADR-0011](../../04-decisions/0011-ui-authoring.md) and
[ADR-0015](../../04-decisions/0015-typed-actions.md).
