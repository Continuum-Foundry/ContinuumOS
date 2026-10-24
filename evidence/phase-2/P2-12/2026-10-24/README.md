# P2-12 — Hosted damage, idle and reload stability pass local acceptance

**P2-12 is Done.** This is Codex local implementation/self-review, not independent
approval or completed M2. Guest UI/reload, efficiency targets and the 256 MiB guest
cold-idle budget remain unaccepted. Source base: `b4d394fdb0173afd72240859518c3ae11a52fcab` plus the exact
[tested inputs](tested-inputs.json) and [source provenance](source-provenance.json).
No dependency pin, frozen Slint input, kernel/ISO source, signing enrollment or
host policy changed. Changes remain local.

Final run: `build/m2/P2-12/acceptance-1792816902462`.
All **ten fresh prerequisites, 13 acceptance commands and four Python consumers**
pass. The final source guard matches. Rust tests pass **23 custom-ui + 17
editor-model + 3 text-core + 26 xtask**, plus 23 behavior-feature cases and
`x86_64-unknown-none` checking. The pre-existing ignored paint test is reported;
resource reload separately exercises real paint-pressure rejection.

Signed optimized executable SHA-256:
`8eb80d7d9d912b25a9571a0f8c7e22d3d55023f363ef10759f2b28dd5fc9ed2c`.
Rust 1.99.1 / LLVM 23.1.1, `m2-comparison`, recorded Windows/i7-1165G7 host.
See [commands/profile](acceptance.json), [criterion review and follow-ups](review.json),
[raw-record review](performance-review.json) and [host conditions](host-environment.json).

## Accepted behavior

- One retained framebuffer and previous owned scene/model snapshot; old/new subtree
  bounds, merged physical damage and painter-order redraw. Rich backdrops,
  fragmentation above 32 contributions and coverage above 60% fall back fully.
  Static content reuses pixels. Native preview and measurement share this core.
- Selected text layout LRU: exact text plus every Config field, one immutable
  pinned-font engine, 128 entries / 2 MiB including owned capacities/metadata.
  Glyph bitmaps retain 128 entries / 1 MiB. **300 cache probes and 108 image
  comparisons** match uncached full redraw with zero byte tolerance, including
  preedit removal, scroll, clipping, opacity, occlusion, scale and changed backgrounds.
- The unchanged **150 captures / 300 workflow states, 45 text and 63 integrated
  text cases**, selected 30-configuration suite and independent edge/semantic
  tolerances pass. Native generation 7-to-8 continuity remains in one window/worker
  with complete state and continued editing; see [lifecycle](native-lifecycle.json),
  [before](native-before.png), [after](native-after.png) and
  [state/pixel consumer](resource-independent-review.json).
- **100 measured file reloads after five warm-up cycles at each resolution**
  preserve the complete checkpoint after every commit. Old readers remain valid,
  then roots retire to exactly one after 100 ms settlement. Fourth-root staging
  refuses safely and succeeds after release. No UI cache is cleared to hide growth.
- Native static/hidden/minimized phases submit zero frames. The caret submits
  **20 frames in 10.065 s**; its worker uses 2515.625 ms process CPU. Quiet phases
  report zero worker CPU delta. Presenter/test observer costs remain visible in
  [native idle](native-idle.json). Rust's six-view clock/caret workload is separate;
  no wall-clock text widget or periodic accessibility timer is claimed.

## Measurements and misses

There are **3,000 frame and 6,763 process samples** across five fresh-process
repetitions and paired tracing, with all six geometry/scale configurations and
six workload families. Process observations run independently every nominal 100 ms;
no missed observer slots were recorded. Active durations overrun the requested
30 s when needed to retain every requested frame. Idle uses actual elapsed
intervals and skips overdue timer periods. See [metrics](metrics.json),
[phase distributions](phases-1920.json) and the CSV samples.

**2,899 / 3,000 frames exceed the 16.667 ms elapsed frame budget.** At scale 1,
trace-off p95 frame durations span **106.744–147.223 ms at 720p** and
**263.304–329.712 ms at 1080p** across the named workloads. These are RAM-consumer
elapsed durations, not measured VSync drops, per-thread CPU or guest input latency.
The eight-/sixteen-millisecond render CPU targets are not certified by wall clocks.
Paired p95 tracing deltas range from **−68.341 to +38.121 ms**; host variation is
material, no stable constant overhead is inferred and nothing is subtracted.

| Actual resource cycles | 720p | 1080p |
| --- | ---: | ---: |
| Measured cycles | 100 | 100 |
| Build p50 | 95.242 ms | 67.970 ms |
| Completed-build-to-RAM p95 | 900.691 ms | 1107.989 ms |
| Completed-build-to-RAM p99 | 1005.395 ms | 1149.573 ms |
| Peak requested Rust allocation during staging | 56,636,513 B | 75,068,517 B |
| Maximum settled private-commit increase | 4,096 B | 0 B |

The **500 ms resource target is missed**. Builder timing is separate; visibility
includes real debounce, validation, staging and RAM submission, not scanout or
compilation. Host variability prevents causal speed comparisons across resolutions.
[PERF-12-01 through PERF-12-03](review.json) bound the raster/group-copy, staged-frame
reuse and timer-damage work. Fidelity, validation and budgets must stay intact.

## Memory and rejection review

Predeclared tolerances are 64 KiB live Rust allocation, 4 MiB private commit and
8 MiB working set relative to warm-up. The observed raw Rust increase is 708 B:
704 B for one baseline counter Value and 4 B across four decimal build-ID copies.
After that accounting, all 100 measured samples form an **exact live-allocation
plateau at each resolution**. Cache limits hold on every cycle; no retired root
survives settlement. Raw cycle resources/checkpoints remain in the CSV files.

The six-view process peaks at **146,145,280 B working set / 143,024,128 B private
commit counter**. Fixed frame/RGB storage is 62,899,200 B and raw fonts 18,005,976 B;
caches, decoded font tables, models/snapshots, observation buffers and runtime costs
remain charged. [Additional accounting](additional-memory-review.json) quantifies
other requested Rust allocation and observation coverage. Process counters include
OS/runtime heaps, stacks and mapped pages; categories overlap and are not summed
as disjoint totals. Native worker/presenter are separate observations. These host
numbers cannot pass the guest budget; guest subsystem costs remain unmeasured.

Self-review found a pre-existing publication defect: a translucent desktop root
could advance generation 7 to 8 before the native host rejected its pixels.
The [original failing probe](opaque-root-before-fix.json) is preserved. Validation
now requires an opaque bottom layer before generation creation. Unit alpha
0/128/254 and builder/API negatives at both resolutions preserve completed output,
active generation, scene, full checkpoint and frame hashes. The
[fixed native host probe](opaque-root-after-fix.json) stays on 7 and accepts another
edit. No field, native code update class or durable activation was added.

[Development findings](development-findings.json) retain sandbox-signing,
incomplete-edit, overdue-observer and superseded-run records. Full logs, binaries,
source/patch copies, traces and images stay local under ignored `build/`/`work/`.
This compact bundle retains raw metric CSVs, normalized observations and hashes of
local inputs; its manifest covers only retained files. Use the
[runbook](../../../../docs/05-delivery/phase-2/performance-stability-runbook.md).
P2-13 receives the final baseline and mandatory clean selected-link/M2 review;
ADRs remain Proposed and M3/M4/M5 obligations remain open.
