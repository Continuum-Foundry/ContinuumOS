# P2-05 — Bounded custom UI candidate accepted locally

**P2-05 is Done.** This is Codex's implementation and local self-review on
`codex/p2-05-minimal-custom-ui`, based on `1381e0a` plus the recorded input hashes.
It is not independent approval. Changes remain local and uncommitted; no push,
PR, remote build or guest launch occurred.

The [review](review.json), [acceptance](acceptance.json),
[feature outcomes](feature-outcomes.json), [adapter inventory](adapter-inventory.json)
and [runbook](../../../../docs/05-delivery/phase-2/custom-ui-runbook.md) define
the tested scope and reproduction commands. The shared P2-02 corpus, P2-03
provider and P2-04 fonts/text locks are unchanged.

- All **30 configurations / 150 checkpoints** render through the portable custom
  Rust core. Every configuration executes ten steps: **300 independently asserted
  workflow states**. Two optimized runs have identical complete structured results,
  raster/semantic manifests and state traces. The final reviewed build also replays
  the frozen candidate-specific goldens. See [captures](captures.json),
  [state traces](state-traces.json) and [semantic observations](semantics.json).
- All **45 P2-04 text/scale cases** and **63 integrated editor/search/label captures**
  pass with the same fonts, shaping, bidi, fallback and composition policy.
  Emoji misses remain declared. P2-04 acceptance, official segmentation rows and
  independent native HarfBuzz references pass again. See [text outcomes](text.json).
- All **28 candidate boundary/failure checks** pass, including duplicate IDs,
  malformed geometry/resources/tokens, stale revisions/pointers, queue overflow,
  focus/capture removal, modeled owner-death reset, buffer exhaustion/withheld and
  stale release, list/cache limits, placeholder images, staged layout/spacing,
  action entry parity, caret hits and independent paint vectors. See
  [negative outcomes](negative.json). Actual native service death/replacement is
  not demonstrated by the modeled reset.
- List realization remains **15 rows** for the authored viewport with a hard cap
  of 64, including logical counts 10,000 / 1,000,000 / 1,000,000,000. Jump-ahead
  labels match every row of the independent sequential LCG. Bitmap cache entries
  and payload remain capped at 128 / 1 MiB; decoded fonts and other allocations
  are separate. Oversized viewports reject rather than exceed realization quota.
- **49 unique Rust tests** pass across the final custom, editor, text and host
  suites; the ten custom tests also pass in the separately compiled uppercase
  behavior variant. Targeted final tests cover text admission, failed paint
  abandonment without publishing/mutating the document, stale lease protection,
  and fully offscreen blur. Freestanding library checking passes. This is not
  guest linking/execution, native code unloading or provider replacement.
- The **native Windows preview replay** passes using the signed Rust owner and a
  small WinForms presentation adapter. It presents real software frames at
  1280×720/scale 1 and closes/reaps its child. Replay injects the locked scripts
  through the same Rust controller; physical OS input translation, native IME,
  multi-DPI transitions and native accessibility were not independently exercised.
  See [native preview](native-preview.json) and [native traces](native-traces.json).
- All ten final offline [prerequisite checks](prerequisites.json), including
  clean probe links, source path with spaces, signing and M1 doctor, pass. The
  existing signer enrollment was refreshed only after explicit owner approval;
  no new certificate or App Control policy was installed. Earlier failures are
  retained in [development findings](development-findings.json).

The five overview PNGs inspect every composition/configuration; columns are HD
1/1.5/2 followed by Full HD 1/1.5/2, rows are the five capture checkpoints. They
sample the 640×360 logical workspace for overview only. Selected full captures
and the text sheet preserve native pixels losslessly. Local inspection found
the prescribed fixed overlap, focus, selection, clipping, script forms and
variant controls present; this is not expert typography/accessibility approval.
Every capture also passes four independent flat-color/exterior-clip samples
([pixel review](pixel-review.json)); portable tests check clipping and a slow
nonconstant blur oracle independently. Screenshots do not prove state.

The candidate has **explicit comparison limitations**: hard rounded coverage,
unhinted grayscale glyphs, immediate standard transitions, full redraw after
changes, host pixel copies, static caret, bounded logical navigation and adapter-
derived semantics. Standard 120/240 ms animation interpolation is unsupported
and retained as a selection finding. Reduced effects retain the same actions.
No common tolerance or scene was weakened to accept these findings.

[Timing samples](timings.json) expose layout/raster and capture hashing/copy work,
damage, selected allocation bytes, shape/cache work and settled idle decisions.
They are not the P2-08/P2-12 workload gate, allocator-wide/process memory, measured
guest performance, live continuity or certification of the 256 MiB cold-idle target.
Raw logs, all PPMs, full traces, metadata and Git patches remain in the local
directories recorded in acceptance. This compact bundle has exact retained-byte
checksums; [compiled inputs](compiled-inputs.json), [dependencies](dependencies.json)
and [reviewed inputs](review-inputs.json) identify its source and asset context.

**P2-06 Slint is next**, then P2-07 target link closure and P2-08 comparison.
No toolkit or ADR is accepted by this candidate result; guest runtime and hosted
resource reload remain later stories.
