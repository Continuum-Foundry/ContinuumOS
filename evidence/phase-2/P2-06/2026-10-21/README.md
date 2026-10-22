# P2-06 — Slint candidate evidence accepted locally

**P2-06 is Done as a bounded comparison experiment.** This is Codex's local
implementation/self-review, not independent approval or acceptance of the native
Slint feature gaps. The implementation is on `codex/p2-06-slint-ui-candidate`,
based on `34dc796ca13cada319e1856eee7d28b2f6298ed8` plus the recorded dirty input
hashes. Changes remain local and uncommitted; no push, PR or guest launch occurred.

The [review](review.json), [acceptance](acceptance.json),
[feature outcomes](feature-outcomes.json), [adapter inventory](adapter-inventory.json)
and [runbook](../../../../docs/05-delivery/phase-2/slint-ui-runbook.md) define
reproduction and limits. No common corpus/profile/font lock or tolerance changed.

- Actual **Slint 1.18.1 AOT Rectangle/Image/TouchArea primitives and software
  rasterization** execute all **30 configurations / 150 captures / 300 asserted
  workflow states** through the common surface/provider. Two optimized runs have
  identical complete result, raster-manifest and state-trace hashes. The unchanged
  P2-05 custom raster/semantic baseline also passes after the shared harness change.
- All **45 text/scale cases and 63 integrated text captures** pass through an
  explicit **P2-04 text Image adapter**, with exact opaque text-image roundtrips.
  P2-04's official segmentation, independent HarfBuzz, fallback/composition and
  failure gates pass again. These are adapted outcomes, not Slint-native shaping.
- All **29 shared/adapter boundary checks** pass, including failed Slint text
  staging without document mutation, frame publication or a leaked surface lease.
  The inherited custom paint-oracle checks are labeled explicitly; they do not
  establish native Slint group opacity or rounded clipping. Slint-native rectangle,
  shared red-over-blue blending, rectangular clipping, TouchArea logical-ID callback,
  settled redraw and pre-shape text-image quota probes are recorded separately.
- **48 Rust unit tests** pass; the pre-existing custom font-dependent paint-failure
  test remains ignored in the default test invocation. The Slint equivalent runs
  explicitly in the candidate negative suite. Both separately compiled text-entry
  variants pass real input-event checks and token/layout property pixel probes.
- The **native Windows replay passes with 11 presented frames**, then closes/reaps
  its child. The shared host adapter presents real Slint software pixels. Physical
  input translation, native IME, multi-DPI transitions and native accessibility
  were not independently exercised. All ten final prerequisite checks pass,
  including clean Phase 0 links, signing, source paths with spaces and M1 doctor.
- The offline input review records **157 active build packages** and verifies
  **6,251 upstream source files** against pinned archives. The **73-package normal closure includes
  the host application and shared adapters**, not a guest shipment inventory.
  Selected Slint runtime features exclude std/winit/LinuxKMS/GPU/system fonts;
  compiler/build dependencies remain separate. Every added lock pin/checksum
  already existed in the Phase 0 probe; no previous pin was removed.

The signed comparison executable SHA-256 is:
`f79452025ce4e090c316d23b0a27100f9a797e4465cb62d16e84848d0238ecfb`.
The distinct compiled uppercase variant and all source/fixture identities are in
[acceptance](acceptance.json) and [compiled inputs](compiled-inputs.json).

**Native limitations remain failing comparison cases.** The isolated Slint Text
probe fails without bitmap-font setup. The selected bitmap shaper's scalar mapping
does not supply the required contextual shaping; this bridge uses P2-04 instead.
The candidate paints the functional opaque launcher fallback in every variant:
rich backdrop blur, isolated 0.88 launcher opacity and rounded ancestor clipping
are not implemented. A fractional native blend yields `[59,59,59,255]` versus the
unchanged required `[60,60,60,255]`, with zero tolerance and no waiver. Standard
transitions are immediate. Logical focus/semantics are shared adapter-derived;
there is no Slint/native accessibility export. The exact hosted interpreter input
is unavailable offline. See the complete [feature table](feature-outcomes.json).

The five overview PNGs inspect every configuration/checkpoint; columns are HD
1/1.5/2 then Full HD 1/1.5/2, rows are the five fixed checkpoints. They sample the
640x360 logical workspace for review. Selected PNGs and the text sheet retain
native pixels. All captures pass the same independent four flat/exterior-clip
samples as custom. Known rich-effect/clip/opacity/blend failures remain visible;
this is not complete cross-candidate visual quality or expert typography approval.
Goldens were frozen after exploratory review and never rewritten by acceptance.

Every capture creates a fresh compiled component/layer model and performs a full
redraw plus explicit text-image, RGB/RGBA and presentation copies. The bridge's
16 MiB text-image payload cap and P2-04's 128-entry/1 MiB bitmap LRU exclude Slint
internals, model/font storage and process memory. Counters/host timings are
instrumentation; no efficiency, 30-second idle, guest memory or reload result is
claimed. The 256 MiB guest cold-idle requirement is unchanged.

Full raw runs, failed-development records, all PPMs, Cargo metadata, binaries and
source manifests stay under ignored `build/m2/P2-06/` and `work/`. This compact
bundle retains all capture metadata, timings/text outcomes, one configuration's
state/semantic observations with hashes of complete local matrices, native traces,
selected images and diagnostic excerpts. [Development findings](development-findings.json)
preserve the sandbox signer failure, lock-change recheck rejection, corrected
behavior-test mistake, initial API errors and native blend divergence.

P2-07 is next: isolate and link the exercised AOT/text/platform path with real
allocation and runtime hooks. P2-08/P2-09 must charge shared adapters, native gaps,
full redraw/copies and maintenance costs honestly, and resolve OS/API distribution
terms before commitment. No toolkit or distribution license is selected; all ADRs
remain Proposed. No guest UI, user isolation, native behavior replacement, live
continuity or release guarantee follows from this experiment.
