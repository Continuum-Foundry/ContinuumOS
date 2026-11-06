# Display roadmap: resolution, scaling and multiple outputs

Status: owner-requested planning amendment, 2026-11-01. Implementation and guest
acceptance remain open. No new P-level stories are created by this plan.
Requirements: R08/R09/R10/R14/R15/R17/R18; continuity R04/R05/R07/R12.
See [milestones](milestones.md), [backlog](backlog.md),
[graphics](../02-architecture/graphics-and-compositor.md) and
[surface contracts](../06-contracts/ui-and-scene-protocol.md).

## Placement and dependency order

Preserve milestone numbers and existing accepted evidence. M4/B16 establishes the
single-output guest baseline and B61's extensible interfaces. M9/B47 seeds display
settings and recoverable window placement. M10/B48 carries supported display/scale
capabilities into SDK documentation. M11/B50/B52 prepares portable preferences and
session recovery without treating a saved monitor identifier as current authority.
M13/B31 checks semantic coordinates against the declared baseline scale.

M4 is now decomposed into [nineteen P4 stories](phase-4/README.md).
[P4-11](phase-4/11-output-scale-and-geometry-contracts.md) owns B61's explicit
output/scale/geometry codecs and single-output guest fixtures, consuming P4-04–P4-10
display/input/UI foundations. P4-19 reviews that subset; stage 3 onward remains M22.

The dedicated M22 display milestone follows accepted M14/B56, consumes those
foundations and completes B62–B66 in order. M15/B57 now requires B66 before named-PC
qualification; actual connector/mode/hot-plug support must then be retested on that
hardware. M22 is required planned platform delivery, independent of M18 acceleration.
Its number is an identifier, not an instruction to wait for M16–M21. M8 and M14
retain their single-display release exits; M22 has its own exact-image review.

## Stages for later story decomposition

| Stage | Gate and broad task | Build and acceptance boundary |
| --- | --- | --- |
| 1 | M4 / B16,B61: one guest output | Render and reload at declared 720p/1080p modes; separate logical layout from physical pixels; define output identity/generation, scale, stride/format and buffer ownership. Validate pointer/hit-test/semantic transforms and checked dimension arithmetic. Seed versioned explicit codecs without claiming several outputs. |
| 2 | M9–M13: desktop integration seeds | Prepare authorized typed display settings, preview/confirm/revert behavior, portable preferences, visible window recovery and SDK scale-change notifications. Real mode changing and extended-desktop acceptance remain M22. Keep installer/login/lock surfaces usable on the supported baseline. |
| 3 | M22 / B62: one 4K output | Discover and select 3840×2160 on one declared VM display backend; render at 100%, 150% and 200% scale, including text, pointer, semantics and resource/app reload continuity. Add mode enumeration and safe mode change with last-working-mode recovery. Firmware framebuffer availability alone cannot prove dynamic mode setting. |
| 4 | M22 / B63: two equal-scale outputs | Extend the desktop to two outputs with independent buffers, damage and presentation ownership; test left/right/above/below layouts, negative desktop coordinates, primary selection, window crossing/spanning, popups, fullscreen placement and pointer/focus/drag continuity. First require two 1080p outputs at 100%. |
| 5 | M22 / B64: mixed sizes and scales | Require 1080p at 100% plus 4K at 200%, then 4K at 150%; change scale while windows cross outputs. Rerasterize text/assets, invalidate scale-keyed caches and preserve logical window/document state. Test cursor hotspots, selection/caret, menus, capture and semantic bounds across seams. |
| 6 | M22 / B65: topology and pacing changes | Support disconnect/reconnect, primary-output loss, mode-change failure and compositor/display-service recovery. Move stranded windows/dialogs to a surviving output; keep lock/login/capture policy intact. Test one output at 60 Hz and another at 75 Hz where the selected backend exposes both; pace independently without letting a slow/idle output stall the other. Unsupported modes reject explicitly. |
| 7 | M22 / B66: aggregate display acceptance | Test the exact optical ISO and installed image, two 4K outputs at 200%, mixed-scale matrix, live resource/app replacement and at least 100 connect/mode/scale/recovery cycles with unsaved work preserved. Reconcile steady/peak memory and buffer retirement; retain visual, semantic, latency, idle and failure evidence. |

These are ordered engineering slices, not pre-created stories or implementation
results. Split each broad task into reviewed P-level stories only when its milestone
is decomposed. Gate failures remain blockers/findings; do not silently reduce the
required matrix or replace guest evidence with hosted screenshots. A backend that
cannot expose the required outputs/modes blocks that subset and needs a reviewed
backend experiment before support can be claimed.

## Contracts, ownership and recovery

Before freezing the display ABI, independently test explicit encodings for output
enumeration, mode/scale/topology revisions, acquire/commit/release and notifications.
Separate connector/output identity, configured desktop position and logical surface
identity. Output generations reject stale commits after removal or mode changes.
Check signed desktop coordinate transforms, rational/fractional scaling, clipping,
stride/size overflow and resource admission. Apps receive scoped surface authority;
they cannot set global modes, capture other sessions or mint output authority.

Prepare a complete candidate topology and reserve old/new headroom before publish.
Failed admission leaves the current topology usable; release old buffers only after
all readers/presentation fences complete. Settings preview has a bounded monotonic
confirmation timeout and reverts on expiry or lost confirmation. Never revive a
removed output merely because a stored identifier matches. Initially require an
extended desktop; mirroring, rotation, HDR, VRR and arbitrary GPUs remain separate
future gates. Refresh-rate independence does not promise tear-free scanout on a
backend without presentation timing/fence support.

## Resource and measurement gates

Keep the existing 256 MiB cold-idle target for its declared baseline. Before M22
performance runs, lock separate 4K/dual-output profiles, memory/peak headroom and
frame/input limits, workloads, seeds and supported refresh rates. Report whether
each extended profile meets 256 MiB; any larger profile budget needs an explicit
reviewed amendment before acceptance and cannot replace the baseline target.

One tightly packed 3840×2160 RGBA8 buffer is 33,177,600 bytes (31.64 MiB).
Two double-buffered 4K outputs need 132,710,400 bytes (126.56 MiB) for those buffers
alone. These are arithmetic lower bounds; stride alignment, app surfaces, cursor,
glyph/effect caches, driver/host copies and old/new mode buffers add costs. Count
shared backing once globally, show attribution and distinguish guest from host RAM.

Retain per-output frame/composition CPU and guest input-to-present-submission
p50/p95/p99/max, sample counts, dropped/missed frames, idle wake/work and allocation
peaks. Record actual backend completion where observable; submission is not scanout.
Repeat under window movement, text editing, fractional scaling, effects and reload.
Require bounded settling with no unexplained growth after the 100-cycle sequence,
no in-use buffer reuse and usable recovery on exhaustion/device loss. M18 can later
compare acceleration using the same corpus; 2D scanout transport is distinct from
accelerated rendering and no existing GPU stack is assumed.

## Qualification handoff

B66 hands M15 an exact supported VM matrix, backend/driver obligations, reproducible
commands, limits and failure/recovery cases. B57 selects actual connectors, monitors,
firmware and driver requirements on one named PC; B58 reviews its installed/repair/
rescue lifecycle including supported output loss. If the PC cannot supply required
display coverage, record the gap and replan that profile rather than infer support.
No physical-disk operation, driver port, remote publication or performance result is
authorized or established by this planning amendment.
