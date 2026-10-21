# P2-04 — Bounded fonts/text acceptance passes

**P2-04 is Done.** This is Codex's local implementation and self-review on
`codex/p2-04-fonts-text-conformance`, based on
`e5c4e8fd184aa8ea60615e84a692d3475e45493b` plus recorded dirty input hashes.
It is not independent approval. Changes remain local and uncommitted; no push,
PR or remote execution occurred.

The [review](review.json), [acceptance](acceptance.json), [structured text results](text-result.json)
and [runbook](../../../../docs/05-delivery/phase-2/text-conformance-runbook.md)
record reproduction and limits. The [contract](../../../../docs/06-contracts/hosted-text.md)
defines exact data versions, supported subset, caret/composition policy and bounds.

- Six unmodified Noto fonts are pinned by immutable public URLs, byte sizes and
  SHA-256, with retained OFL notices. Explicit fallback covers the authored Latin,
  Arabic, Hebrew, Devanagari and Chinese cases, plus a mono Latin editor slot.
  All three P2-02 font slots have explicit orders. Fonts and official test data remain
  local; missing/wrong assets block execution. No Windows font discovery is used.
- All **45 observations** pass: 15 shared cases at scales 1.0/1.5/2.0. They cover
  combining accents, emoji editing/misses, flags, ligatures, equal-advance mono, Arabic, Hebrew,
  Devanagari prebase/conjunct shaping, mixed bidi/numbers, CJK wrapping, missing
  glyphs, blank/trailing lines and a long path. Every possible byte selection is
  checked against the P2-03 document boundary policy; caret/hit equivalents, full
  and partial selection geometry, feature changes and line ends pass.
- Six single-script results exactly match independently executed native
  **HarfBuzz 14.3.0** glyph IDs/clusters, with a 0.002 physical-pixel tolerance for
  converted advances/offsets. The existing pinned QEMU DLL is the reference;
  Python ctypes shares no Rust shaping/layout implementation.
- All **766 official Unicode 17 grapheme cases** and **1944 word cases** pass.
  Authored bidi/wrap expectations are separate from those algorithm tests. The
  cached bidi/normalization/linebreak tables have explicitly different Unicode
  versions; this is not full single-version Unicode conformance.
- All **six composition scenarios** pass: commit after view relayout, cancellation,
  denied caller, deadline, cancelled request and stale document. Preedit validation
  is atomic and same-owner relayout leaves committed state unchanged. Commit uses
  the existing typed executor and creates one undo step. This is modeled input,
  not a native IME or demonstrated live reload.
- All **14 bounds/cache checks** pass, including maximum/oversized input, invalid
  sizes/scale/width, control/override rejection, line/shape quotas, corrupt/missing
  fonts, glyph indices, LRU eviction and scale-key regeneration. SFNT malformed
  inputs reject before deep parsing. Arbitrary fonts are not admitted.
- All **38 Rust tests** pass (17 editor-model, two text-core, 19 xtask). The text
  library compiles for `x86_64-unknown-none`; this is not guest linking/execution.
  P2-02/P2-03 replay regressions and the independent shaping oracle pass.
- Two separate text commands produce identical structured bytes, including all
  45 raster hashes: result SHA-256
  `1d25bb1988bcd106d71992578f6798bcfe845539d96888e5f6b4e3304bd248f3`.
  Signed host executable SHA-256 is
  `a28a0e17515533f53e4befaea6f4e47514a89ef7c1696e8b1dd9bdd5a4b190aa`.
- All ten offline prerequisite commands pass after fixing Test-M0's existing
  omission of `crates/` from its source-path copy and reviewing/reenrolling the
  changed lock with the existing signer. No certificate or host policy changed.
  Formatting, contract validation and repository audits pass; no VM was launched.

[Compiled inputs](compiled-inputs.json) identify the tested code/fixtures separately
from the later [reviewed input set](review-inputs.json). [Dependency closure](dependencies.json)
records exact features/terms and host build scripts; it is not a shipped guest SBOM.
[Development findings](development-findings.json) preserve the earlier certificate,
path-copy, lock guard and corrupted-font-fixture outcomes. Full raw logs, patches,
metadata, fonts and all PGM images remain in the ignored local run directories
recorded in acceptance. Only this compact reviewed bundle is promoted.

The selected [contact sheet](contact-sheet.png) is a lossless conversion of passing
PGM output. Columns are scales 1.0/1.5/2.0. Rows are combining, ligature, mono spacing, Arabic,
Hebrew, Indic prebase, Indic conjunct, mixed bidi and CJK wrapping. Local visual
inspection found the tested marks, joined forms, Indic order and wrapping present;
this is no independent multilingual typography/accessibility approval. Unhinted
grayscale rendering and equal-subdivision ligature carets remain prototype policy.

This accepts R08/R15/R18's bounded text preparation and R07 composition groundwork.
Color emoji, universal script coverage, native IME/assistive technology, recoverable
OOM, complete visual-arrow navigation, actual candidate UI, guest runtime, measured
performance and live reload remain open. Encoded/decoded fonts and temporary
allocations are separate from the 1 MiB bitmap payload cap; it is not a total-memory
guarantee. The 256 MiB guest cold-idle target remains unchanged and unmeasured.
**P2-05/P2-06 receive identical fonts/corpus and must report their own shaping,
bidi, fallback, composition and raster outcomes.** No toolkit is selected.
