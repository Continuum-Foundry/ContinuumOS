# P2-04 — Establish pinned fonts and a bounded text conformance corpus

Issue type: Story. Epic: P2. Status: Done. Owner: Unassigned.
Depends on: P2-03 Done. Suggested labels: `phase-2`, `text`, `accessibility`.
Backlog: B09, B36 text comparison. Requirements: R08/R15/R18; R07 groundwork.
Test families: hosted T05/T10 and text-input bounds from T11.

## Local acceptance — 2026-10-21

The [local implementation/self-review](../../../evidence/phase-2/P2-04/2026-10-21/README.md)
accepts this bounded story. [Contract](../../06-contracts/hosted-text.md) and
[runbook](text-conformance-runbook.md) define six pinned fonts, mixed Unicode data
versions, 15 common cases at three scales, independent native shaping references,
caret/selection/wrapping, modeled composition and resource limits. Both candidates
inherit these fixtures and separate shaping/bidi/fallback reporting obligations;
neither candidate is implemented here. Color emoji, full Unicode, native IME,
recoverable OOM, guest linking/execution and real reload remain unsupported.

## User story

As an editor user, I need predictable text, caret and selection behavior so the UI
comparison evaluates real writing rather than an ASCII-only rendering demo.

## Implementation scope

1. Inventory and pin font bytes, coverage, fallback order, licensing/notices and
   Unicode data versions. Use explicit fixtures instead of machine font discovery;
   missing assets block the affected cases rather than changing fonts silently.
2. Specify UTF-8 → grapheme/word boundaries → bidi/script runs → fallback/shaping →
   line breaking → glyph/cache/raster behavior, with logical-to-visual hit testing.
   Reuse suitable portable libraries only after the exact dependency/feature audit.
3. Provide common expected text/edit fixtures and a custom-path implementation of
   the bounded subset. Slint may use its own pipeline, but must run the same corpus;
   record differences and required adapter work instead of forcing identical internals.
4. Include combining marks, emoji sequences, ligatures, mixed Arabic/Hebrew/numbers,
   an Indic shaping case, CJK wrapping, missing-glyph fallback, long paths and scale
   changes. Separate semantic text correctness from raster pixel tolerances.
5. Specify text/composition input and commit/cancel behavior, including future reload
   during composition. Bound document size, glyph work, font tables and caches;
   declare allocation failure/malformed-font handling and cache key inputs.

## Acceptance criteria

- [x] Font/data identities, licenses, exact supported script subset and missing
  coverage are reproducible; no unspecified host font is required by goldens.
- [x] Grapheme editing and selection never split invalid UTF-8 or the declared
  grapheme cases; logical/visual caret and hit testing have expected results.
- [x] The shared corpus contains a required complex-script case and both candidates
  must report shaping/bidi/fallback behavior separately, including known limitations.
- [x] Composition commit/cancel, line wrapping and scale/fallback changes have
  deterministic fixtures; unsupported IME integration is named rather than implied.
- [x] Font/cache/size bounds and failure handling are testable, and supported subset
  claims agree with the specification and candidate comparison requirements.

## Tests and evidence

Use pinned official algorithm fixtures where locally available and relevant, plus
small authored editing/shaping/hit-test cases. Exercise zero/large text, malformed
font inputs, fallback misses, cache eviction and scale/font changes. Record corpus
coverage, expected glyph/selection mappings, selected images and bounds failures.
An algorithm test pass is not full Unicode support or a native assistive consumer.

## Exclusions and handoff

No universal script coverage, full native IME service, screen reader or unrestricted
font parser port. P2-05/P2-06 receive identical fonts/corpus, the state/edit model
and explicit support obligations. Sources:
[text/accessibility](../../02-architecture/accessibility-and-internationalization.md),
[UI comparison](../../01-research/ui-toolkit-comparison.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
