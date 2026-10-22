# P2-06 — Bounded Slint candidate

This experiment uses Slint **1.18.1**, its compiled Rust macro and software
renderer, with an explicit P2-04 text Image adapter. The common corpus, logical
model, fonts, input/state/semantic assertions and surface ownership rules remain
unchanged. Native text, rich effects and native accessibility are separate findings;
a passing adapter workflow is not a passing native toolkit feature.

```powershell
. ./tools/Enter-Dev.ps1
pwsh -NoProfile -File tools/Test-P2-06.ps1
cargo run --locked --offline -p xtask --features slint-candidate --profile m2-comparison -- test --suite slint-ui
pwsh -NoProfile -File tools/Preview-Slint.ps1 -Replay
```

The acceptance script checks offline prerequisites, exact expanded dependency
source/archive identities, tests and independent P2-04 references; compiles two
behavior variants; exercises real token/layout properties; expects isolated native
text and offline interpreter failures; compares two complete candidate runs against
reviewed raster baselines; replays the unchanged custom candidate; and presents a
bounded native Windows replay. Run in the existing enrolled developer account.
The ordinary sandbox has a different certificate store. The existing compiler
signer enrollment was refreshed after reviewing Cargo.lock: every added pin and
checksum already existed in Phase 0's dependency-probe lock, and no existing pin
was removed. No certificate or App Control policy was created or changed.

For interactive presentation omit `-Replay`. The host-only WinForms adapter is
shared with P2-05 and selects the `slint-host` Rust command. The provider owns
state and input; Windows supplies the event loop and presents owned BGR24 bitmaps.
The native replay injects the locked script, rather than independently exercising
physical input translation, native IME, accessibility or a multi-DPI transition.
The adapter closes/reaps its process on success, timeout and error.

## Actual feature sets

`tools/xtask` enables Slint only with `slint-candidate`. The exact requested
features are `compat-1-18`, `unsafe-single-threaded`, `libm`, `renderer-software`,
with defaults disabled. The normal renderer closure has no `std`, winit,
LinuxKMS, GPU, host font discovery or native event loop. **The xtask executable
itself uses Windows std** for files, timing, signing and presentation orchestration.
The compiled component and platform adapter currently live in this host harness;
P2-07 must isolate and actually link the exercised component/text adapter for the
guest. This story does not prove authored UI code compiles or links freestanding.

`Review-SlintInputs.py` verifies active crate archives against Cargo.lock and every
expanded source member against its archive. It records the actual Cargo `tree`
normal/no-proc-macro closure separately from the host compiler/build graph.
`cargo metadata` can unify conditional and host features; its union must not be
used to claim that std/winit/fontique entered the chosen runtime. Conversely,
compiler-side fontique, swash, image, SVG and platform facilities are host inputs,
not guest-compatible runtime findings. Full Cargo metadata and inactive/other-platform graphs stay local; active feature
trees, package features/checksums and the review are retained. The lock contains disabled and
other-platform optional packages that are not linked.

Hosted compiled UI is exercised. A different hosted `std`/vector-font configuration
is not tested here; its host services and font collection must be isolated and
audited before it can receive coverage credit. Hosted interpreter preview is a separate offline
resolution attempt for `slint-interpreter = 1.18.1`; the input is unavailable in
the local cache, and Cargo's expected failure is retained without network override.
No guest interpreter is proposed or tested. Token colors and geometry reach
compiled properties/models through genuine Slint setters; changing compiled Rust
behavior still rebuilds/restarts the host. Neither operation proves live native
replacement or P2-11 state-preserving reload.

## Renderer and adapter boundaries

Slint creates `MinimalSoftwareWindow` through the small `Platform` implementation.
It owns AOT Rectangle/Image/TouchArea items and rasterizes them into RGB8. The host
converts that result to opaque RGBA8 and commits/releases the same versioned
surface lease used by P2-05. An analytic rectangle, blend, rectangular clip,
TouchArea logical-ID callback and settled-window redraw test exercise actual Slint
APIs independently of the shared provider. Every normal capture creates a fresh
AOT component/window and rebuilds its layer model; full redraw and copies are
intentional first-candidate costs, not optimized partial rendering.

The runtime clock is an explicit monotonic millisecond value advanced by the
harness. No animations, timers, cross-thread proxy or native clipboard are
installed. All Slint work runs on the main host thread; `unsafe-single-threaded`
requires that policy globally, not merely Rust `Rc` discipline in this module.
Callbacks, font work and rendering belong in future user/service execution,
never in the kernel or IRQ path. P2-07/M3/M4 must supply allocator/OOM/panic,
FP/SIMD policy, real clock/wait/input and presentation hooks. No such guest
services are assumed from the Phase 0 library compile.

The fixture bridge reuses P2-05's bounded controller, geometry validation, list
realization, surface leases, semantic mapping and **text-only** Canvas API.
`paint::render` does not draw the Slint candidate. Scene rectangles, controls,
image composition and output pixels are Slint's. These shared first-party adapters
must be charged to both candidate maintenance costs; this prototype does not
establish an off-the-shelf Slint widget/FocusScope/semantic implementation.
Locked input is routed through the common logical controller, not Slint TextInput.
A separate Slint TouchArea probe confirms a toolkit callback carries the stable
logical control ID. Common keyboard/semantic actions use the one typed executor;
there is no native assistive-technology consumer or native semantic-tree export.

## Explicit text and feature outcomes

| Feature | This candidate | Completion path / comparison consequence |
| --- | --- | --- |
| Locked text, shaping, bidi, selection, composition | P2-04 rasterized premultiplied text Images; all text/scale and integrated cases execute | Shared adapter effort and temporary images count toward this candidate; native text is not credited |
| Native no-std Slint text | Isolated Text probe fails with no bitmap font; native font setup is not implemented by this bridge | The pinned bitmap pipeline maps scalars to glyphs without the required contextual shaping; a locked shaped-glyph/atlas integration or different audited pipeline is needed |
| Font fallback | Exactly six hash-checked fonts through P2-04; no discovery | Compiler/native host fallback cannot fill missing coverage; emoji misses remain visible |
| Logical focus/semantics/actions | Shared adapter-derived IDs and assertions; Slint TouchArea ID probe | Implement/audit toolkit focus and native semantic export before claiming either |
| Backdrop blur | Unsupported in this bridge; functional opaque token fallback in every variant | Implement compositor-side effect integration and test the fixed three-pass oracle, or disqualify the feature at selection |
| Rounded ancestor clip | Axis-aligned clipping only; native rounded backgrounds | Masked layers/compositor clip integration and fixed edge tests required |
| Launcher opacity | Opaque fallback; isolated 0.88 group opacity not implemented | Preserve overlap failure in P2-08; do not treat fallback as rich-effect parity |
| Fractional alpha arithmetic | Exact shared red-over-blue fixture passes; additional round-to-nearest comparison fails and retains expected/actual channels | Match the common arithmetic through an audited pixel adapter/upstream integration before claiming exact general blend parity |
| Motion | Immediate transitions, no animation | Implement standard interpolation or retain as quality gap; reduced motion remains immediate |
| Virtual files and text cache | Same bounded realization and P2-04 bitmap LRU as custom | Slint image/item storage and its internal caches are separate; no universal cache/heap cap claim |
| Clipboard, IME, native accessibility | Unavailable | Future platform services and adapter work |

All 30 configurations execute the ten workflow steps and five capture checkpoints.
State `effects`/`motion_ms` fields describe the **requested common model**, while
capture `rich_effect` and the table above describe actual paint. The 45 standalone
text rasters also pass through real Slint Image items; this validates the adapter,
not Slint shaping. The 63 integrated captures cover every common complex/long
string in editor/search/label nodes. Official segmentation and independent native
HarfBuzz references are replayed separately by P2-04 acceptance.

The original boundary/failure checks test the shared adapters (including validation,
focus removal, stale input, overflow reset, missing resources, large logical lists,
cache quotas and surface exhaustion). They are not renamed as Slint-native tests.
An additional Slint text-adapter failure verifies no unpublished frame/document
mutation or outstanding surface lease survives a failed paint.

`timings.json` records host `Instant` intervals for shared layout, adapter/text +
Slint raster work, and surface/RGB copying/hashing. Image allocation, component
construction and full redraw are included in the raster interval; file writes are
excluded. Surface bytes/allocations, realized rows, text shape work, bitmap cache,
staged image payload and RGB scratch are recorded separately. The bridge rejects
more than 16 MiB of staged text-image payload before the next image allocation;
this excludes Slint internals, model/font storage and the RGB/common surfaces. `touched` counts
output pixels copied; native overdraw/allocation counters are unavailable. Native
process RSS/commit and 30-second idle/damage/reload measurements remain P2-08/P2-12.
Neither hosted costs nor the VM capacity satisfy the 256 MiB guest cold-idle target.

## Distribution questions and source review

No distribution license is selected by this experiment. Original project terms
remain unchanged and no binaries are uploaded. Intended use is a private UI
implementation in the original OS shell and native applications; a future public
OS UI SDK could expose toolkit functionality, which is a separate unresolved case.

The pinned manifests declare GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR
LicenseRef-Slint-Software-3.0. The cached royalty-free 2.0 text covers defined
applications with attribution and excludes standalone toolkit/API exposure and
embedded use; the cached commercial text is version 3.0.5 and also limits API
exposure. GPL distribution implications, OS/platform classification, SDK exposure,
attribution and any commercial/API agreement need qualified review before selection
or distribution. No assumption that a custom OS implies embedded or desktop-app
license eligibility is made. No contact with the vendor is authorized here.

Primary materials rechecked on 2026-10-21: [platform API](https://docs.slint.dev/latest/docs/rust/slint/platform/),
[software renderer](https://docs.slint.dev/latest/docs/rust/slint/platform/software_renderer/),
[renderer limitations](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backends_and_renderers/),
[royalty-free 2.0 text](https://slint.dev/agreements/slint-royalty-free-license.pdf),
[upstream license files](https://github.com/slint-ui/slint/tree/v1.18.1/LICENSES).
Exact cached source/notice hashes, rather than moving documentation URLs, define
the tested implementation. P2-08/P2-09 receive the unsupported cases and license
questions; this story selects no toolkit and changes no ADR to Accepted.
