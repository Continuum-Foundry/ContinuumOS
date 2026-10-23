# Draft contract: UI, scenes and surfaces

Status: design draft, version 0.1. R04, R08, R15.
Revision 2 separates common surface/input/buffer contracts from the optional custom
scene-node format. A toolkit can render privately and submit a buffer without
implementing SceneCommit. Freeze integration after the comparison; semantic/action
requirements remain in either path.

P2-02's [hosted comparison refinement](hosted-ui-fixtures.md) now fixes version-1
geometry, encoded-sRGB premultiplied blending, buffer ownership, finite bounds,
event ordering/cancellation and token/layout data limits with authored fixtures.
It is an experimental data contract, not a frozen process ABI or implemented
compositor. The optional SceneCommit design below remains a separate integration.

## Scene transaction

P2-09 [provisionally selects custom-ui](../05-delivery/phase-2/ui-selection-review.md)
behind these boundaries. Its internal Rust Scene/Controller/surface types remain
local implementation values. M3/M4 must validate explicit codecs, owner/session
authority, epochs, quota and release semantics before any cross-process use.
The guest backend receives software pixels through brokered leases and exports
semantic snapshots/actions separately; it does not inherit hosted struct layouts
or toolkit window authority. P2-11 owns coherent hosted generation publication.

An app submits `SceneCommit(surface_id, scene_epoch, base_revision, new_revision,
resource_generation, operations, declared_damage)`. The compositor validates the
whole commit and publishes all or none of it. Reject stale base revisions, cycles,
unknown resources, illegal references or excessive complexity before publication.
Scene data contains IDs and values, not executable callbacks or native pointers.

Initial operations: create/update/remove node, set transform/clip/opacity, set paint
primitive, attach image/glyph resource, reorder children, update hit-test/semantic
reference. Paint primitives have bounded arrays and finite numeric values; reject
NaN/infinite transforms, invalid bounds and pathological recursion.

## Identity

Surface IDs belong to an app/session authority. Node/widget IDs are stable logical
IDs within that surface. Allocation addresses and draw-order indices are not identity.
Reload reconciliation retains compatible IDs and explicitly retires missing ones.
Semantic nodes may refer to visual bounds but are not forced to mirror paint nodes.

## Resources

Each resource records type, dimensions, stride/format, immutable digest or mutable
buffer lease, owner, bytes and reference count. The compositor checks lengths against
format/stride with overflow protection. Images/fonts remain untrusted parser inputs.
Resources referenced by an old scene stay alive until that scene/presentation reader
releases them. Eviction cannot invalidate an in-use handle.

## Surface buffers

Use `acquire → write → commit → in_use → released` ownership. The writer cannot
modify an in-use buffer. A CPU fallback may copy into a compositor-owned buffer;
zero-copy is a future measured optimization. One display's initial format is
RGBA8/premultiplied alpha with explicitly defined conversion to scanout format.
An opaque hint is verified/limited by contract; lying must not reveal uninitialized
pixels or break privacy through stale damage assumptions.

## Damage and pacing

Client damage is a hint validated against known operations; compositor computes
old/new bounds, clip/filter expansion and exposed areas. Limit region fragmentation.
`RequestFrame` grants opportunity based on visibility and output scheduling; do not
ask hidden apps to continuously animate. A release/fence event is distinct from a
frame callback. Hardware page-flip support must be real before claiming tear-free
presentation.

## Input/events

Events include monotonic timestamp, input sequence, target logical ID, coordinates
with a known transform/scene revision, and device/action data. Reconcile layout
changes with hit testing; never send a click to a control based on stale geometry
without a policy. Capture/focus changes are explicit and cancel on owner death.

Update quiescence completes or cancels active gestures and IME composition according
to the UI state schema. Buffered inputs are bounded and ordered. Overflow generates
a recoverable cancellation/reset, not silent loss of key-up events.

## Test corpus

Golden scenes include rounded clips, gradients, text, translucent overlap, scrolling,
old/new window bounds, fractional scaling and missing resources. Compare software
and accelerated paths against declared pixel tolerances. Semantic tests are separate
from screenshots and include focus/action behavior after reload.
