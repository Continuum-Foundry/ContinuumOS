# P2-08 visual and functional review

Reviewer: Codex, local implementation/self-review, 2026-10-22. This is not an
independent design approval or native assistive-consumer test.

The [contact sheet](comparison.png) has custom Rust on the left and Slint AOT on
the right. Rows are the six cases listed in [quality.json](quality.json), in order.
Each cell samples the same 640×360 logical workspace; its nearest-sample reduction
is a review aid. Full PPMs remain in the hash-bound local regression directories.
Native 1280×720 scale-2 mixed Hebrew/Arabic/digit editor captures were separately
converted losslessly and inspected at full resolution under `work/p2-08-quality/`.

| Aspect | Custom Rust | Slint AOT with P2-04 adapter |
| --- | --- | --- |
| Typography / hierarchy | Shared six-font raster text; sidebar/search/editor/list/settings hierarchy readable in the bounded workspace | Same shared shaped text images and hierarchy; this does not validate native Slint Text |
| Complex text | Mixed Hebrew/Arabic/digits visible with shared bidi/cluster geometry; 45 standalone and 63 integrated text observations | Same observations through Slint Image layers; no independent native shaping credit |
| Clipping / corners | Rounded ancestor clip and isolated launcher opacity have dedicated passing analytic vectors; hard coverage gives visibly coarse custom corner edges | Smoother native rounded primitive edges, but rounded ancestor clip is unsupported; opaque launcher hides underlying rows where custom shows dimmed/blurred content |
| Scaling | Shared physical sizes and 1.0/1.5/2.0 logical scale corpus retained; fixed scene bounds, not responsive layout | Same corpus; native fractional channel rounding remains a zero-tolerance failure from P2-06 |
| Focus / selection | Visible accent selection and yellow focus around the current control | Shared selection/focus geometry visible; no persistent native Slint focus tree tested |
| Overlap / effects | Actual three-pass backdrop blur/tint and isolated opacity; substantial raster cost remains a finding | Rich requested effect unavailable; opaque functional fallback painted and costed in every requested variant |
| High contrast / reduced effects | Tested yellow focus, cyan selection and opaque functional fallback remain readable | Same tested functional fallback and shared semantics; native rich-mode fidelity is still missing |
| Motion | Requested model settings and immediate transitions; normal animated transitions/interruption are unsupported | Same model settings; normal animated transitions/interruption unsupported by the current bridge |
| Product completeness | Fixed comparison workspace has deliberate occlusion: the settings surface covers launcher text and parts of the editor/list. It is not a finished desktop design | Same fixed comparison workspace and occlusion; native/widget breadth is not demonstrated by drawing primitives |

Functional assertions were checked independently from images:

- Fresh optimized replays pass 300 state observations each and produce identical
  logical traces across candidates: unsaved document edits, selection, search,
  file scroll/selection, focus, effect/motion actions and composition commit/cancel.
- All 150 semantic captures are equal across the two adapters, including stable
  names/roles/actions, bounds, logical list count and current focus. Native
  accessibility export/consumer, native IME and trusted input remain unavailable.
- Keyboard focus order and Enter/Space activation are fixture assertions. Typed
  action parity across button, shortcut, semantic and automation entry points
  passes separately. Functional reduced-effects paths preserve these workflows.
- All 28 custom and 29 Slint boundary cases remain in `quality.json`: malformed
  trees/resources, stale geometry, overflow reset, transient input cancellation,
  list/cache quotas and surface ownership. Slint additionally rejects failed
  staged text-image paint without publishing a frame or changing the document.
- [Pixel differences](pixel-differences.csv) describe all 150 full RGB image pairs,
  with exact hashes and different-pixel/bit counts. They are observations, not
  a relaxed equality tolerance. Candidate goldens remain unchanged and distinct.
- The existing independent flat-color/outer-clip reviewer also passes all four
  checks on each of the 150 fresh captures per candidate; see
  [custom vectors](custom-pixel-vectors.json) and [Slint vectors](slint-pixel-vectors.json).

The shared pipeline's Unicode versions, coverage, shaping and composition limits
remain those accepted by P2-04. This review does not rerun the independent native
HarfBuzz/official-segmentation oracle; their immutable P2-04 acceptance remains the
source. The portable text/font source and lock inputs have not changed. The fresh
candidate regressions rerun the locked text rasters, integrated scenes and current
keyboard/action assertions. No native screen-reader, guest text/input or full
Unicode conformance is inferred.

Both candidates report missing coverage in the same nine standalone observations:
`emoji-zwj` (one missing cluster per scale), `emoji-flags` (two per scale) and the
deliberate `missing-glyph` probe (one per scale). That is 12 missing-cluster
observations across the three scales, not complete emoji support. Other locked
standalone text cases report zero missing clusters. Their outcomes remain distinct
from the independently checked grapheme/shaping contracts.
