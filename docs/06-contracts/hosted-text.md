# Bounded hosted text experiment (P2-04)

This is a local Rust library and comparison corpus, not a process ABI or native
IME service. [text-core](../../crates/text-core/src/lib.rs) is `no_std` + `alloc`;
the host xtask owns files, diagnostics and preview images. Neither the kernel nor
the ISO links this code. Toolkit selection and all ADR statuses remain Proposed.

## Inputs and supported subset

[Font lock](../../fixtures/m2/text-v1/fonts.lock.json) pins immutable upstream
URLs, sizes, SHA-256, face zero and this fallback order:

1. Noto Sans Regular: Latin, combining accents and ligatures in the corpus.
2. Noto Sans Arabic Regular: the Arabic corpus, including contextual joining.
3. Noto Sans Hebrew Regular: Hebrew corpus with mixed European numbers.
4. Noto Sans Devanagari Regular: prebase vowel and conjunct fixtures.
5. Noto Sans CJK SC Regular: the simplified Chinese wrapping fixture.
6. Noto Sans Mono Regular: equal-advance Latin editor text.

P2-02's `ui-sans` and `complex-fallback` slots use the order above. `ui-mono`
uses font six first, then fonts one through five, in that order. The font lock's
zero-based `slots` arrays bind all three names explicitly. `Config.mono` selects
the editor order; false selects the shared sans/fallback order. Monospacing is
claimed only for covered Latin mono glyphs, not script fallbacks. Both candidates
must use these same slot bindings and report any integration differences.

Only these exact six byte identities are parsed. No host discovery, arbitrary
font upload, collection index, variable axes, synthetic bold or color emoji is
supported. Goldens require all assets; a missing/wrong asset blocks execution.
The OFL notices are retained separately for [Noto](../../licenses/third-party/noto-fonts/OFL.txt)
and [CJK](../../licenses/third-party/noto-cjk/OFL.txt). Font bytes remain under
ignored `work/text-inputs/`; the explicit supplier verifies them offline by default.

The supported claim is the authored corpus, not every script contained in a font.
Emoji ZWJ, skin-tone and flag sequences are grapheme-edit fixtures with explicit
coverage misses, not a color emoji rendering claim. U+10FFFF is an authored
missing-glyph case. Missing clusters use face zero's `.notdef` glyph(s), with the
entire logical cluster reported missing; the fallback never divides a grapheme
between faces. Multiple `.notdef` glyphs can represent one unsupported sequence.

## Pipeline and mappings

UTF-8 storage remains in P2-03's 64 KiB document owner. A text layout request is at
most 4096 UTF-8 bytes. Larger documents need a candidate-owned paragraph/viewport
adapter; this API does not automatically truncate them.

- Extended grapheme and word boundaries: unicode-segmentation 1.13.3, Unicode 17.
  [Data lock](../../fixtures/m2/text-v1/data.lock.json) pins official Unicode 17
  GraphemeBreakTest/WordBreakTest inputs; all supplied rows run.
- Paragraph bidi and per-line visual runs: unicode-bidi 0.3.18, Unicode 16.
  The fixture fixes mixed Arabic/Hebrew/number visual cluster order independently.
- Within each directional run, assign a font per grapheme and coalesce matching
  font/script pieces. Common/inherited script follows the preceding piece;
  rustybuzz guesses segment script and direction is supplied from bidi levels.
  This is a bounded itemizer, not a universal language/script resolver.
- Shape with rustybuzz 0.20.1, monotone grapheme clusters, default features, no
  language override. Ligature-off changes `liga`/`clig`; required Indic shaping
  remains active. Return glyph IDs, UTF-8 cluster spans, offsets and advances.
- unicode-linebreak 0.1.5 uses Unicode 15 opportunities. Fit logical advances,
  prefer a grapheme-aligned legal break, and emergency-wrap a long path at a
  grapheme boundary. Reshape each line for contextual forms; shrink again if
  reshaping exceeds width. A single overwide grapheme is retained with overflow.
  LF is supported, including blank/trailing lines. Other control characters,
  CR/CRLF, paragraph/line separators and bidi isolates/overrides reject explicitly.
- Fontdue 0.9.4 performs unhinted grayscale coverage rasterization. Preview places
  bitmaps at integer pixels using shaping offsets; antialias overlap uses coverage
  union. There is no subpixel positioning, LCD, color glyph or small-text hinting
  quality claim. Logical font sizes 8–48 and scales 1.0/1.5/2.0 are accepted;
  width is physical pixels, 1–4096. Font size times scale determines raster pixels.

Mixed data versions are intentional recorded limitations of the cached stack:
rustybuzz's normalization, combining class and mirroring data use Unicode 16;
its resolved script/general-category dependencies use Unicode 17. The older
characters in the authored script corpus must pass; no single-version full Unicode
conformance claim is made. Upgrading requires new locks, fixtures and review.

Each line contains visual glyphs and caret stops with logical byte offsets.
Consecutive glyphs sharing a cluster aggregate their advance. Stops occur only at
extended-grapheme boundaries. An `ffi` ligature gets equal-width subdivisions;
this is an explicit editing policy, not OpenType GDEF caret extraction. RTL
clusters reverse their logical stops. At bidi boundaries one byte can have two
visual positions. Hit testing selects the closest stop, ties to the smaller byte;
the adapter must retain the chosen visual affinity for arrow navigation. This
experiment does not implement a complete visual-arrow keyboard policy.

Per-line selection accepts only valid caret endpoints and returns cluster-aligned
visual intervals, including partial ligatures and discontiguous bidi selections.
Adapters clip a multi-line logical range to each line. Pixels are not used to
infer document correctness. Font/scale changes rebuild layout; no shape/layout
memoization exists. Bitmap cache keys include immutable font slot/identity, glyph,
logical size and scale. Direction/features/offsets belong to layout, not bitmap
identity; adding variation, subpixel origin or raster modes requires key revision.

## Composition and resource changes

[Composition fixtures](../../fixtures/m2/text-v1/composition.json) keep preedit,
caret and captured document revision/selection outside committed text/history.
Updates are at most 1024 UTF-8 bytes and require a grapheme-aligned preedit caret.
Invalid updates preserve the previous preedit. Cancellation drops only preedit.
Commit substitutes the preedit through the existing authorized typed Insert action
once, including selection replacement and one undo step. Empty committed preedit
therefore replaces the selection with empty text. The adapter clears preedit after
successful commit; failures preserve it. The executor still checks caller, rights,
deadline, cancellation, epoch and revisions. A changed captured document or
selection reports Conflict before commit. Resource-only relayout discards views,
retaining the same live provider and composition owner. This is modeled relayout,
not a demonstrated hot reload. Provider/IME replacement must explicitly cancel or
reject staging; no composition checkpoint or native Windows/guest IME exists.

## Resource and failure policy

Limits are checked before shaping/raster work where possible:

| Resource | Bound/policy |
| --- | --- |
| Text request | 4096 UTF-8 bytes; document owner separately permits 65536 |
| Font | Six exact identities; at most 24 MiB each, 128 SFNT tables, 20 MiB/table |
| SFNT preflight | Magic, directory and overflow-safe table ranges before deep parsing; not sanitization |
| Glyph output | 8192 per shape and aggregate final layout |
| Lines | 256; excess rejects instead of silently dropping text |
| Reshaping | 131072 cumulative UTF-8 input bytes per request; excess rejects |
| Raster request | Valid font/glyph index, at most 96 px/em; preflight outline box ≤256×256 |
| Bitmap | At most 65536 bytes; reject excess |
| Bitmap LRU | At most 128 entries and 1 MiB payload; least recently used eviction |
| Decoded font geometry | Lazy, at most six immutable faces; separately retained outside bitmap LRU |

Encoded font bytes, parsed outline geometry, layout vectors, cache metadata and
temporary rasterizer allocations are **not** included in the 1 MiB bitmap payload
cap. This is not a total-memory guarantee. Only known pinned fonts cross the deep
parser, so these tests do not admit hostile arbitrary fonts. Reassess eager
Fontdue geometry, recursion, allocation and CPU bounds before accepting new fonts
or guest integration. The hosted experiment uses `try_reserve` at font-list and
preedit construction but dependencies/other vectors still allocate infallibly;
allocator exhaustion can abort the host process. It is not recoverable OOM or a
claim of safe state preservation across process failure. Quota failures return an
error without changing committed document state. Guest allocation remains open.

## Shared comparison obligations

[Corpus](../../fixtures/m2/text-v1/corpus.json) and its lock apply to both
candidates. Native HarfBuzz 14.3.0 from the pinned QEMU DLL supplies independent
single-script expected glyph IDs/clusters/positions via the host-only
[reference tool](../../tools/Reference-TextShape.py). Its DLL hash and corpus hash
are in the reference fixture; this oracle does not share Rust implementation code.
Semantic expectations and selected raster images have separate acceptance.
Exact repeated local raster hashes test determinism, not cross-renderer equality.
Small antialias differences cannot waive logical text, fallback or cluster errors.
P2-08 must fix cross-candidate visual tolerances before comparing its captures.

P2-05 and P2-06 must report shaping, bidi, fallback, input/composition and raster
results separately for every case and scale. Slint may retain its own pipeline;
it must load the same fonts without undisclosed platform fallback and record every
difference or required adapter. Candidate execution is pending these stories.
The prototype does not choose a toolkit, certify accessibility, implement native
assistive technology, or establish guest memory/performance/reload guarantees.
