# ADR-0011: Provisional custom Rust UI after the toolkit comparison

Status: Proposed. Date: 2026-10-17.
Revision 2 replaces the untested custom-builder-first default with a bounded
Slint/custom comparison; the motivation to avoid a new language remains.

P2-02 supplies the [common token/layout subset and fixture lock](../06-contracts/hosted-ui-fixtures.md)
with separate future visual and semantic/state acceptance. Both candidate stories
reference it. P2-04 adds the [bounded text pipeline and common fonts/corpus](../06-contracts/hosted-text.md),
with independent shaping references and explicit mixed Unicode data limitations.
P2-05 now adds the [locally accepted bounded custom candidate](../../evidence/phase-2/P2-05/2026-10-21/README.md)
with reference rendering, shared workflows/semantics and native presentation.
Its full redraw/copy costs, immediate standard transitions and text/native limits
remain comparison inputs. P2-06
[adds Slint evidence](../../evidence/phase-2/P2-06/2026-10-21/README.md): AOT/software
primitives with explicit shared text/input/semantic adapters and recorded native
text/effect/clip/blend gaps. This does not establish native toolkit parity or
resolve OS/API distribution terms. P2-07's
[standalone final-link review](../../evidence/phase-2/P2-07/2026-10-22/README.md)
now closes both actual scene/text/raster configurations with bounded probe memory
and explicit M3/M4 hooks. This does not establish guest execution, sustained heap
use, native feature parity or runtime replacement. At P2-07, reload and toolkit
selection remained open; its result selected no new UI language or production toolkit.

P2-08's [comparison and conditional recommendation](../../evidence/phase-2/P2-08/2026-10-22/comparison-report.md)
now supplies optimized host distributions, process retention, independent quality
observations and direct resource/compiled-helper cases. Custom has more required
fidelity but substantial frame-budget misses; Slint has lower costs in many cells
while retaining shown components and missing required fidelity/rights closure.
Those are current adapter/prototype findings. The P2-09 review below assesses the
conditional recommendation and bounded completion path; this ADR remains Proposed.
No native-window/file-driven hosted reload or guest replacement is accepted here.

## P2-09 provisional decision — 2026-10-23

Select custom-ui/text-core/editor-model 0.1.0 with the existing CPU renderer,
Rust builders and validated token/layout data for bounded P2-10 onward completion.
The [decision review](../05-delivery/phase-2/ui-selection-review.md) evaluates all
eight axes, mandatory criteria, exact configuration, source ownership, guest
integration, license choices and completion/reconsideration gates. The
[selection inventory](../../legal/ui-selection.json) pins active target features,
all 28 third-party packages, fonts, notices and frozen Slint comparison inputs.
Review is local Codex self-review, not independent or owner architectural approval.

Required fidelity and explicit ownership favor custom; performance does not:
its trace-off frame p95 is 159.358 ms and every sampled frame misses 16.667 ms.
Damage/cache/layout/motion work and optimized remeasurement remain required.
Slint's primitive speed merits credit, but current feature/lifetime gaps and
unresolved OS/API terms block its adoption. Freeze its prototype for comparison
and relevant shared regressions; maintain one production direction.

The chosen upstream terms are explicitly selected for the intended proprietary
OS/app/SDK use, with notice obligations; no project distribution is cleared.
P2-10/P2-11 must finish all M2 behavior and real same-process resource continuity;
P2-12/P2-13 must measure/review the final core. M3 supplies isolated runtime hooks,
M4 confirms this exact core and resource reload in the guest, and M5 proves
behavior route transfer/retirement. Failure of these gates reopens the decision.
This ADR remains Proposed; the earlier paragraphs preserve comparison history.

## Context and decision

The comparison used the same scenes, action/state model and OS surface/input
boundaries. P2-09 provisionally selects the custom Rust path; confirm guest
behavior at M4. Semantic tokens, stable identity, accessibility and versioned reload
state remain requirements whichever path wins.

## P2-10 selected hosted completion — 2026-10-23

The [selected-core local review](../../evidence/phase-2/P2-10/2026-10-23/README.md) accepts bounded hosted
layout/invalidation, input/semantic policies, visual text affinity and motion with
the unchanged corpus and tolerances. Slint remains frozen. Real file reload and
measured optimization still require P2-11/P2-12; M2 and guest confirmation remain
open, so this ADR stays **Proposed**.

## Alternatives

Slint may reduce widget/tooling work but needs an exact no-OS platform/reload and
text/effects audit. A custom core gives control but increases first-party work.
HTML/CSS/JS imports a browser engine/runtime; a new DSL adds language/tooling work.

## Consequences

Toolkit internals need not equal our compositor scene protocol. Slint may submit
raster buffers through common surface APIs. Compiled changes can use process/module
replacement. Hosted preview does not prove guest interpreter or native reload support.

## Gate and reconsideration

M2 compares editor/file-list scenes, updates, frame/memory cost, authoring effort and
guest compile feasibility. M4 confirms the candidate; M5 proves compiled behavior
replacement with unsaved text retained. Resolve distribution terms before commitment.
Do not maintain two full production UI stacks by default.

Evidence: [UI design](../02-architecture/ui-framework-and-design.md),
[comparison](../01-research/ui-toolkit-comparison.md),
[S76](../01-research/sources.md#s76), [S78](../01-research/sources.md#s78).

## P2-11 hosted resource continuity — 2026-10-23

The [local resource reload review](../../evidence/phase-2/P2-11/2026-10-23/README.md) now accepts bounded file building, coherent token/topology publication and real same-window/process state continuity with explicit input/reader retirement. The selected baseline geometry survives the initial reload, and caret/transition scheduling is retained. P2-12 optimization/stability, P2-13 M2 acceptance and M4 guest confirmation remain open. This ADR remains **Proposed**.
