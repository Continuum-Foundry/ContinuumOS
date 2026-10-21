# ADR-0011: Compare UI toolkits before committing to a custom core

Status: Proposed. Date: 2026-10-17.
Revision 2 replaces the untested custom-builder-first default with a bounded
Slint/custom comparison; the motivation to avoid a new language remains.

P2-02 supplies the [common token/layout subset and fixture lock](../06-contracts/hosted-ui-fixtures.md)
with separate future visual and semantic/state acceptance. Both candidate stories
reference it. P2-04 adds the [bounded text pipeline and common fonts/corpus](../06-contracts/hosted-text.md),
with independent shaping references and explicit mixed Unicode data limitations.
Actual candidate integration, rendering, reload and toolkit selection gates remain
open; no new UI language or production toolkit is selected.

## Context and decision

Compare Slint and a minimal Rust builder/renderer using the same scenes, action/state
model and OS surface/input boundaries. Choose provisionally at M2 and confirm guest
behavior at M4. Semantic tokens, stable identity, accessibility and versioned reload
state remain requirements whichever path wins.

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
