# Graphics and compositor design

Status: proposed. R08, R09, R15. Decisions: [ADR-0008](../04-decisions/0008-rendering-path.md).

## Pipeline and interfaces

Input event → app state update → incremental layout → display list/scene commit →
rasterization → surface commit → compositor damage/visibility → presentation.
Measure each stage separately. A frame deadline does not describe total input latency.

Use a retained scene with stable node IDs, explicit transforms/clips, ordered paint
commands, resource references and semantic identity. Apps own content; compositor
owns z-order/presentation and applies shell-controlled geometry. Start with software
surfaces and one display. A host backend presents the same renderer for fast iteration.
Revision 2 compares Slint/custom implementations through common app-surface/input
contracts. A toolkit may own a private tree and submit raster buffers; it need not
export our optional custom scene-node format. OS composition, window authority and
live instance handover remain ours.

Retained scenes and compatible batching are established rendering techniques;
surface damage and frame pacing are explicit in mature compositor designs.
[S39](../01-research/sources.md#s39), [S40](../01-research/sources.md#s40),
[S42](../01-research/sources.md#s42)

## Baseline renderer

P2-09 [provisionally chooses the existing custom CPU renderer](../05-delivery/phase-2/ui-selection-review.md).
Portable scene/paint/surface code stays in custom-ui, text in text-core, host
presentation in xtask. M4 must adapt versioned brokered surface leases, monotonic
frame/wait services and input/semantic dispatch; no kernel renderer or working
guest surface is implied. P2-12 must validate partial damage against full redraw,
including blur backgrounds, old/new bounds and translucent overlap, and bound
cache/scratch/old+new generation memory before claiming an optimization. Existing
frame misses do not change the targets or justify silently reducing rich mode.

Implement clear/fill, rectangles/rounded rectangles, clipped images, glyph runs,
transforms and alpha composition. Define premultiplied alpha, pixel format, stride,
color-space conversion and blend behavior. sRGB versus linear-light choices must
be explicit and tested; an arbitrary gradient buffer is not a color pipeline.

Damage expands to cover antialiasing, shadows, filter radii and old/new transformed
bounds. Moving a window damages both exposed old regions and its new footprint.
Occlusion rejection is safe only behind truly opaque coverage; translucent windows
still depend on content beneath them. Bound region complexity and fall back to
larger repaint when tracking costs exceed savings.

Double-buffer initially and define who may write each buffer. Never mutate a buffer
still being displayed/read by the compositor. Presentation fencing and release
events determine reuse. A boot framebuffer may lack reliable tear-free page flips;
state that limitation instead of claiming perfect VSync.

## Efficient visual effects

- Cache glyphs, reusable shadows and static complex subtrees with per-class budgets.
- Use downsampled blur only where visually justified; invalidate when sampled
  background content changes. Animated backgrounds cannot use a permanently stale cache.
- Animate transforms/opacity of retained app surfaces without requiring app re-layout.
- Precompute assets or shaders at build time only where backend/toolchain support exists.
- Sleep without damage or active animation; use display-paced work rather than a busy loop.
- Reduce effect radius/resolution for lower profiles using explicit quality settings;
  do not silently degrade text or make accessibility depend on GPU capability.

Offscreen effect layers can add render passes and memory traffic. Cache only when
measured reuse pays for memory and invalidation complexity.
[S43](../01-research/sources.md#s43)

## Memory reality

A 1920×1080 RGBA8 buffer consumes 8,294,400 bytes (~7.91 MiB). Two consume ~15.82 MiB.
A 3840×2160 buffer consumes ~31.64 MiB; three consume ~94.92 MiB before any app,
blur, atlas or history buffers. A full 4K RGBA8 write at 120 Hz is ~3.98 GB/s of
payload alone. These are arithmetic lower bounds, not measured bandwidth requirements.
Blending, reads, copies and host transfers can multiply traffic.

Prefer allocating by actual surface size, pooling bounded temporary buffers and
clipping work. Never claim “nearly no memory” because the kernel ELF is small.

## Acceleration gate

VirtIO GPU 2D can transport software-rendered resources. Virgl/rutabaga/other
accelerated paths need supported host backends and a compatible guest command/API
stack. Porting a Vulkan/OpenGL API is not a one-line dependency addition.
[S37](../01-research/sources.md#s37)

Benchmark a fixed feature subset before choosing a GPU backend. Keep software
fallback and scene conformance tests. Do not implement every modern GPU/vendor
driver as a prerequisite for a beautiful first desktop.

## Failure and update behavior

Keep surfaces visible while shell/app behavior updates if their owner survives.
When the compositor restarts, clients must reconnect and resubmit scene/resources;
initially allow a visible recovery interval. GPU device loss invalidates GPU handles
and requires reconstruction. It does not justify trusting serialized pointer values.
