# Hosted UI boundaries and comparison corpus

Status: P2-02 experimental draft, data schema version **1**, 2026-10-21.
This refines the [surface contract](ui-and-scene-protocol.md),
[action contract](actions-and-context.md) and [UI design](../02-architecture/ui-framework-and-design.md).
It freezes comparison inputs for the two bounded candidates, not a process ABI,
wire encoding, toolkit selection or guest implementation. Later incompatible
data revisions require a new corpus version and explicit comparison review.

## Common inputs

The authored [corpus directory](../../fixtures/m2/ui-v1/README.md) contains seven
positive JSON inputs, mutation-based negative fixtures and an exact-byte
[lock](../../fixtures/m2/ui-v1/lock.json). Both candidates consume that same lock,
[host profile](../../config/m2-profiles.json), generated file list and future
P2-04 font lock. Neither may substitute easier text, data, effects or input scripts.
The [runbook](../05-delivery/phase-2/shared-fixtures-runbook.md) describes checks.

| Input | Purpose |
| --- | --- |
| `surface.json` | Portable dimensions, storage, revision, damage and buffer-lease traces |
| `tokens.json` | Straight semantic color values, font slots, spacing, radii, effect and motion data |
| `layout.json` | Stable logical tree, geometry, resources, actions and separate semantic IDs |
| `corpus.json` | Profiles, seed, text, state, list bounds, tolerances and required failure scenarios |
| `workflow.json` | Physical/text/composition/pointer events and independent state/semantic expectations |
| `variants.json` | Exact high-contrast colors, reduced settings and screenshot checkpoints |
| `paint.json` | Independent premultiplication/blend vectors and authored missing-icon pixels |

All documents reject unknown fields and unsupported versions. Schemas constrain
structure. Source JSON, including the profile input, uses canonical UTF-8/LF
bytes without a BOM; the corpus validator rejects CR so Git normalization cannot
silently change an accepted checksum on checkout. Cross-input semantic validation
is a separate pass. `negative.json` describes test mutations, not allowed layout
operations. No test mutation language
is exposed to an app or asset interpreter. JSON drafts contain values/IDs only.
No URI loading, expressions, script, callbacks, native pointers or host commands
are accepted. IDs identify logical objects, not authority-bearing handles.

## Geometry, pixels and bounds

Physical surface extents are integer pixels, initially 1–4096 per axis. Logical
extent is physical extent / scale. Coordinates and rectangle dimensions are
finite logical pixels; absolute coordinate magnitude and dimensions cannot exceed
16384. Rectangles use `[x,y,width,height]`, nonnegative dimensions and half-open
bounds. Damage rectangles alone use physical pixels. Scale is exactly 1/1.5/2.
Multiply logical edges by scale; floor left/top and ceil right/bottom for damage
and clips. Hit testing uses unrounded logical geometry and the current revision.
Negative origins are permitted for offscreen windows, with clipping at the surface.

Storage is top-down RGBA bytes, R first, no native-word/endian reinterpretation.
Stride is at least checked `width*4`, divisible by four; buffer length is exactly
checked `stride*height`, including padding, and at most 64 MiB. Initialize all
bytes before publication. Padding is never sampled or exposed in screenshots.
This provisional cap is a buffer safety limit, not an idle-memory allowance;
three full buffers and caches must still be counted against the 256 MiB guest
cold-idle target later. Host memory and guest memory stay separate.

The v1 software comparison uses **sRGB-encoded channels and premultiplied alpha**.
Token colors are straight RGBA8. Premultiply each color channel with
`(channel*alpha+127)/255` using integer division; alpha stays unchanged. A stored
channel must be <= alpha; zero alpha has zero color. Source-over for each channel,
including alpha, is `src + (dst*(255-src_alpha)+127)/255`. Blending is performed
in the encoded domain for this bounded software baseline; it is not a linear-light
or wide-gamut claim. All candidates use this same policy. A future policy change
requires new paint fixtures/goldens. The final opaque desktop is converted by the
host adapter to its declared native channel order; no host color-management,
scanout, VSync or end-to-end display timing guarantee follows. Capture common RGBA
bytes before host presentation, not a desktop screenshot affected by host scaling.

## Ownership, generations and damage

Each app/surface session has at most three buffer IDs. The abstract lifecycle is
`released -> acquired -> in_use -> released`; writes are permitted only while
acquired. A complete validated commit transfers read ownership; no writes or
reacquisition occur before explicit release of the same buffer/generation/lease
serial. Each acquire has a strictly increasing per-buffer lease serial, independent
of resource generation. Writes/commits/releases must carry that exact serial;
a delayed old release cannot free a newly acquired buffer in the same generation.
Serial exhaustion retires the buffer/session identity rather than wrapping it.
A repeated acquire, release without commit, wrong-generation release or withheld
release prevents reuse. The fixture finishes with every lease released. The
adapter may copy into owned storage, but it must report the copy and keep the same
visible lifecycle. A frame opportunity and a buffer release are distinct events.

An immutable generation owns all token/font/image/layout references. Old readers
retain the old generation until their releases; incrementing a generation does
not free old data or grant authority. The baseline is generation 7 and scene
revision 3; the valid candidate publishes revision 4 atomically. Reject stale or
skipped revisions and unresolved resources before publication. A rejection leaves
the previous generation and visible state intact. The lease trace also shows
buffer reuse at generation 8 after the generation-7 release. Real resource
retirement, publication and reload are implemented/tested by later stories.

At most 64 declared damage rectangles, each wholly inside the physical extent.
Excess fragmentation rejects before commit; the producer can retry with one full
surface rectangle. Damage is a hint. The compositor unions old/new bounds,
exposed areas, clips and expanded filter bounds. It must force a full redraw when
it cannot prove coverage; it cannot trust an opaque/damage hint to skip clearing.
Effects expand affected bounds by their full kernel support. Static content does
not request continuous frames; caret/clock timers are named separately later.

## Input and identity

Each event has an increasing contiguous session sequence, nondecreasing monotonic
process-relative microseconds, target logical ID and scene revision. Timestamps
are ordering data, never wall time or a cross-process latency measurement. Equal
timestamps retain sequence order. Workflow events use revision 4; stale pointer
geometry rejects and requests re-hit-testing against the current tree. Never
retarget a stale click using its old target alone. After a layout swap, enqueue
newly routed events only after publication and focus/capture reconciliation.

`key` carries a physical USB usage and pressed state, independent of generated
text; usage examples do not implement a keymap service. Modifiers are represented
by their own physical key usages (225 in the reset fixture). Repeats in future
adapters must be explicit; the current script has matched down/up transitions.
`text` carries committed UTF-8, never a fake key press. Composition has distinct
start/update/commit/cancel phases. Only commit inserts text once; cancel changes
no document revision. A composition commit must not also generate a duplicate
text insertion. Pointer positions are logical surface coordinates; scroll deltas
are logical pixels. Buttons have paired down/up; capture pins routing until
release/cancellation. The fixture's x/y are pre-scale values for every profile.

Focus/capture requests name an existing focusable logical widget. An empty capture
target releases capture. The shell owns routing/focus policy; these hosted IDs
are not proof of isolation. A removed focused widget falls back to `editor`, or
the next enabled focusable widget in declared preorder if editor is absent.
Removing a capture owner, owner death, queue overflow or reload quiescence emits
one explicit cancellation/reset: cancel gesture/composition, release capture,
clear all pressed keys/buttons, report reason and dropped-event count, then
require fresh downs. Never replay an orphan key/button up or silently lose it.

Pending input is bounded to 256 events plus one reserved reset slot. If full,
discard the pending batch, emit reset and acknowledge cancellation before accepting
the next batch. No unbounded buffering during reload. This story validates a
scripted reset trace; actual overflow/owner-death injection is a candidate test.
A whole workflow is bounded to 4096 events, 64 steps, 60 seconds of relative
timestamps and 4096 Unicode scalars per text event. Schemas bound individual
steps to 256 events. Nonfinite values reject at decode/shape validation.

Widget and semantic IDs are stable opaque strings, at most 64 Unicode scalars
(at most 256 UTF-8 bytes), unique within their respective session namespaces.
Paint order is never identity. Semantic IDs are independent of paint IDs; a
candidate maps its private tree to these logical identities. Semantics exposes
role/name/value/state/actions/bounds/focus. Virtual lists expose 10,000 logical
items and navigation while at most 64 rows are realized. Allocation addresses
and raw native toolkit IDs are never serialized.

## Token and layout data subset

Version 1 supports a complete immutable token set and a complete immutable layout
tree; there is no incremental patch/expression language. Allowed node kinds are
row, column, overlay, box, scroll, virtual-list, label, button, text-field, editor
and toggle. Their behavior is compiled first-party code, not asset callbacks.
Fields are explicit rectangle, clip, opacity, literal label, child IDs, optional
resource/action reference, focusability and semantic data. No constraint solver,
arbitrary bindings or custom native operation is supported. Reject unknown kinds,
fields, actions/versions, resources, cycles, shared children, orphans, duplicate
IDs and depth >32 (root depth 1). Maximum 1024 nodes and 64 children per node.
Validate the whole graph before applying any operation. This is the comparison
layout subset, not a required export format for Slint's private tree.

The authored rectangles are absolute logical coordinates within a 640×360
reference workspace; they fit the smallest logical extent (1280×720 at 2×).
`desktop` expands its background/clip to the actual logical extent; other fixture
rectangles stay fixed, so higher resolutions expose extra desktop. Child geometry
is absolute, and sibling paint order follows `children`. Column/row describe
containment/tab order here; the shared corpus does not introduce implicit layout
reflow. Later reload fixtures may change explicit rectangles while retaining IDs.
Both candidates must implement these same product geometries.

Action references are the six registered logical version-1 names in `corpus.json`.
Button, shortcut and automation paths resolve to the shared P2-03 provider, whose
revision/authority/cancellation checks apply at invocation time. This draft declares
edit/search/list/launcher/settings expectations; it does not implement those
providers, durable save, trusted capabilities or native code replacement.
Resource names identify synthetic document/list data. P2-04 must bind all three
font slots to one exact common inventory with digests, licenses, Unicode version,
coverage and fallback order before rendering either candidate. Missing required
fonts reject staging; no machine-installed font fallback is allowed. A deliberately
missing decorative icon uses the authored 2×2 magenta/black placeholder scaled
nearest-neighbor into a 16×16 logical box, with its accessible name preserved.

P2-04's [text contract](hosted-text.md) and [font lock](../../fixtures/m2/text-v1/fonts.lock.json)
now bind all three slots, including Noto Sans Mono for covered Latin editor text.
The text corpus adds explicit script/fallback/composition limitations; both
candidates must report those outcomes independently of raster comparison.

## Product corpus and effects

Initial unsaved text is `Draft: unsaved` plus newline, revision 11, UTF-8 byte
selection [7,14), focus `editor`, window [8,8,420,328]. Selection offsets are
byte boundaries in this draft; P2-03/P2-04 enforce grapheme-safe editing and
logical/visual mapping. Synthetic files use stable `file.00000`–`file.09999` IDs.
The generator starts at 20261020 and applies
`state=(state*1664525+1013904223) mod 2^32` once per row. Each LF-terminated UTF-8
JSON line has sorted keys `id,label,size`, no extra spaces; the label is
`File {index:05} — {state:08x}.txt`, size `state mod 1048576`. Exact bytes are
locked. Scrolling by 2400 selects logical row 101 at y=32 in a list starting y=8
with 24-pixel rows. Clamp scrolling to the logical count; never open real files.

The script replaces selected unsaved text, searches, scrolls/selects, invokes the
launcher, toggles effects/motion, cancels composition, commits Japanese text, and
resets pressed modifiers/capture. Expected document revisions/selections/actions
are assertions, independent of raster output. Complex-text samples additionally
include combining accents, emoji/ZWJ, Arabic/Hebrew plus numbers, Indic, CJK,
ligatures and a long path. Apply them to the editor/search/labels as independent
text cases, not as ASCII substitutions. The candidate records exact unsupported
coverage; the presence of strings is not a text-rendering pass.

The rich effect is a clipped launcher backdrop blur with three separable box
passes, radius 8 logical pixels (physical radius `ceil(8*scale)` per pass), integer
round-to-nearest channel averages and clamp-to-edge sampling. Filter the backdrop
before launcher children, then source-over the token tint. Expand damage by
three radii in each axis. The launcher opacity is 0.88; multiply coverage/opacity
into premultiplied channels once. Rounded launcher clip uses panel radius 8.
The settings panel overlaps both editor/list and launcher, in declared paint order.
Reduced effects use the opaque fallback token, no blur, while retaining every
control/action/semantic ID. High contrast uses the exact variant palette and
also removes blur; reduced motion uses zero transition duration and no translation,
zoom, parallax or idle decorative animation. Standard motion durations are the
token values. Constant-color blur and alpha arithmetic have independent vectors.
General toolkit filter support/cost is a later measured comparison question.

## Visual and independent semantic acceptance

The 2 sizes × 3 scales × 5 variants produce **30 configurations**, each with the
five named capture checkpoints. Record source/candidate/font/profile/corpus hashes
and exact RGBA geometry for every image. Goldens are future reviewed, candidate-
specific raster baselines; neither candidate is the other's reference renderer.
P2-02 supplies no fabricated screenshot or successful rendered golden. Both must
also satisfy the same independent geometry, clip, color/blend and semantic rules.
Generate goldens only after P2-04 font locking and explicit visual review; never
silently regenerate one to turn a mismatch into a pass.

Bounds/baselines must agree with declared geometry within 0.5 logical pixels.
Paint channels allow absolute error <=3 away from edges and <=16 within a common
one-physical-pixel glyph/rounded-clip edge mask. The mask is derived from the
reviewed reference geometry, fixed before measurement, at most 1% of image pixels;
never derive a new mask from differences. At most 0.1% of unmasked pixels may
exceed the channel tolerance; report raw maximum error/counts per region and full
image. No changed pixel outside the expanded clip/damage region is allowed.
Analytic flat color/blend vectors are exact. Missing glyphs, lost controls, wrong
stacking, clipping or semantic/state failures cannot be waived by aggregate error.

For each checkpoint, independently assert document bytes/hash/revision, selection,
search matches, focused logical ID, capture/pressed/composition state, selected
file/scroll, semantic logical item count, realized-row cap, action/version and
window geometry. Compare roles/names/actions and keyboard reachability in all
variants. Record toolkit-native versus adapter-derived semantics. Test a removed
focus node, stale pointer, overflow reset, missing icon/font, long label, complex
script and invalid token/layout without losing old state. Screenshots do not
satisfy any of these assertions. P2-03 implements state/action execution;
P2-05/P2-06 execute rendering, routing and these injected faults.

## Open adapter gates

P2-05's [custom candidate](../05-delivery/phase-2/custom-ui-runbook.md) now executes
this corpus through portable Rust layout/paint/input/semantic code and a host-only
Windows adapter. Its reviewed candidate-specific baselines are in
[custom-v1](../../fixtures/m2/custom-v1/README.md); no shared corpus/lock/tolerance
was changed. Native IME/accessibility, full motion interpolation, optimized damage,
guest integration and live reload remain open. P2-06 uses the same common inputs
and independently reports its outcomes.

For each candidate record actual buffer copy/release behavior, host pixel conversion,
event-loop clock and overflow/reset injection, DPI coordinate transform, private-to-
logical semantic mapping, virtualization, font fallback/shaping, blur parity and
token/layout reload support. Slint need not export this tree; both must provide
the common software buffers and independent model/semantic observations. Exact
hosted and freestanding feature/link graphs and license questions remain P2-04–
P2-09 gates. No new third-party dependency is adopted here. The renderer/host
window code remains outside the future portable Rust UI/domain core.
