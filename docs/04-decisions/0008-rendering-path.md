# ADR-0008: Portable software renderer before accelerated graphics

Status: Proposed. Date: 2026-10-17.

P2-02 refines the experiment with a [locked hosted boundary/corpus](../06-contracts/hosted-ui-fixtures.md):
encoded-sRGB premultiplied RGBA8, explicit leases/damage, analytic paint vectors
and identical rich/reduced-effect cases. These data checks do not pass the M2/M4
render/measurement gate or select a toolkit. Status remains Proposed.

## Context and decision

We need beautiful UI without pretending a new kernel already has a GPU stack.
Share a retained scene/UI engine between a hosted preview and guest software
presentation. Add damage tracking, bounded caches, pacing and clear color/alpha
semantics. Compare Slint and a minimal custom UI/raster path before choosing the
shared implementation; a toolkit may submit pixels without exposing our scene
graph. Evaluate acceleration at a separate VirtIO GPU gate.

## Alternatives

Immediate full redraw is simple but expensive at high resolutions. GPU-first can
stall the project on guest graphics drivers. Porting a full toolkit might reduce UI
work but imports substantial platform/runtime dependencies.

## Consequences

Software rendering limits high-resolution/high-refresh effects; these are measured
limits, not architectural failure. Shared scene semantics aid testing but do not
prove host and guest timing equivalence. Fallback supports recovery without GPU code.

## Gate and reconsideration

M2/M4 render the same reference scenes in host/guest and profile damage, idle work,
memory and frame distributions. Select acceleration only after guest/host backend
feasibility and real benefit are demonstrated.

Evidence: [S39](../01-research/sources.md#s39), [S43](../01-research/sources.md#s43),
[S37](../01-research/sources.md#s37), [graphics spec](../02-architecture/graphics-and-compositor.md).
