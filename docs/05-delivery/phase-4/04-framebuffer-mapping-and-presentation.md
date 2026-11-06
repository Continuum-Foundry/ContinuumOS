# P4-04 — Present a checked guest framebuffer

Issue type: Story. Epic: P4. Status: Not started. Owner: Platform/Graphics.
Depends on: P4-03. Backlog: B16 display backend.
Requirements: R01/R08/R09/R10/R17. Test families: T01/T03/T04/T05/T09.
Labels: `phase-4`, `display`, `framebuffer`.

## User story

As a graphics client, I need a real guest output with validated pixels and a clear
submission boundary so a successful RAM render cannot be mistaken for presentation.

## Implementation scope

1. Validate actual boot framebuffer dimensions, pitch, extent, pixel/channel format,
   reserved physical ranges and mapping/cache policy. Reject unsupported formats and
   absent outputs with precise serial diagnostics; do not call exited firmware APIs.
2. Expose scanout only to the trusted display role through P4-03's resource path.
   Implement checked software-buffer conversion/copy, clipping and complete initial
   clear, including padded stride. App processes never receive scanout authority.
3. Define submit/read completion and buffer-release behavior. Describe the real
   backend's lack of page-flip/VSync/scanout observation where applicable. An internal
   copy fence proves reuse safety, not tear-free monitor presentation.
4. Package actual display code in the independently bootable ISO. Capture QEMU's
   displayed output as well as independent CPU-buffer results at 720p and 1080p.

## Acceptance criteria

- [ ] The exact optical ISO displays identified test patterns at both declared
  resolutions; channel order, pitch, clipping and initial background match references.
- [ ] Overflow, overlapping reservation, unsupported format and missing-output
  inputs fail safely with no out-of-range writes or false desktop-ready marker.
- [ ] Unauthorized app mapping/capture and stale device generation deny; buffer
  mutation/reuse follows the actual backend completion rule.
- [ ] Serial remains responsive on presentation error; boot metadata, memory,
  exception and timer regressions pass.
- [ ] Evidence separates CPU raster, submission/copy completion and observed guest
  display; no scanout-latency or VSync claim follows from a screenshot.

## Tests and evidence

Use edge/corner/pitch/channel patterns, padded rows, invalid dimensions/extent and
allocation/map failure. Inspect actual page permissions/cache policy, display capture
and serial identities. Retain elapsed copy/counter observations as instrumentation,
not final desktop performance results.

## Exclusions and handoff

No compositor, toolkit, mode switching or accelerated renderer yet. P4-05 receives
a bounded presentation backend. Read [boot contract](../../02-architecture/boot-and-platform.md)
and [graphics](../../02-architecture/graphics-and-compositor.md).
Apply the [epic definition of done](README.md#evidence-and-story-completion).
