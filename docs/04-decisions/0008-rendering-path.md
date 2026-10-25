# ADR-0008: Portable software renderer before accelerated graphics

Status: Proposed. Date: 2026-10-17.

P2-02 refines the experiment with a [locked hosted boundary/corpus](../06-contracts/hosted-ui-fixtures.md):
encoded-sRGB premultiplied RGBA8, explicit leases/damage, analytic paint vectors
and identical rich/reduced-effect cases. These data checks do not pass the M2/M4
render/measurement gate or select a toolkit. Status remains Proposed.

P2-07 [closes both real candidate standalone final links](../../evidence/phase-2/P2-07/2026-10-22/README.md)
with owned bounded probe allocation and independent ELF/codegen inspection.
The extracted Slint software/text-image adapter is shared with the hosted harness.
This is compile/link feasibility only; guest presentation, user context/heap policy,
measured efficiency and the M2/M4 rendering gates remain open.

P2-08's [hosted cost comparison](../../evidence/phase-2/P2-08/2026-10-22/measurement-summary.md)
now measures full redraw/copy paths, raw damage/idle/allocation/process costs and
frequent frame-budget misses. Slint's per-paint shown components retain resources;
this is a current bridge-lifetime defect, not a GPU requirement or intrinsic toolkit
footprint. CPU damage/cache/lifetime work remains before efficiency acceptance;
no host result passes the guest budget or changes this Proposed status.

## Context and decision

P2-09 now [provisionally selects the custom CPU path](../05-delivery/phase-2/ui-selection-review.md)
with exact source/asset/license pins and M3/M4 surface/input/semantic integration
obligations. Full damage, expensive raster and frame misses are retained findings;
P2-10/P2-12 must complete invalidation, bounded caches, correct incremental damage
and remeasurement. No acceleration, guest budget or final rendering gate passes
by this decision; status remains Proposed.

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
This separate experiment is now M18/B29. M14 and one M15 physical-PC profile
retain software presentation; acceleration is not a release prerequisite. Require
resource/fence/reset tests, measured workload distributions and a maintained fallback.

Evidence: [S39](../01-research/sources.md#s39), [S43](../01-research/sources.md#s43),
[S37](../01-research/sources.md#s37), [graphics spec](../02-architecture/graphics-and-compositor.md).

## P2-12 hosted measurement update — 2026-10-24

[Local acceptance](../../evidence/phase-2/P2-12/2026-10-24/README.md) now supplies
bounded retained damage and layout caching, exact full-render parity, static and
hidden pacing, measured timer costs and 100 real file reloads at each resolution
with an exact normalized live-allocation plateau. It also fixes opaque-root
validation before publication. Frame and resource-latency misses are substantial;
PERF-12-01–03 name bounded optimization work. Host elapsed/process observations do
not pass guest targets. P2-13 must review the final clean selected link and M2 gate;
M4 guest confirmation remains open. This ADR remains **Proposed**.

## P2-13 final hosted review — 2026-10-24

[Final aggregate acceptance](../../evidence/phase-2/P2-13/2026-10-24/README.md) now passes the declared M2
hosted subset and clean final linking of the selected retained/damage/layout-cache
path. Final conformance, same-window L0/L1 continuity, 200 measured file reloads,
idle and five paired optimized measurements pass their evidence gates. Frame and
resource-latency misses remain in the review and PERF-12-01–03; no guest budget is
accepted. This ADR remains **Proposed** pending its guest and later gates.


## Hosted performance remediation — 2026-10-24

[The bounded local follow-up](../../evidence/phase-2/performance-remediation/2026-10-24/README.md) now verifies
exact opaque spans, clipped group scratch, owned validated-frame publication and
conservative caret/backdrop damage against the unchanged oracle. No persistent
filtered-backdrop cache or new dependency was needed. Primary 1080p/scale-1 trace-off elapsed frame p95 ranges from 0.789 to 18.642 ms across the twelve workload/effect cells (1,000 samples each). Real file-reload completed-build-to-RAM p95 is 129.527 ms at 720p / 128.116 ms at 1080p. Rich typing/search p95 (18.642 ms) and effects/motion p95 (17.424 ms) still exceed the 16.667 ms period; ten other primary trace-off cells are below it. All misses remain recorded; elapsed/RAM results do not accept renderer-thread CPU, guest input or whole-guest memory targets.
All M1 and final hosted regressions pass. The evidence supports continuing the
software path to guest qualification while keeping remaining misses visible.
This ADR remains **Proposed**; M4 guest and distribution gates are unchanged.
